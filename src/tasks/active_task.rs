use crate::model::{DatabaseTimedTaskEvents, DatabaseUuid, TaskArguments};
use crate::tasks::{
    ActiveTask, ActiveTaskKey, TaskEvent, TaskOutput, TaskUpdatedNotification,
    TaskUpdatedNotificationBody, TimedTaskEvent,
};
use deadpool_diesel::sqlite::Pool;
use diesel::dsl::{insert_into, update};
use diesel::{ExpressionMethods, RunQueryDsl};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast::{channel, Sender};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

impl ActiveTask {
    pub fn create_and_run(
        script_key: Uuid,
        task_id: Uuid,
        script_path: PathBuf,
        arguments: Option<TaskArguments>,
        tasks_notifications: Sender<TaskUpdatedNotification>,
        database_pool: Pool,
    ) -> (JoinHandle<()>, Arc<RwLock<Self>>) {
        let (tx, _) = channel::<TimedTaskEvent>(32);
        let task = Self {
            created_on: chrono::Utc::now(),
            launched_on: None,
            finished_on: None,
            script_key,
            task_id,
            exit_code: None,
            events: vec![],
            events_tx: tx,
            arguments,
        };
        let task = Arc::new(RwLock::new(task));

        let join_handler = tokio::spawn(Self::run(
            task.clone(),
            tasks_notifications,
            database_pool,
            script_path.clone(),
        ));
        (join_handler, task.clone())
    }

    async fn run(
        task: Arc<RwLock<Self>>,
        tasks_notifications: Sender<TaskUpdatedNotification>,
        database_pool: Pool,
        script_path: PathBuf,
    ) {
        let task_id = task.read().await.task_id;
        let script_key = task.read().await.script_key;
        let arguments = task.read().await.arguments.clone();

        let key = ActiveTaskKey {
            task_id,
            script_key,
        };
        let tx = task.read().await.events_tx.clone();

        // Task created, update database, notify inner details stream and parents notification stream
        let inserted_arguments = arguments.clone();
        database_pool
            .get()
            .await
            .unwrap()
            .interact(move |conn| {
                use crate::schema::tasks::dsl;
                insert_into(dsl::tasks)
                    .values((
                        dsl::id.eq::<DatabaseUuid>(task_id.clone().into()),
                        dsl::script_key.eq::<DatabaseUuid>(script_key.clone().into()),
                        dsl::output.eq(vec![]),
                        dsl::arguments.eq(inserted_arguments.clone()),
                    ))
                    .execute(conn)
            })
            .await
            .unwrap()
            .unwrap();

        _ = tasks_notifications
            .send(TaskUpdatedNotification::now(
                key.clone(),
                TaskUpdatedNotificationBody::Created {
                    arguments: arguments.clone(),
                },
            ))
            .inspect_err(|e| {
                tracing::error!("Failed to send created notification to task manager: {}", e)
            });

        // Launching task

        let mut command = tokio::process::Command::new(&script_path);

        if let Some(arguments) = arguments.clone() {
            for argument in arguments.iter() {
                command.arg(argument);
            }
        }

        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(err) => {
                // Task launch failed, update database and notify inner details stream

                let event = TimedTaskEvent::now(TaskEvent::Started);
                task.write().await.events.push(event.clone());
                _ = tx.send(event);
                _ = tasks_notifications
                    .send(TaskUpdatedNotification::now(
                        key.clone(),
                        TaskUpdatedNotificationBody::Launched {
                            arguments: arguments.clone(),
                        },
                    ))
                    .inspect_err(|e| {
                        tracing::error!(
                            "Failed to send launched notification to task manager from failed task launch: {}",
                            e
                        )
                    });

                let events = task.read().await.events.clone();
                database_pool
                    .clone()
                    .get()
                    .await
                    .unwrap()
                    .interact(move |conn| {
                        use crate::schema::tasks::dsl;
                        update(dsl::tasks)
                            .set((
                                dsl::launched_on.eq(diesel::dsl::now),
                                dsl::output.eq::<DatabaseTimedTaskEvents>(Into::into(events)),
                            ))
                            .filter(dsl::id.eq::<DatabaseUuid>(task_id.clone().into()))
                            .execute(conn)
                    })
                    .await
                    .unwrap()
                    .unwrap();

                let event = TaskEvent::Output(TaskOutput::Stderr(format!(
                    "Failed to start script: {}",
                    err
                )));
                let event = TimedTaskEvent::now(event);
                task.write().await.events.push(event.clone());
                let _ = tx.send(event);

                let event = TaskEvent::Finished(-1);
                let event = TimedTaskEvent::now(event);
                task.write().await.events.push(event.clone());
                let _ = tx.send(event);

                _ = tasks_notifications
                    .send(TaskUpdatedNotification::now(
                        key.clone(),
                        TaskUpdatedNotificationBody::Finished {
                            arguments: arguments.clone(),
                            exit_code: -1,
                        },
                    ))
                    .inspect_err(|e| {
                        tracing::error!(
                            "Failed to send finished notification to task manager from failed task launch: {}",
                            e
                        )
                    });

                let events = task.read().await.events.clone();
                database_pool
                    .clone()
                    .get()
                    .await
                    .unwrap()
                    .interact(move |conn| {
                        use crate::schema::tasks::dsl;
                        update(dsl::tasks)
                            .set((
                                dsl::exit_code.eq(Some(-1)),
                                dsl::finished_on.eq(diesel::dsl::now),
                                dsl::output.eq::<DatabaseTimedTaskEvents>(Into::into(events)),
                            ))
                            .filter(dsl::id.eq::<DatabaseUuid>(task_id.clone().into()))
                            .execute(conn)
                    })
                    .await
                    .unwrap()
                    .unwrap();
                return;
            }
        };

