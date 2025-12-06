use crate::model::api::{ScriptStatus, TaskResponsePayload};
use crate::model::TaskArguments;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub enum TaskMessage {
    #[serde(rename = "new_task")]
    NewTask {
        #[serde(rename = "task_id")]
        task_id: Uuid,
        #[serde(rename = "script_key")]
        script_key: Uuid,
        #[serde(rename = "arguments")]
        arguments: Option<TaskArguments>,
    },
}

#[derive(Debug, Serialize)]
pub struct ScriptDetailsEvent {
    #[serde(rename = "key")]
    pub key: Uuid,
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "created_on")]
    pub created_on: chrono::NaiveDateTime,
    #[serde(rename = "updated_on")]
    pub updated_on: chrono::NaiveDateTime,
    #[serde(rename = "status")]
    pub status: ScriptStatus,
    #[serde(rename = "tasks")]
    pub tasks: Vec<TaskResponsePayload>,
}

#[derive(Debug, Serialize)]
pub struct ScriptTaskUpdatedEvent {
    #[serde(rename = "timestamp")]
    pub timestamp: chrono::NaiveDateTime,
    #[serde(rename = "body")]
    pub body: TaskResponsePayload,
}
