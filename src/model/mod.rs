pub mod api;
mod task_arguments;
mod worker_secret_key;

#[derive(Debug, Clone)]
pub struct WorkerSecretKey(Vec<u8>);

#[derive(Debug, Clone)]
pub struct TaskArguments(pub Vec<String>);
