use crate::model::TaskArguments;
pub(crate) use crate::tasks::timestamped::Timestamped;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub(crate) mod task;
mod timed_task_event;
mod timestamped;

pub struct TaskLaunchResult {
    pub(crate) created_on: chrono::DateTime<chrono::Utc>,
    pub(crate) output: Sender<Timestamped<TaskOutput>>,
    pub(crate) handler: JoinHandle<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdatedNotification {
    #[serde(rename = "timestamp")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "body")]
    pub body: TaskUpdatedNotificationBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskUpdatedNotificationBody {
    #[serde(rename = "created")]
    Created {
        #[serde(rename = "arguments")]
        arguments: Option<TaskArguments>,
    },
    #[serde(rename = "launched")]
    Launched {
        #[serde(rename = "arguments")]
        arguments: Option<TaskArguments>,
    },
    #[serde(rename = "finished")]
    Finished {
        #[serde(rename = "arguments")]
        arguments: Option<TaskArguments>,
        #[serde(rename = "exit_code")]
        exit_code: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskOutput {
    #[serde(rename = "stdout")]
    Stdout(String),
    #[serde(rename = "stderr")]
    Stderr(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskEvent {
    #[serde(rename = "output")]
    Output(TaskOutput),
    #[serde(rename = "started")]
    Started,
    #[serde(rename = "finished")]
    Finished(i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedTaskEvent {
    #[serde(rename = "event")]
    pub event: TaskEvent,
    #[serde(rename = "timestamp")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct ActiveTask {
    pub created_on: chrono::DateTime<chrono::Utc>,
    pub launched_on: Option<chrono::DateTime<chrono::Utc>>,
    pub finished_on: Option<chrono::DateTime<chrono::Utc>>,
    pub script_key: Uuid,
    pub task_id: Uuid,
    pub exit_code: Option<i32>,
    pub events: Vec<TimedTaskEvent>,
    pub arguments: Option<TaskArguments>,
    pub events_tx: Sender<TimedTaskEvent>,
}

pub struct CreatedTask {
    pub created_on: chrono::DateTime<chrono::Utc>,
    pub script_key: Uuid,
    pub task_id: Uuid,
    pub events: Vec<TimedTaskEvent>,
    pub arguments: Option<TaskArguments>,
}

pub struct LaunchedTask {
    pub created_on: chrono::DateTime<chrono::Utc>,
    pub launched_on: chrono::DateTime<chrono::Utc>,
    pub script_key: Uuid,
    pub task_id: Uuid,
    pub events: Vec<TimedTaskEvent>,
    pub arguments: Option<TaskArguments>,
}

pub struct FinishedTask {
    pub created_on: chrono::DateTime<chrono::Utc>,
    pub launched_on: chrono::DateTime<chrono::Utc>,
    pub finished_on: chrono::DateTime<chrono::Utc>,
    pub script_key: Uuid,
    pub task_id: Uuid,
    pub exit_code: i32,
    pub events: Vec<TimedTaskEvent>,
    pub arguments: Option<TaskArguments>,
}
