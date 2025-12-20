use anyhow::Context;
use axum::http::Uri;
use std::ops::Deref;

pub struct ServerAddress(Uri);

impl ServerAddress {
    pub fn from_string(address: String) -> anyhow::Result<Self> {
        let mut parts = Uri::try_from(&address)
            .context("invalid server address format")?
            .into_parts();
        if parts.scheme.is_none() {
            parts.scheme = Some("wss".try_into()?);
        }
        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".try_into()?);
        }
        let uri = Uri::from_parts(parts)?;
        Ok(ServerAddress(uri))
    }
}

impl Deref for ServerAddress {
    type Target = Uri;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
