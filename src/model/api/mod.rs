mod script_response_payload;
mod task_response_payload;
mod worker_response_payload;
pub mod ws;

use crate::model::WorkerSecretKey;
use crate::tasks::TimedTaskEvent;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

pub enum ResponsePayload<T> {
    Success(T),
    Failure(ErrorCode),
}

#[derive(Debug, Serialize, Copy, Clone)]
pub enum ErrorCode {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "invalid_credentials")]
    InvalidCredentials,
    #[serde(rename = "unauthorized")]
    Unauthorized,
    #[serde(rename = "forbidden")]
    Forbidden,
}

impl IntoResponse for ErrorCode {
    fn into_response(self) -> Response {
        ResponsePayload::<()>::Failure(self).into_response()
    }
}

impl Into<StatusCode> for ErrorCode {
    fn into(self) -> StatusCode {
        match self {
            ErrorCode::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InvalidCredentials => StatusCode::BAD_REQUEST,
            ErrorCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorCode::Forbidden => StatusCode::FORBIDDEN,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum ApiResponseStatus {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failure")]
    Failure,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    #[serde(rename = "status")]
    status: ApiResponseStatus,
    #[serde(rename = "payload", skip_serializing_if = "Value::is_null")]
    payload: Value,
}

#[derive(Debug, Serialize)]
pub struct ApiResponseErrorPayload {
    #[serde(rename = "code")]
    pub code: ErrorCode,
}

impl<T> IntoResponse for ResponsePayload<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match self {
            ResponsePayload::Success(body) => (
                StatusCode::OK,
                Json(ApiResponse {
                    status: ApiResponseStatus::Success,
                    payload: serde_json::to_value(body).unwrap(),
                }),
            )
                .into_response(),
            ResponsePayload::Failure(code) => (
                Into::<StatusCode>::into(code),
                Json(ApiResponse {
                    status: ApiResponseStatus::Failure,
                    payload: serde_json::to_value(ApiResponseErrorPayload { code }).unwrap(),
                }),
            )
                .into_response(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthStatusResponsePayload {
    #[serde(rename = "is_authenticated")]
    pub is_authenticated: bool,
}

#[derive(Debug, Deserialize)]
pub struct AuthSignInRequestPayload {
    #[serde(rename = "username")]
    pub username: String,
    #[serde(rename = "password")]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthSignInResponsePayload {
    #[serde(rename = "success")]
    pub success: bool,
}

#[derive(Debug, Serialize)]
pub struct AuthSignOutResponsePayload {
    #[serde(rename = "success")]
    pub success: bool,
}

#[derive(Debug, Serialize)]
pub enum ScriptStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "disabled")]
    Disabled,
}

#[derive(Debug, Serialize)]
pub struct ScriptResponsePayload {
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "key")]
    pub key: Uuid,
    #[serde(rename = "status")]
    pub status: ScriptStatus,
    #[serde(rename = "created_on")]
    pub created_on: chrono::NaiveDateTime,
    #[serde(rename = "updated_on")]
    pub updated_on: chrono::NaiveDateTime,
    #[serde(rename = "deleted_on")]
    pub deleted_on: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct NewTaskRequestPayload {
    #[serde(rename = "script")]
    pub script: Uuid,
    #[serde(rename = "arguments")]
    pub arguments: HashMap<String, String>,
    #[serde(rename = "task_id")]
    pub task_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct NewWorkerRequestPayload {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "id")]
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct WorkerResponsePayload {
    #[serde(rename = "id")]
    pub id: Uuid,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "created_on")]
    pub created_on: chrono::NaiveDateTime,
    #[serde(rename = "modified_on")]
    pub modified_on: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct NewWorkerResponsePayload {
    #[serde(rename = "id")]
    pub id: Uuid,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "created_on")]
    pub created_on: chrono::NaiveDateTime,
    #[serde(rename = "modified_on")]
    pub modified_on: chrono::NaiveDateTime,
    #[serde(rename = "secret_key")]
    pub secret_key: WorkerSecretKey,
}

#[derive(Debug, Serialize)]
pub struct TaskResponsePayload {
    #[serde(rename = "id")]
    pub id: Uuid,
    #[serde(rename = "script_key")]
    pub script_key: Uuid,
    #[serde(rename = "exit_code")]
    pub exit_code: Option<i32>,
    #[serde(rename = "output")]
    pub output: Vec<TimedTaskEvent>,
    #[serde(rename = "arguments")]
    pub arguments: Option<Vec<String>>,
    #[serde(rename = "created_on")]
    pub created_on: chrono::NaiveDateTime,
    #[serde(rename = "launched_on")]
    pub launched_on: Option<chrono::NaiveDateTime>,
    #[serde(rename = "finished_on")]
    pub finished_on: Option<chrono::NaiveDateTime>,
}
