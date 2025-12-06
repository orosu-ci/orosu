use crate::tasks::{ActiveTaskKey, TaskUpdatedNotification, TaskUpdatedNotificationBody};

impl TaskUpdatedNotification {
    pub fn now(key: ActiveTaskKey, body: TaskUpdatedNotificationBody) -> Self {
        Self {
            key,
            timestamp: chrono::Utc::now(),
            body,
        }
    }
}
