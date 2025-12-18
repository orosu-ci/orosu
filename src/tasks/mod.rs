pub(crate) use crate::tasks::timestamped::Timestamped;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use tokio::task::JoinHandle;

pub(crate) mod task;
mod timestamped;

pub struct TaskLaunchResult {
    pub(crate) created_on: chrono::DateTime<chrono::Utc>,
    pub(crate) output: Sender<Timestamped<TaskOutput>>,
    pub(crate) handler: JoinHandle<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskOutput {
    #[serde(rename = "stdout")]
    Stdout(String),
    #[serde(rename = "stderr")]
    Stderr(String),
}
