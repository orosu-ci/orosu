use crate::script::Script;
use crate::tasks::task::Task;
use crate::tasks::{ActiveTaskKey, TaskLaunchResult, Tasks};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

impl Tasks {
    pub fn new() -> Self {
        Self {
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_or_start(
        &self,
        run_id: Uuid,
        client_name: String,
        script: Script,
        arguments: Vec<String>,
    ) -> anyhow::Result<TaskLaunchResult> {
        let active_tasks = self.active_tasks.clone();
        let mut lock = active_tasks.lock().await;
        let key = ActiveTaskKey {
            script_name: script.name.clone(),
            client_name,
            run_id,
        };

        if let Some(active_task) = lock.get(&key) {
            tracing::info!("Task {} already running", run_id);
            let active_task = active_task.clone();
            let active_task = active_task.read().await;
            return active_task.join().await;
        }

        let task = Arc::new(RwLock::new(Task::create(script.clone())));
        lock.insert(key, task.clone());

        let task = task.read().await;
        task.run(arguments).await
    }
}
