use crate::model::TaskArguments;
use crate::tasks::{ActiveTask, ActiveTaskKey, TaskLaunchResult, TaskUpdatedNotification, Tasks};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast::channel;
use tokio::sync::Mutex;
use uuid::Uuid;

impl Tasks {
    pub fn new() -> Self {
        let (tx, _) = channel::<TaskUpdatedNotification>(32);
        Self {
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
        }

        let (join_handle, new_task) = ActiveTask::create_and_run(
            script_key,
            task_id,
            script_path,
            arguments,
            self.changes_tx.clone(),
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
