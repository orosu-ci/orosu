use crate::model::{DatabaseTask, DatabaseUuid, TaskArguments};
use crate::tasks::{ActiveTask, ActiveTaskKey, TaskLaunchResult, TaskUpdatedNotification, Tasks};
use deadpool_diesel::sqlite::Pool;
use diesel::ExpressionMethods;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast::channel;
use tokio::sync::Mutex;
use uuid::Uuid;

impl Tasks {
    pub fn new(pool: Pool) -> Self {
        let (tx, _) = channel::<TaskUpdatedNotification>(32);
        Self {
            pool,
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            changes_tx: tx,
        }
    }

    pub async fn get_or_start(
        &self,
        script_key: Uuid,
        task_id: Uuid,
        script_path: PathBuf,
        arguments: Option<TaskArguments>,
    ) -> anyhow::Result<TaskLaunchResult> {
        let active_tasks = self.active_tasks.clone();
        let mut lock = active_tasks.lock().await;
        let key = ActiveTaskKey {
            script_key,
            task_id,
        };

        if let Some(active_task) = lock.get(&key) {
            tracing::info!("Task {} already running", task_id);
            return Ok(TaskLaunchResult::Created(active_task.clone()));
        } else {
            let connection = self.pool.get().await.map_err(|e| anyhow::anyhow!(e))?;
            let database_task = connection
                .interact(move |conn| {
                    use crate::schema::tasks::dsl::*;
                    tasks
                        .select(DatabaseTask::as_select())
                        .filter(id.eq(&DatabaseUuid(task_id)))
                        .first::<DatabaseTask>(conn)
                })
                .await
                .map_err(|e| anyhow::anyhow!("Interaction error: {}", e))?;
            if let Ok(database_task) = database_task {
                tracing::info!("Task {} already finished", task_id);
                return Ok(TaskLaunchResult::Finished(database_task.into()));
            }
        }

        let (join_handle, new_task) = ActiveTask::create_and_run(
            script_key,
            task_id,
            script_path,
            arguments,
            self.changes_tx.clone(),
            self.pool.clone(),
        );

        lock.insert(key.clone(), new_task.clone());
        drop(lock);

        let active_tasks = self.active_tasks.clone();
        tokio::spawn(async move {
            join_handle.await.unwrap();
            let mut lock = active_tasks.lock().await;
            lock.remove(&key);
        });

        Ok(TaskLaunchResult::Created(new_task.clone()))
    }
}
