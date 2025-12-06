use crate::model::api::ScriptResponsePayload;
use crate::model::DatabaseScript;

impl From<DatabaseScript> for ScriptResponsePayload {
    fn from(value: DatabaseScript) -> Self {
        Self {
            path: value.path,
            key: value.key.into(),
            status: value.status.into(),
            created_on: value.created_on,
            updated_on: value.updated_on,
            deleted_on: value.deleted_on,
        }
    }
}
