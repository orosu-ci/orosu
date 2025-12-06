use crate::model::api::{
    ErrorCode, NewWorkerRequestPayload, NewWorkerResponsePayload, ResponsePayload,
    WorkerResponsePayload,
};
use crate::model::{DatabaseUuid, DatabaseWorker, WorkerSecretKey};
use crate::server::handler::WorkersHandler;
use crate::server::{ServerState, UserAuthContext};
use axum::extract::{Path, State};
use axum::Json;
use diesel::dsl::{delete, insert_into};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;
use uuid::Uuid;

impl WorkersHandler {
    pub async fn list(
        _: UserAuthContext,
        State(server_state): State<Arc<ServerState>>,
    ) -> Result<ResponsePayload<Vec<WorkerResponsePayload>>, ErrorCode> {
        let pool = server_state.database_pool.clone();
        let connection = pool.get().await.map_err(|_| ErrorCode::Unknown)?;
        let workers = connection
            .interact(|conn| {
                use crate::schema::workers::dsl::*;
                workers
                    .select(DatabaseWorker::as_select())
                    .order(created_on.desc())
                    .load::<DatabaseWorker>(conn)
            })
            .await
            .map_err(|_| ErrorCode::Unknown)?
            .map_err(|_| ErrorCode::Unknown)?
            .into_iter()
            .map(|e| e.into())
            .collect();

        Ok(ResponsePayload::Success(workers))
    }

    pub async fn delete(
        _: UserAuthContext,
        State(server_state): State<Arc<ServerState>>,
        Path(worker_id): Path<Uuid>,
    ) -> Result<ResponsePayload<()>, ErrorCode> {
        let pool = server_state.database_pool.clone();
        let connection = pool.get().await.map_err(|_| ErrorCode::Unknown)?;
        connection
            .interact(move |conn| {
                use crate::schema::workers::dsl::*;
                delete(workers)
                    .filter(id.eq::<DatabaseUuid>(worker_id.into()))
                    .execute(conn)
            })
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete worker: {:?}", e);
                ErrorCode::Unknown
            })?
            .map_err(|e| {
                tracing::error!("Failed to delete worker: {:?}", e);
                ErrorCode::Unknown
            })?;
        Ok(ResponsePayload::Success(()))
    }

    pub async fn create(
        _: UserAuthContext,
        State(server_state): State<Arc<ServerState>>,
        Json(payload): Json<NewWorkerRequestPayload>,
    ) -> Result<ResponsePayload<NewWorkerResponsePayload>, ErrorCode> {
        let pool = server_state.database_pool.clone();
        let connection = pool.get().await.map_err(|_| ErrorCode::Unknown)?;
        let worker_id = payload.id;
        let secret_key_bytes = rand::random::<[u8; 32]>();
        let secret_key_value = WorkerSecretKey::from(secret_key_bytes.to_vec());
        let new_worker = connection
            .interact(move |conn| {
                use crate::schema::workers::dsl::*;
                insert_into(workers)
                    .values((
                        id.eq::<DatabaseUuid>(worker_id.into()),
                        name.eq(payload.name),
                        secret_key.eq(secret_key_value),
                    ))
                    .execute(conn)?;
                workers
                    .select(DatabaseWorker::as_select())
                    .filter(id.eq::<DatabaseUuid>(worker_id.into()))
                    .first::<DatabaseWorker>(conn)
            })
            .await
            .map_err(|e| {
                tracing::error!("Failed to create worker: {:?}", e);
                ErrorCode::Unknown
            })?
            .map_err(|e| {
                tracing::error!("Failed to create worker: {:?}", e);
                ErrorCode::Unknown
            })?;
        let response = new_worker.into();
        Ok(ResponsePayload::Success(response))
    }
}
