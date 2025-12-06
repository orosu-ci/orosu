use crate::model::api::{ErrorCode, ResponsePayload, ScriptResponsePayload};
use crate::model::DatabaseScript;
use crate::server::handler::ScriptsHandler;
use crate::server::{ServerState, UserAuthContext};
use axum::extract::State;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;

impl ScriptsHandler {
    pub async fn list(
        _: UserAuthContext,
        State(server_state): State<Arc<ServerState>>,
    ) -> Result<ResponsePayload<Vec<ScriptResponsePayload>>, ErrorCode> {
        let connection = server_state
            .database_pool
            .get()
            .await
            .inspect_err(|e| tracing::error!("Cannot get database connection: {:?}", e))
            .map_err(|_| ErrorCode::Unknown)?;
        use crate::schema::scripts::dsl::*;
        let s = connection
            .interact(|conn| {
                scripts
                    .select(DatabaseScript::as_select())
                    .load::<DatabaseScript>(conn)
                    .inspect_err(|e| tracing::error!("Cannot query scripts: {:?}", e))
                    .map_err(|_| ErrorCode::Unknown)
                    .map(|v| v.into_iter().map(Into::into).collect())
            })
            .await
            .inspect_err(|e| tracing::error!("Cannot interact with database: {:?}", e))
            .map_err(|_| ErrorCode::Unknown)??;
        Ok(ResponsePayload::Success(s))
    }
}
