use crate::model::WorkerSecretKey;
use base64::prelude::BASE64_STANDARD;
use base64::{DecodeError, Engine};
use diesel::deserialize::FromSql;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::Binary;
use diesel::sqlite::{Sqlite, SqliteValue};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::Deref;

impl From<Vec<u8>> for WorkerSecretKey {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<&[u8]> for WorkerSecretKey {
    fn from(value: &[u8]) -> Self {
        Self(value.to_vec())
    }
}

impl TryFrom<&str> for WorkerSecretKey {
    type Error = DecodeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = BASE64_STANDARD.decode(value)?;
        Ok(Self(bytes))
    }
}

impl TryFrom<String> for WorkerSecretKey {
    type Error = DecodeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let bytes = BASE64_STANDARD.decode(value)?;
        Ok(Self(bytes))
    }
}

impl Into<Vec<u8>> for WorkerSecretKey {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl Deref for WorkerSecretKey {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for WorkerSecretKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&BASE64_STANDARD.encode(&self.0))
    }
}

impl<'de> Deserialize<'de> for WorkerSecretKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = BASE64_STANDARD
            .decode(&s)
            .map_err(serde::de::Error::custom)?;
        Ok(Self(bytes))
    }
}

impl ToSql<Binary, Sqlite> for WorkerSecretKey {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> diesel::serialize::Result {
        out.set_value(self.0.clone());
        Ok(IsNull::No)
    }
}

impl FromSql<Binary, Sqlite> for WorkerSecretKey {
    fn from_sql(bytes: SqliteValue) -> diesel::deserialize::Result<Self> {
        let bytes = Vec::<u8>::from_sql(bytes)?;
        Ok(Self(bytes))
    }
}
