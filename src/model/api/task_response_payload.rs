use crate::model::api::TaskResponsePayload;
use crate::model::DatabaseTask;

impl From<DatabaseTask> for TaskResponsePayload {
    fn from(value: DatabaseTask) -> Self {
        Self {
            id: value.id.0,
            script_key: value.script_key.0,
            exit_code: value.exit_code,
            output: value.output.0,
            arguments: value.arguments.map(|arguments| arguments.0),
            created_on: value.created_on,
            launched_on: value.launched_on,
            finished_on: value.finished_on,
        }
    }
}
