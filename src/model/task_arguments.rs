use crate::model::TaskArguments;
use diesel::deserialize;
use diesel::deserialize::FromSql;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::Binary;
use diesel::sqlite::{Sqlite, SqliteValue};
use serde::{Deserialize, Serialize, Serializer};
use std::ops::Deref;

impl From<Vec<String>> for TaskArguments {
    fn from(value: Vec<String>) -> Self {
        Self(value)
    }
}

impl Into<Vec<String>> for TaskArguments {
    fn into(self) -> Vec<String> {
        self.0
    }
}

impl Deref for TaskArguments {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for TaskArguments {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TaskArguments {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Vec::<String>::deserialize(deserializer).map(Self)
    }
}

impl FromSql<Binary, Sqlite> for TaskArguments {
    fn from_sql(bytes: SqliteValue) -> deserialize::Result<Self> {
        let bytes = <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(bytes)?;
        let value = serde_json::from_slice::<TaskArguments>(&bytes)?;
        Ok(value)
    }
}

impl ToSql<Binary, Sqlite> for TaskArguments {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> diesel::serialize::Result {
        let bytes = serde_json::to_vec(&self.0)?;
        out.set_value(bytes);
        Ok(IsNull::No)
    }
}
