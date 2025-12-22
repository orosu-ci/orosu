pub mod client;
pub mod envelopes;
pub mod file_chunk;
mod user_agent_header;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StartTaskRequest {
    #[serde(rename = "script")]
    pub script_name: String,
    #[serde(rename = "args")]
    pub arguments: Vec<String>,
    #[serde(rename = "file")]
    pub file: Option<FileAttachment>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileAttachment {
    #[serde(rename = "hash")]
    pub hash: Vec<u8>,
    #[serde(rename = "size")]
    pub size: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TaskLaunchStatus {
    #[serde(rename = "awaiting_files")]
    AwaitingFiles { offset: usize },
    #[serde(rename = "launched")]
    Launched { started_on: DateTime<Utc> },
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

pub struct UserAgentHeader {
    pub version: String,
}
