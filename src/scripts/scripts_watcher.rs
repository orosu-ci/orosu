use notify::event::{CreateKind, ModifyKind, RemoveKind};
use notify::{Event, EventKind};
use std::fs::metadata;
use std::path::PathBuf;

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

async fn handle_task(task: ScriptWorkerTask) -> anyhow::Result<()> {
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

    Ok(())
}
