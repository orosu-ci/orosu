use crate::model::TaskArguments;
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
