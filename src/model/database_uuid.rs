use crate::model::DatabaseUuid;
use diesel::deserialize;
use diesel::deserialize::FromSql;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::{Sqlite, SqliteValue};
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

impl Deref for DatabaseUuid {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<Uuid> for DatabaseUuid {
    fn into(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for DatabaseUuid {
    fn from(uuid: Uuid) -> Self {
        DatabaseUuid(uuid)
    }
}

impl FromSql<Text, Sqlite> for DatabaseUuid {
    fn from_sql(bytes: SqliteValue) -> deserialize::Result<Self> {
        let string = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        let uuid = Uuid::from_str(&string)?;
        Ok(DatabaseUuid(uuid))
    }
}

impl ToSql<Text, Sqlite> for DatabaseUuid {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> diesel::serialize::Result {
        out.set_value(self.0.to_string());
        Ok(IsNull::No)
    }
}
