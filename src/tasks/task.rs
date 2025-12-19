use crate::script::Script;
use crate::tasks::{TaskLaunchResult, TaskOutput, Timestamped};
use std::collections::VecDeque;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{broadcast, watch};

pub struct Task {
    created_on: chrono::DateTime<chrono::Utc>,
    script: Script,
    exit_code_tx: watch::Sender<Option<Timestamped<i32>>>,
    output_tx: broadcast::Sender<Timestamped<TaskOutput>>,
}

impl Task {
    pub fn create(script: Script) -> Self {
        let created_on = chrono::Utc::now();
        let (exit_code_tx, _) = watch::channel(None);
        let (output_tx, _) = broadcast::channel(32);
        Self {
            created_on,
            script,
            exit_code_tx,
            output_tx,
        }
    }

    fn append_stdout<T: Into<String>>(tx: broadcast::Sender<Timestamped<TaskOutput>>, line: T) {
        Self::append_output(tx, TaskOutput::Stdout(line.into()));
    }

    fn append_stderr<T: Into<String>>(tx: broadcast::Sender<Timestamped<TaskOutput>>, line: T) {
        Self::append_output(tx, TaskOutput::Stderr(line.into()));
    }

    fn append_output(tx: broadcast::Sender<Timestamped<TaskOutput>>, output: TaskOutput) {
        let event = Timestamped::now(output);
        if let Err(e) = tx.send(event) {
            tracing::error!("Failed to send task output event: {e}");
        }
    }

    fn set_exit_code(tx: watch::Sender<Option<Timestamped<i32>>>, exit_code: i32) {
        let event = Timestamped::now(exit_code);
        if let Err(e) = tx.send(Some(event)) {
            tracing::error!("Failed to send task exit code: {e}");
        }
    }

    pub async fn run(&self, arguments: Vec<String>) -> anyhow::Result<TaskLaunchResult> {
        let created_on = self.created_on;
        let output_tx = self.output_tx.clone();
        let script = self.script.clone();

        let mut command_with_arguments = VecDeque::from(script.command);
        let command = command_with_arguments.pop_front();
        let Some(command) = command else {
            anyhow::bail!("Script command is empty");
        };
        let mut command = tokio::process::Command::new(command);
        if !command_with_arguments.is_empty() {
            command.args(command_with_arguments);
        }
        if !arguments.is_empty() {
            command.args(arguments);
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(username) = script.run_as {
                unsafe {
                    command.pre_exec(move || {
                        let user = users::get_user_by_name(&username).ok_or_else(|| {
                            std::io::Error::new(std::io::ErrorKind::NotFound, "User not found")
                        })?;

                        nix::unistd::setgid(nix::unistd::Gid::from_raw(user.primary_group_id()))
                            .map_err(|e| {
                                std::io::Error::new(std::io::ErrorKind::PermissionDenied, e)
                            })?;
                        nix::unistd::setuid(nix::unistd::Uid::from_raw(user.uid()))
                            .map_err(|e| {
                                std::io::Error::new(std::io::ErrorKind::PermissionDenied, e)
                            })?;
                        Ok(())
                    });
                }
            };
        }

        tracing::info!("Running script: {:?}", command);

        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
                Self::append_stderr(output_tx, format!("Failed to start script: {e}"));
                Self::set_exit_code(self.exit_code_tx.clone(), 1);
                return Err(e.into());
            }
        };

        let handler_output_tx = output_tx.clone();
        let result_output_tx = output_tx.clone();
        let handler_exit_code_tx = self.exit_code_tx.clone();

        let handler = tokio::spawn(async move {
            let stdout = child.stdout.take().unwrap();
            let stderr = child.stderr.take().unwrap();

            let stdout_output_tx = handler_output_tx.clone();
            let stderr_output_tx = handler_output_tx.clone();

            let stdout_task = tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    Self::append_stdout(stdout_output_tx.clone(), line);
                }
            });

            let stderr_task = tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    Self::append_stderr(stderr_output_tx.clone(), line);
                }
            });

            let _ = tokio::join!(stdout_task, stderr_task);

            let exit_code = match child.wait().await {
                Ok(status) => match status.code() {
                    None => {
                        Self::append_stderr(output_tx, "Command terminated by signal");
                        -1
                    }
                    Some(code) => code,
                },
                Err(e) => {
                    Self::append_stderr(output_tx, format!("Command failed: {e}"));
                    -1
                }
            };
            Self::set_exit_code(handler_exit_code_tx, exit_code);
            exit_code
        });
        Ok(TaskLaunchResult {
            created_on,
            output: result_output_tx,
            handler,
        })
    }
}
