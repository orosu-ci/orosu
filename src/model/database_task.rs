use crate::model::{DatabaseTask, DatabaseTimedTaskEvents};
use crate::tasks::{FinishedTask, TimedTaskEvent};
use diesel::deserialize;
use diesel::deserialize::FromSql;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::Binary;
use diesel::sqlite::{Sqlite, SqliteValue};
use std::ops::Deref;

impl Deref for DatabaseTimedTaskEvents {
    type Target = Vec<TimedTaskEvent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<TimedTaskEvent>> for DatabaseTimedTaskEvents {
    fn from(value: Vec<TimedTaskEvent>) -> Self {
        Self(value)
    }
}

impl FromSql<Binary, Sqlite> for DatabaseTimedTaskEvents {
    fn from_sql(bytes: SqliteValue) -> deserialize::Result<Self> {
        let bytes = <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(bytes)?;
        let value = serde_json::from_slice::<Vec<TimedTaskEvent>>(&bytes)?;
        Ok(DatabaseTimedTaskEvents(value))
    }
}

impl ToSql<Binary, Sqlite> for DatabaseTimedTaskEvents {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> diesel::serialize::Result {
        let bytes = serde_json::to_vec(&self.0)?;
        out.set_value(bytes);
        Ok(IsNull::No)
    }
}

impl Into<FinishedTask> for DatabaseTask {
    fn into(self) -> FinishedTask {
        FinishedTask {
            created_on: self.created_on.and_utc(),
            launched_on: self.launched_on.unwrap().and_utc(),
            finished_on: self.finished_on.unwrap().and_utc(),
            script_key: self.script_key.0,
            task_id: self.id.0,
            exit_code: self.exit_code.unwrap(),
            events: self.output.0,
            arguments: self.arguments,
        }
    }
}
