use crate::scripts::{Configuration, ScriptsWatcher};
use chrono::NaiveDateTime;
use deadpool_diesel::sqlite::Pool;
use diesel::{update, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use notify::event::{CreateKind, ModifyKind, RemoveKind};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::fs::metadata;
use std::path::PathBuf;
use std::sync::mpsc;

#[derive(Debug)]
enum ScriptWorkerTask {
    Init(PathBuf),
    HandleEvent(Event),
}

#[derive(Debug)]
enum TaskResult {
    Directory(Vec<ScriptStatus<PathBuf>>),
    SingleFile(ScriptStatus<PathBuf>),
    Nothing,
}

#[derive(Debug)]
enum ScriptStatus<T> {
    Active(T),
    Skip(T),
}

impl<T> ScriptStatus<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, op: F) -> ScriptStatus<U> {
        match self {
            ScriptStatus::Active(value) => ScriptStatus::Active(op(value)),
            ScriptStatus::Skip(value) => ScriptStatus::Skip(op(value)),
        }
    }
}

impl ScriptsWatcher {
    pub fn new(configuration: Configuration, database: Pool) -> Self {
        Self {
            directory: configuration.scripts_directory,
            database,
        }
    }

    pub async fn watch(&self) -> anyhow::Result<()> {
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

        let mut watcher = notify::recommended_watcher(tx)?;

        watcher.watch(self.directory.as_path(), RecursiveMode::NonRecursive)?;

        handle_task(
            ScriptWorkerTask::Init(self.directory.clone()),
            self.database.clone(),
        )
        .await?;

        for res in rx {
            match res {
                Ok(event) => {
                    handle_task(ScriptWorkerTask::HandleEvent(event), self.database.clone()).await?
                }
                Err(e) => tracing::error!("watch error: {:?}", e),
            }
        }

        Ok(())
    }
}

async fn handle_task(task: ScriptWorkerTask, database: Pool) -> anyhow::Result<()> {
    let task_result = match &task {
        ScriptWorkerTask::Init(p) => {
            let mut results = vec![];
            let files = std::fs::read_dir(p)?;
            for file in files {
                let file = file?;
                let p = &file.path().canonicalize()?;
                let metadata = &file.metadata()?;
                if metadata.is_dir() {
                    results.push(ScriptStatus::Skip(p.clone()));
                    continue;
                }
                results.push(ScriptStatus::Active(p.clone()));
            }
            TaskResult::Directory(results)
        }
        ScriptWorkerTask::HandleEvent(event) => match event.kind {
            EventKind::Create(create) => match create {
                CreateKind::File => TaskResult::SingleFile(ScriptStatus::Active(
                    event.paths.first().unwrap().clone(),
                )),
                _ => TaskResult::Nothing,
            },
            EventKind::Modify(modify) => match modify {
                ModifyKind::Name(_) => {
                    let p = event.paths.first().unwrap().clone();
                    let file_exists = p.exists();
                    match file_exists {
                        true => {
                            let metadata = metadata(&p)?;
                            match metadata.is_dir() {
                                true => TaskResult::Nothing,
                                false => TaskResult::SingleFile(ScriptStatus::Active(p)),
                            }
                        }
                        false => TaskResult::SingleFile(ScriptStatus::Skip(p)),
                    }
                }
                _ => TaskResult::Nothing,
            },
            EventKind::Remove(remove) => match remove {
                RemoveKind::File => {
                    TaskResult::SingleFile(ScriptStatus::Skip(event.paths.first().unwrap().clone()))
                }
                _ => TaskResult::Nothing,
            },
            _ => TaskResult::Nothing,
        },
    };

    use crate::schema::scripts::dsl::*;
    let connection = database.get().await?;
    connection
        .interact(|conn| {
            let existing_paths = scripts
                .select(path)
                .load::<String>(conn)
                .map_err(|e| anyhow::anyhow!(e))?;

            match task_result {
                TaskResult::Directory(results) => {
                    for result in results {
                        process_script(result, &existing_paths, conn)?;
                    }
                }
                TaskResult::SingleFile(result) => process_script(result, &existing_paths, conn)?,
                TaskResult::Nothing => { /* do nothing */ }
            }
            anyhow::Ok(())
        })
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))??;
    Ok(())
}

fn process_script(
    script: ScriptStatus<PathBuf>,
    existing_paths: &Vec<String>,
    connection: &mut SqliteConnection,
) -> anyhow::Result<()> {
    use crate::schema::scripts::dsl::*;
    let result = script.map(|p| p.to_str().unwrap().to_string());
    match result {
        ScriptStatus::Active(p) => {
            if existing_paths.contains(&p) {
                update(scripts)
                    .filter(path.eq(&p))
                    .set((
                        deleted_on.eq::<Option<NaiveDateTime>>(None),
                        updated_on.eq(diesel::dsl::now),
                    ))
                    .execute(connection)
                    .inspect_err(|e| {
                        tracing::error!("Cannot delete script: {:?}", e);
                    })?;
            } else {
                diesel::insert_into(scripts)
                    .values((path.eq(&p), key.eq(uuid::Uuid::new_v4().to_string())))
                    .execute(connection)?;
            }
        }
        ScriptStatus::Skip(p) => {
            if existing_paths.contains(&p) {
                update(scripts)
                    .filter(path.eq(&p))
                    .set((
                        deleted_on.eq(diesel::dsl::now),
                        updated_on.eq(diesel::dsl::now),
                    ))
                    .execute(connection)
                    .inspect_err(|e| {
                        tracing::error!("Cannot delete script: {:?}", e);
                    })?;
            }
        }
    };
    Ok(())
}
