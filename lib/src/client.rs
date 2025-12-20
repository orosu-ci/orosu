use crate::script::Script;
use cidr::IpCidr;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Client {
    #[serde(rename = "name")]
    pub(crate) name: String,
    #[serde(rename = "secret_file")]
    pub(crate) secret_file: PathBuf,
    #[serde(rename = "whitelisted_ips")]
    pub(crate) whitelisted_ips: Option<Vec<IpCidr>>,
    #[serde(rename = "blacklisted_ips")]
    pub(crate) blacklisted_ips: Option<Vec<IpCidr>>,
    #[serde(rename = "scripts")]
    pub(crate) scripts: Vec<Script>,
}