        let event = TimedTaskEvent::now(TaskEvent::Started);
        task.write().await.events.push(event.clone());

        let events = task.read().await.events.clone();
        database_pool
            .clone()
            .get()
            .await
            .unwrap()
            .interact(move |conn| {
                use crate::schema::tasks::dsl;
                update(dsl::tasks)
                    .set((
                        dsl::launched_on.eq(diesel::dsl::now),
                        dsl::output.eq::<DatabaseTimedTaskEvents>(Into::into(events)),
                    ))
                    .filter(dsl::id.eq::<DatabaseUuid>(task_id.clone().into()))
                    .execute(conn)
            })
            .await
            .unwrap()
            .unwrap();

        _ = tx.send(event);
        _ = tasks_notifications
            .send(TaskUpdatedNotification::now(
                key.clone(),
                TaskUpdatedNotificationBody::Launched {
                    arguments: arguments.clone(),
                },
            ))
            .inspect_err(|e| {
                tracing::error!(
                    "Failed to send launched notification to task manager: {}",
                    e
                )
            });

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let tx_stdout = tx.clone();
        let tx_stderr = tx.clone();

        let moved_stdout_task = task.clone();
        let moved_stderr_task = task.clone();

        let stdout_task = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let event = TaskEvent::Output(TaskOutput::Stdout(line));
                let event = TimedTaskEvent::now(event);
                moved_stdout_task.write().await.events.push(event.clone());
                let _ = tx_stdout.send(event);
            }
        });

        let stderr_task = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let event = TaskEvent::Output(TaskOutput::Stderr(line));
                let event = TimedTaskEvent::now(event);
                moved_stderr_task.write().await.events.push(event.clone());
                let _ = tx_stderr.send(event);
            }
        });

        let _ = tokio::join!(stdout_task, stderr_task);

        let exit_code = match child.wait().await {
            Ok(status) => status.code().unwrap_or(-1),
            Err(e) => {
                tracing::error!("Command failed: {}", e);
                -1
            }
        };

        let event = TimedTaskEvent::now(TaskEvent::Finished(exit_code));
        task.write().await.events.push(event.clone());

        let events = task.read().await.events.clone();
        database_pool
            .clone()
            .get()
            .await
            .unwrap()
            .interact(move |conn| {
                use crate::schema::tasks::dsl;
                update(dsl::tasks)
                    .set((
                        dsl::exit_code.eq(Some(exit_code)),
                        dsl::finished_on.eq(diesel::dsl::now),
                        dsl::output.eq::<DatabaseTimedTaskEvents>(Into::into(events)),
                    ))
                    .filter(dsl::id.eq::<DatabaseUuid>(task_id.clone().into()))
                    .execute(conn)
            })
            .await
            .unwrap()
            .unwrap();

        let _ = tx.send(event);
        _ = tasks_notifications
            .send(TaskUpdatedNotification::now(
                key.clone(),
                TaskUpdatedNotificationBody::Finished {
                    arguments: arguments.clone(),
                    exit_code,
                },
            ))
            .inspect_err(|e| {
                tracing::error!(
                    "Failed to send finished notification to task manager: {}",
                    e
                )
            });
    }
}
