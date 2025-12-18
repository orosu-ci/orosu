use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Script {
    #[serde(rename = "name")]
    pub(crate) name: String,
    #[serde(rename = "working_directory")]
    pub(crate) working_directory: PathBuf,
    #[serde(rename = "command")]
    pub(crate) command: Vec<String>,
}
