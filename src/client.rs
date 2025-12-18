use crate::script::Script;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Client {
    #[serde(rename = "name")]
    pub(crate) name: String,
    #[serde(rename = "secret")]
    pub(crate) secret: String,
    #[serde(rename = "whitelisted_ips")]
    pub(crate) whitelisted_ips: Option<Vec<IpAddr>>,
    #[serde(rename = "blacklisted_ips")]
    pub(crate) blacklisted_ips: Option<Vec<IpAddr>>,
    #[serde(rename = "scripts")]
    pub(crate) scripts: Vec<Script>,
}
