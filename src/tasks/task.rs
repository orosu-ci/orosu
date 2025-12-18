use crate::script::Script;
use crate::tasks::{TaskLaunchResult, TaskOutput, Timestamped};
use std::ops::Deref;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{broadcast, watch, RwLock};

pub struct Task {
    created_on: chrono::DateTime<chrono::Utc>,
    script: Script,
    exit_code_tx: watch::Sender<Option<Timestamped<i32>>>,
    output: Arc<RwLock<Vec<Timestamped<TaskOutput>>>>,
    output_tx: broadcast::Sender<Timestamped<TaskOutput>>,
}

impl Task {
    pub fn create(script: Script) -> Self {
        let created_on = chrono::Utc::now();
        let (exit_code_tx, _) = watch::channel(None);
        let (output_tx, _) = broadcast::channel(32);
        Self {
            created_on,
            script,
            exit_code_tx,
            output: Arc::new(RwLock::new(Vec::new())),
            output_tx,
        }
    }

    async fn append_stdout<T: Into<String>>(
        collection: Arc<RwLock<Vec<Timestamped<TaskOutput>>>>,
        tx: broadcast::Sender<Timestamped<TaskOutput>>,
        line: T,
    ) {
        Self::append_output(collection, tx, TaskOutput::Stdout(line.into())).await;
    }

    async fn append_stderr<T: Into<String>>(
        collection: Arc<RwLock<Vec<Timestamped<TaskOutput>>>>,
        tx: broadcast::Sender<Timestamped<TaskOutput>>,
        line: T,
    ) {
        Self::append_output(collection, tx, TaskOutput::Stderr(line.into())).await;
    }

    async fn append_output(
        collection: Arc<RwLock<Vec<Timestamped<TaskOutput>>>>,
        tx: broadcast::Sender<Timestamped<TaskOutput>>,
        output: TaskOutput,
    ) {
        let event = Timestamped::now(output);
        let mut lock = collection.write().await;
        lock.push(event.clone());
        if let Err(e) = tx.send(event) {
            tracing::error!("Failed to send task output event: {}", e);
        }
    }

    fn set_exit_code(tx: watch::Sender<Option<Timestamped<i32>>>, exit_code: i32) {
        let event = Timestamped::now(exit_code);
        if let Err(e) = tx.send(Some(event)) {
            tracing::error!("Failed to send task exit code: {}", e);
        }
    }

    pub async fn join(&self) -> anyhow::Result<TaskLaunchResult> {
        let exit_code_tx = self.exit_code_tx.clone();
        let exit_code_subscription = exit_code_tx.subscribe();

        let exit_code_value = {
            let exit_code = exit_code_subscription.borrow();
            exit_code.clone()
        };

        match exit_code_value {
            None => {
                let mut exit_code_rx = exit_code_tx.clone().subscribe();
                let handler = tokio::spawn(async move {
                    loop {
                        exit_code_rx.changed().await.ok();
                        let exit_code = exit_code_rx.borrow();
                        if let Some(code) = exit_code.deref() {
                            return code.value;
                        }
                    }
                });
                Ok(TaskLaunchResult::Joined {
                    created_on: self.created_on,
                    output: self.output.clone().read().await.clone(),
                    output_tx: self.output_tx.clone(),
                    handler,
                })
            }
            Some(exit_code) => Ok(TaskLaunchResult::Finished {
                created_on: self.created_on,
                finished_on: exit_code.timestamp,
                exit_code: exit_code.value,
                output: self.output.clone().read().await.clone(),
            }),
        }
    }

    pub async fn run(&self, arguments: Vec<String>) -> anyhow::Result<TaskLaunchResult> {
        let created_on = self.created_on.clone();
        let output_tx = self.output_tx.clone();
        let script = self.script.clone();

        let mut command = tokio::process::Command::new(script.command);
        if !arguments.is_empty() {
            command.args(arguments);
        }

        tracing::info!("Running script: {:?}", command);

        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
                Self::append_stderr(
                    self.output.clone(),
                    output_tx,
                    format!("Failed to start script: {}", e),
                )
                .await;
                Self::set_exit_code(self.exit_code_tx.clone(), 1);
                return Err(e.into());
            }
        };

        let handler_output_tx = output_tx.clone();
        let handler_output = self.output.clone();
        let result_output_tx = output_tx.clone();
        let handler_exit_code_tx = self.exit_code_tx.clone();

        let handler = tokio::spawn(async move {
            let stdout = child.stdout.take().unwrap();
            let stderr = child.stderr.take().unwrap();

            let stdout_output = handler_output.clone();
            let stdout_output_tx = handler_output_tx.clone();

            let stderr_output = handler_output.clone();
            let stderr_output_tx = handler_output_tx.clone();

            let stdout_task = tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    Self::append_stdout(stdout_output.clone(), stdout_output_tx.clone(), line)
                        .await;
                }
            });

            let stderr_task = tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    Self::append_stderr(stderr_output.clone(), stderr_output_tx.clone(), line)
                        .await;
                }
            });

            let _ = tokio::join!(stdout_task, stderr_task);

            let exit_code = match child.wait().await {
                Ok(status) => match status.code() {
                    None => {
                        Self::append_stderr(
                            handler_output.clone(),
                            output_tx,
                            "Command terminated by signal",
                        )
                        .await;
                        -1
                    }
                    Some(code) => code,
                },
                Err(e) => {
                    Self::append_stderr(
                        handler_output.clone(),
                        output_tx,
                        format!("Command failed: {}", e),
                    )
                    .await;
                    -1
                }
            };
            Self::set_exit_code(handler_exit_code_tx, exit_code);
            exit_code
        });
        Ok(TaskLaunchResult::Created {
            created_on,
            output_tx: result_output_tx,
            handler,
        })
    }
}
