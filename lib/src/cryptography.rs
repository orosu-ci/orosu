use anyhow::Context;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use ed25519_dalek::ed25519::signature::rand_core::OsRng;
use ed25519_dalek::SigningKey;

#[derive(
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct ClientKey {
    pub client_name: String,
    pub key: Vec<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub(crate) sub: String,
    pub(crate) exp: usize,
}

impl ClientKey {
    pub fn from_string(value: String) -> anyhow::Result<Self> {
        let key = STANDARD.decode(value).context("invalid key format")?;
        let key =
            rkyv::from_bytes::<Self, rkyv::rancor::Error>(&key).context("invalid key format")?;
        Ok(key)
    }
}

pub struct Keygen {
    public_key: Vec<u8>,
    private_key: ClientKey,
}

impl Keygen {
    pub fn new(name: String) -> Self {
        let key = SigningKey::generate(&mut OsRng);

        let private_key = key.to_bytes();
        let public_key = key.verifying_key().to_bytes().into();

        let private_key = ClientKey {
            client_name: name,
            key: private_key.into(),
        };

        Self {
            public_key,
            private_key,
        }
    }

    pub fn public_key(&self) -> &Vec<u8> {
        &self.public_key
    }

    pub fn private_key(&self) -> &ClientKey {
        &self.private_key
    }

    pub fn public_key_base64(&self) -> String {
        STANDARD.encode(&self.public_key)
    }

    pub fn private_key_base64(&self) -> String {
        let private_key = rkyv::to_bytes::<rkyv::rancor::Error>(&self.private_key).unwrap();
        STANDARD.encode(private_key)
    }
}
