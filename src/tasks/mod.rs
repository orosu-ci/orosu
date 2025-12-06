use crate::model::TaskArguments;
use deadpool_diesel::sqlite::Pool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

mod active_task;
pub mod tasks;
mod timed_task_event;
mod task_update_notification;

pub struct Tasks {
    pub pool: Pool,
    pub active_tasks: Arc<Mutex<HashMap<ActiveTaskKey, Arc<RwLock<ActiveTask>>>>>,
    pub changes_tx: Sender<TaskUpdatedNotification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdatedNotification {
    #[serde(rename = "key")]
    pub key: ActiveTaskKey,
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ActiveTaskKey {
    pub script_key: Uuid,
    pub task_id: Uuid,
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

pub enum TaskLaunchResult {
    Created(Arc<RwLock<ActiveTask>>),
    Finished(FinishedTask),
}
