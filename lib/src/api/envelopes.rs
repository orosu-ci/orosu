use crate::api::file_chunk::FileChunk;
use crate::api::{ServerErrorResponse, ServerTaskNotification, StartTaskRequest, TaskLaunchStatus};
use crate::tasks::{TaskOutput, Timestamped};
use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestEnvelope<T> {
    #[serde(rename = "body")]
    pub body: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseEnvelope<T, E> {
    #[serde(rename = "success")]
    Success {
        #[serde(rename = "body")]
        body: T,
    },
    #[serde(rename = "failure")]
    Failure {
        #[serde(rename = "error")]
        error: E,
    },
}

impl<T> From<RequestEnvelope<T>> for Bytes
where
    T: Serialize,
{
    fn from(value: RequestEnvelope<T>) -> Self {
        serde_json::to_vec(&value).unwrap().into()
    }
}

impl<T, E> From<ResponseEnvelope<T, E>> for Bytes
where
    T: Serialize,
    E: Serialize,
{
    fn from(value: ResponseEnvelope<T, E>) -> Self {
        serde_json::to_vec(&value).unwrap().into()
    }
}

impl<T> From<Bytes> for RequestEnvelope<T>
where
    T: DeserializeOwned,
{
    fn from(value: Bytes) -> Self {
        serde_json::from_slice(&value).unwrap()
    }
}

impl<T, E> From<Bytes> for ResponseEnvelope<T, E>
where
    T: DeserializeOwned,
    E: DeserializeOwned,
{
    fn from(value: Bytes) -> Self {
        serde_json::from_slice(&value).unwrap()
    }
}

pub type FileChunkRequestEnvelope = RequestEnvelope<FileChunk>;
pub type TaskLaunchStatusResponseEnvelope = ResponseEnvelope<TaskLaunchStatus, ServerErrorResponse>;
pub type TaskEventResponseEnvelope =
    ResponseEnvelope<ServerTaskNotification<Timestamped<TaskOutput>, i32>, ServerErrorResponse>;
pub type TaskLaunchRequestEnvelope = RequestEnvelope<StartTaskRequest>;
