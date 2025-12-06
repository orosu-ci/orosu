use crate::model::api::ScriptStatus;
use crate::model::DatabaseScriptStatus;
use anyhow::anyhow;
use diesel::deserialize;
use diesel::deserialize::FromSql;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::{Sqlite, SqliteValue};

impl Into<ScriptStatus> for DatabaseScriptStatus {
    fn into(self) -> ScriptStatus {
        match self {
            DatabaseScriptStatus::Active => ScriptStatus::Active,
            DatabaseScriptStatus::Disabled => ScriptStatus::Disabled,
        }
    }
}

impl FromSql<Text, Sqlite> for DatabaseScriptStatus {
    fn from_sql(bytes: SqliteValue) -> deserialize::Result<Self> {
        let string = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        match string.as_str() {
            "ACTIVE" => Ok(DatabaseScriptStatus::Active),
            "DISABLED" => Ok(DatabaseScriptStatus::Disabled),
            _ => Err(anyhow!("Unknown database script status: {}", string).into()),
        }
    }
}

impl ToSql<Text, Sqlite> for DatabaseScriptStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> diesel::serialize::Result {
        let value = match self {
            DatabaseScriptStatus::Active => "ACTIVE",
            DatabaseScriptStatus::Disabled => "DISABLED",
        };
        out.set_value(value.to_string());
        Ok(IsNull::No)
    }
}
