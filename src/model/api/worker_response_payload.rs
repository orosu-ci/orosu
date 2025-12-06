use crate::model::api::{NewWorkerResponsePayload, WorkerResponsePayload};
use crate::model::DatabaseWorker;

impl From<DatabaseWorker> for WorkerResponsePayload {
    fn from(value: DatabaseWorker) -> Self {
        Self {
            id: value.id.0,
            name: value.name,
            created_on: value.created_on,
            modified_on: value.modified_on,
        }
    }
}

impl From<DatabaseWorker> for NewWorkerResponsePayload {
    fn from(value: DatabaseWorker) -> Self {
        Self {
            id: value.id.0,
            name: value.name,
            created_on: value.created_on,
            modified_on: value.modified_on,
            secret_key: value.secret_key,
        }
    }
}
