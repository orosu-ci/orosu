use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timestamped<T> {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: T,
}

impl<T> Timestamped<T> {
    pub fn now(value: T) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            value,
        }
    }
}
