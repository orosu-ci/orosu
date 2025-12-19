use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Script {
    #[serde(rename = "name")]
    pub(crate) name: String,
    #[serde(rename = "command")]
    pub(crate) command: Vec<String>,
}
