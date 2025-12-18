use crate::model::TaskArguments;
use crate::tasks::task::Task;
pub(crate) use crate::tasks::timestamped::Timestamped;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use uuid::Uuid;

mod task;
mod task_update_notification;
pub mod tasks;
mod timed_task_event;
mod timestamped;

pub struct Tasks {
    pub active_tasks: Arc<Mutex<HashMap<ActiveTaskKey, Arc<RwLock<Task>>>>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ActiveTaskKey {
    pub script_name: String,
    pub client_name: String,
    pub run_id: Uuid,
}

pub enum TaskLaunchResult {
    Created {
        created_on: chrono::DateTime<chrono::Utc>,
        output_tx: Sender<Timestamped<TaskOutput>>,
        handler: JoinHandle<i32>,
    },
    Joined {
        created_on: chrono::DateTime<chrono::Utc>,
        output: Vec<Timestamped<TaskOutput>>,
        output_tx: Sender<Timestamped<TaskOutput>>,
        handler: JoinHandle<i32>,
    },
    Finished {
        created_on: chrono::DateTime<chrono::Utc>,
        finished_on: chrono::DateTime<chrono::Utc>,
        exit_code: i32,
        output: Vec<Timestamped<TaskOutput>>,
    },
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
