use crate::api::{ServerErrorResponse, ServerTaskNotification, StartTaskRequest, TaskLaunchStatus};
use crate::tasks::{TaskOutput, Timestamped};
use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestEnvelope<T> {
    #[serde(rename = "id")]
    pub id: Uuid,
    #[serde(rename = "body")]
    pub body: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseEnvelope<T, E> {
    #[serde(rename = "success")]
    Success {
        #[serde(rename = "id")]
        id: Uuid,
        #[serde(rename = "body")]
        body: T,
    },
    #[serde(rename = "failure")]
    Failure {
        #[serde(rename = "id")]
        id: Uuid,
        #[serde(rename = "error")]
        error: E,
    },
}

impl<T> Into<Bytes> for RequestEnvelope<T>
where
    T: Serialize,
{
    fn into(self) -> Bytes {
        serde_json::to_vec(&self).unwrap().into()
    }
}

impl<T, E> Into<Bytes> for ResponseEnvelope<T, E>
where
    T: Serialize,
    E: Serialize,
{
    fn into(self) -> Bytes {
        serde_json::to_vec(&self).unwrap().into()
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

pub type TaskLaunchStatusResponseEnvelope = ResponseEnvelope<TaskLaunchStatus, ServerErrorResponse>;
pub type TaskEventResponseEnvelope =
    ResponseEnvelope<ServerTaskNotification<Timestamped<TaskOutput>, i32>, ServerErrorResponse>;
pub type TaskLaunchRequestEnvelope = RequestEnvelope<StartTaskRequest>;
