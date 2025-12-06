use diesel::sql_types::Binary;
use diesel::sqlite::Sqlite;
use diesel::{AsExpression, FromSqlRow, Queryable, QueryableByName, Selectable};

pub mod api;
mod database_script_status;
pub mod database_task;
mod database_uuid;
mod task_arguments;
mod worker_secret_key;

use crate::tasks::TimedTaskEvent;

use crate::schema::scripts;
use crate::schema::tasks;
use crate::schema::workers;

#[derive(Debug, Clone, Copy, FromSqlRow, AsExpression, Hash, Eq, PartialEq)]
#[diesel(sql_type = diesel::sql_types::Text)]
#[diesel(check_for_backend(Sqlite))]
pub struct DatabaseUuid(pub uuid::Uuid);

#[derive(Debug, Clone, Copy, FromSqlRow, AsExpression, Hash, Eq, PartialEq)]
#[diesel(sql_type = diesel::sql_types::Text)]
#[diesel(check_for_backend(Sqlite))]
pub enum DatabaseScriptStatus {
    Active,
    Disabled,
}

#[derive(Queryable, Debug, Selectable, QueryableByName)]
#[diesel(table_name = scripts)]
#[diesel(check_for_backend(Sqlite))]
pub struct DatabaseScript {
    pub path: String,
    pub key: DatabaseUuid,
    pub deleted_on: Option<chrono::NaiveDateTime>,
    pub updated_on: chrono::NaiveDateTime,
    pub created_on: chrono::NaiveDateTime,
    pub status: DatabaseScriptStatus,
}

#[derive(Debug, Clone, FromSqlRow, AsExpression)]
#[diesel(sql_type = Binary)]
#[diesel(check_for_backend(Sqlite))]
pub struct WorkerSecretKey(Vec<u8>);

#[derive(Queryable, Debug, Selectable, QueryableByName)]
#[diesel(table_name = workers)]
#[diesel(check_for_backend(Sqlite))]
pub struct DatabaseWorker {
    pub id: DatabaseUuid,
    pub name: String,
    pub created_on: chrono::NaiveDateTime,
    pub modified_on: chrono::NaiveDateTime,
    pub secret_key: WorkerSecretKey,
}

#[derive(Debug, FromSqlRow, AsExpression, Clone)]
#[diesel(sql_type = diesel::sql_types::Binary)]
#[diesel(check_for_backend(Sqlite))]
pub struct TaskArguments(pub Vec<String>);

#[derive(Debug, FromSqlRow, AsExpression, Clone)]
#[diesel(sql_type = diesel::sql_types::Binary)]
#[diesel(check_for_backend(Sqlite))]
pub struct DatabaseTimedTaskEvents(pub Vec<TimedTaskEvent>);

#[derive(Queryable, Debug, Selectable, QueryableByName)]
#[diesel(table_name = tasks)]
#[diesel(check_for_backend(Sqlite))]
pub struct DatabaseTask {
    pub id: DatabaseUuid,
    pub script_key: DatabaseUuid,
    pub exit_code: Option<i32>,
    pub output: DatabaseTimedTaskEvents,
    pub arguments: Option<TaskArguments>,
    pub created_on: chrono::NaiveDateTime,
    pub launched_on: Option<chrono::NaiveDateTime>,
    pub finished_on: Option<chrono::NaiveDateTime>,
}
