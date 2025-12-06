use crate::tasks::{TaskEvent, TimedTaskEvent};

impl TimedTaskEvent {
    pub fn now(event: TaskEvent) -> Self {
        Self {
            event,
            timestamp: chrono::Utc::now(),
        }
    }
}
