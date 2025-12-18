use crate::api::UserAgentHeader;
use axum::http::HeaderValue;

impl Default for UserAgentHeader {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl Into<HeaderValue> for UserAgentHeader {
    fn into(self) -> HeaderValue {
        format!("Orosu/{}", self.version).parse().unwrap()
    }
}

impl TryFrom<&HeaderValue> for UserAgentHeader {
    type Error = anyhow::Error;

    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        let string = value.to_str()?;
        let parts: Vec<&str> = string.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid user agent header: {string}");
        }
        if parts[0] != "Orosu" {
            anyhow::bail!("Invalid user agent header: {string}");
        }
        Ok(Self {
            version: parts[1].to_string(),
        })
    }
}
