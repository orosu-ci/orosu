pub mod client;
pub mod envelopes;

use crate::tasks::{TaskOutput, Timestamped};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct StartTaskRequest {
    #[serde(rename = "script")]
    pub script_name: String,
    #[serde(rename = "args")]
    pub arguments: Vec<String>,
    #[serde(rename = "run_id")]
    pub run_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TaskLaunchStatus {
    #[serde(rename = "launched")]
    Launched { started_on: DateTime<Utc> },
    #[serde(rename = "running")]
    Running {
        #[serde(rename = "started_on")]
        started_on: DateTime<Utc>,
        #[serde(rename = "output")]
        output: Vec<Timestamped<TaskOutput>>,
    },
    #[serde(rename = "finished")]
    Finished {
        #[serde(rename = "started_on")]
        started_on: DateTime<Utc>,
        #[serde(rename = "finished_on")]
        finished_on: DateTime<Utc>,
        #[serde(rename = "exit_code")]
        exit_code: i32,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartTaskResponse {
    #[serde(rename = "run_id")]
    run_id: Uuid,
    #[serde(rename = "status")]
    status: TaskLaunchStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerErrorResponse {
    #[serde(rename = "cannot_launch_script")]
    CannotLaunchScript,
    #[serde(rename = "script_not_found")]
    ScriptNotFound,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerTaskNotification<O, E> {
    #[serde(rename = "output")]
    Output(O),
    #[serde(rename = "exit_code")]
    ExitCode(E),
}
