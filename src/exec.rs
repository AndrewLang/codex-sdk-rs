use std::collections::HashMap;
use std::env;
use std::fmt;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Stdio;

use async_stream::try_stream;
use futures::Stream;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tokio_util::sync::CancellationToken;

use crate::error::CodexError;
use crate::thread_options::{ApprovalMode, ModelReasoningEffort, SandboxMode, WebSearchMode};

pub type CodexLineStream = Pin<Box<dyn Stream<Item = Result<String, CodexError>> + Send>>;

#[derive(Clone, Debug)]
pub struct CodexExec {
    executable_path: PathBuf,
    env_override: Option<HashMap<String, String>>,
    config_overrides: Option<Value>,
}

#[derive(Clone, Debug, Default)]
pub struct CodexExecArgs {
    pub input: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub thread_id: Option<String>,
    pub images: Option<Vec<String>>,
    pub model: Option<String>,
    pub sandbox_mode: Option<SandboxMode>,
    pub working_directory: Option<String>,
    pub additional_directories: Option<Vec<String>>,
    pub skip_git_repo_check: Option<bool>,
    pub output_schema_file: Option<PathBuf>,
    pub model_reasoning_effort: Option<ModelReasoningEffort>,
    pub cancel: Option<CancellationToken>,
    pub network_access_enabled: Option<bool>,
    pub web_search_mode: Option<WebSearchMode>,
    pub web_search_enabled: Option<bool>,
    pub approval_policy: Option<ApprovalMode>,
}

impl fmt::Display for CodexExecArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let api_key = if self.api_key.is_some() {
            "Some([redacted])"
        } else {
            "None"
        };
        let cancel = if self.cancel.is_some() {
            "Some(<token>)"
        } else {
            "None"
        };

        write!(
            f,
            "CodexExecArgs {{ input_len: {}, base_url: {:?}, api_key: {}, thread_id: {:?}, images: {}, model: {:?}, sandbox_mode: {:?}, working_directory: {:?}, additional_directories: {:?}, skip_git_repo_check: {:?}, output_schema_file: {:?}, model_reasoning_effort: {:?}, cancel: {}, network_access_enabled: {:?}, web_search_mode: {:?}, web_search_enabled: {:?}, approval_policy: {:?} }}",
            self.input.len(),
            self.base_url,
            api_key,
            self.thread_id,
            self.images.as_ref().map(|items| items.len()).unwrap_or(0),
            self.model,
            self.sandbox_mode,
            self.working_directory,
            self.additional_directories,
            self.skip_git_repo_check,
            self.output_schema_file,
            self.model_reasoning_effort,
            cancel,
            self.network_access_enabled,
            self.web_search_mode,
            self.web_search_enabled,
            self.approval_policy,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandSpec {
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

const INTERNAL_ORIGINATOR_ENV: &str = "CODEX_INTERNAL_ORIGINATOR_OVERRIDE";
const RUST_SDK_ORIGINATOR: &str = "codex_sdk_rs";

impl CodexExec {
    pub fn new(
        executable_path: Option<PathBuf>,
        env: Option<HashMap<String, String>>,
        config_overrides: Option<Value>,
    ) -> Result<Self, CodexError> {
        let executable_path = match executable_path {
            Some(path) => path,
            None => PathBuf::from("codex"),
        };

        Ok(Self {
            executable_path,
            env_override: env,
            config_overrides,
        })
    }

    #[doc(hidden)]
    pub fn build_command(&self, args: &CodexExecArgs) -> Result<CommandSpec, CodexError> {
        log::debug!("Building codex command");
        let mut command_args = vec!["exec".to_string(), "--experimental-json".to_string()];

        if let Some(config_overrides) = &self.config_overrides {
            let overrides = Self::serialize_config_overrides(config_overrides)?;
            log::debug!("Config override count: {}", overrides.len());
            for override_entry in overrides {
                command_args.push("--config".to_string());
                command_args.push(override_entry);
            }
        }

        if let Some(model) = &args.model {
            command_args.push("--model".to_string());
            command_args.push(model.clone());
        }

        if let Some(mode) = &args.sandbox_mode {
            command_args.push("--sandbox".to_string());
            command_args.push(mode.as_str().to_string());
        }

        if let Some(dir) = &args.working_directory {
            command_args.push("--cd".to_string());
            command_args.push(dir.clone());
        }

        if let Some(dirs) = &args.additional_directories {
            for dir in dirs {
                command_args.push("--add-dir".to_string());
                command_args.push(dir.clone());
            }
        }

        if args.skip_git_repo_check.unwrap_or(false) {
            command_args.push("--skip-git-repo-check".to_string());
        }

        if let Some(path) = &args.output_schema_file {
            command_args.push("--output-schema".to_string());
            command_args.push(path.to_string_lossy().to_string());
        }

        if let Some(effort) = &args.model_reasoning_effort {
            command_args.push("--config".to_string());
            command_args.push(format!("model_reasoning_effort=\"{}\"", effort.as_str()));
        }

        if let Some(network_access) = args.network_access_enabled {
            command_args.push("--config".to_string());
            command_args.push(format!(
                "sandbox_workspace_write.network_access={}",
                network_access
            ));
        }

        if let Some(mode) = &args.web_search_mode {
            command_args.push("--config".to_string());
            command_args.push(format!("web_search=\"{}\"", mode.as_str()));
        } else if let Some(web_search_enabled) = args.web_search_enabled {
            let value = if web_search_enabled {
                "live"
            } else {
                "disabled"
            };
            command_args.push("--config".to_string());
            command_args.push(format!("web_search=\"{}\"", value));
        }

        if let Some(policy) = &args.approval_policy {
            command_args.push("--config".to_string());
            command_args.push(format!("approval_policy=\"{}\"", policy.as_str()));
        }

        if let Some(thread_id) = &args.thread_id {
            command_args.push("resume".to_string());
            command_args.push(thread_id.clone());
        }

        if let Some(images) = &args.images {
            for image in images {
                command_args.push("--image".to_string());
                command_args.push(image.clone());
            }
        }

        let env = self.build_env(args);

        log::debug!("Command args count: {}", command_args.len());
        for arg in &command_args {
            log::debug!("\t Arg: {}", arg);
        }

        log::debug!("Environment variable count: {}", env.len());
        for (key, value) in &env {
            log::debug!("\t {}={}", key, value);
        }

        Ok(CommandSpec {
            args: command_args,
            env,
        })
    }

    fn build_env(&self, args: &CodexExecArgs) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();
        if let Some(override_env) = &self.env_override {
            env_vars.extend(override_env.clone());
            log::debug!("Using explicit environment override");
        } else {
            for (key, value) in env::vars() {
                env_vars.insert(key, value);
            }
            log::debug!("Using inherited environment");
        }

        env_vars
            .entry(INTERNAL_ORIGINATOR_ENV.to_string())
            .or_insert_with(|| RUST_SDK_ORIGINATOR.to_string());

        env_vars
            .entry("CI".to_string())
            .or_insert_with(|| "true".to_string());
        env_vars
            .entry("TERM".to_string())
            .or_insert_with(|| "xterm".to_string());

        if let Some(base_url) = &args.base_url {
            env_vars.insert("OPENAI_BASE_URL".to_string(), base_url.clone());
            log::debug!("OPENAI_BASE_URL set");
        }
        if let Some(api_key) = &args.api_key {
            env_vars.insert("CODEX_API_KEY".to_string(), api_key.clone());
            log::debug!("CODEX_API_KEY set");
        }

        env_vars
    }

    pub fn run(&self, args: CodexExecArgs) -> Result<CodexLineStream, CodexError> {
        let command = self.build_command(&args)?;
        let executable_path = self.executable_path.clone();
        let cancel = args.cancel.clone();
        let input = args.input.clone();

        log::debug!(
            "Running codex with executable: {}",
            executable_path.display()
        );

        let stream = try_stream! {
            if let Some(token) = &cancel {
                if token.is_cancelled() {
                    log::debug!("Execution aborted before spawn");
                    Err(CodexError::Aborted)?;
                }
            }

            let mut child = Self::spawn_codex(&executable_path, &[], &command.args, &command.env)?;

            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                stdin.write_all(input.as_bytes()).await?;
                stdin.shutdown().await?;
            }

            let stdout = child.stdout.take().ok_or(CodexError::MissingChildStream("stdout"))?;
            let stderr = child.stderr.take().ok_or(CodexError::MissingChildStream("stderr"))?;
            let stderr_task = Self::capture_stderr(stderr);

            let mut lines = BufReader::new(stdout).lines();
            let mut poll = interval(Duration::from_millis(250));
            poll.set_missed_tick_behavior(MissedTickBehavior::Delay);
            let mut exit_status = None;

            log::debug!("Codex process spawned, waiting for output...");

            enum LoopAction {
                Line(Option<String>),
                Tick,
            }

            loop {
                let action = if exit_status.is_some() {
                    LoopAction::Line(lines.next_line().await?)
                } else {
                    let result: Result<LoopAction, CodexError> = tokio::select! {
                        _ = async {
                            if let Some(token) = &cancel {
                                token.cancelled().await;
                            } else {
                                std::future::pending::<()>().await;
                            }
                        } => {
                            child.kill().await.ok();
                            log::debug!("Execution aborted during stream");
                            Err(CodexError::Aborted)
                        }
                        line = lines.next_line() => line.map(LoopAction::Line).map_err(CodexError::from),
                        _ = poll.tick() => Ok(LoopAction::Tick),
                    };
                    result?
                };

                match action {
                    LoopAction::Line(next_line) => {
                        log::debug!("Read line: {:?}", next_line);
                        match next_line {
                            Some(line) => yield line,
                            None => break,
                        }
                    }
                    LoopAction::Tick => {
                        if exit_status.is_none() {
                            exit_status = child.try_wait().map_err(CodexError::from)?;
                        }
                    }
                }
            }

            log::debug!("Codex process completed, waiting for exit status...");

            let status = match exit_status {
                Some(status) => status,
                None => child.wait().await?,
            };
            let stderr_buffer = stderr_task.await.unwrap_or_default();
            if !status.success() {
                let detail = status
                    .code()
                    .map(|code| format!("code {}", code))
                    .unwrap_or_else(|| "signal".to_string());
                let stderr_text = String::from_utf8_lossy(&stderr_buffer).to_string();
                Err(CodexError::ExecFailed(detail, stderr_text))?;
            }
        };

        Ok(Box::pin(stream))
    }

    fn spawn_codex(
        exe: &Path,
        pre_args: &[String],
        args: &[String],
        envs: &HashMap<String, String>,
    ) -> Result<Child, CodexError> {
        #[cfg(target_os = "windows")]
        let mut command = {
            let mut cmd = Command::new("cmd");
            cmd.arg("/C").arg(exe);
            cmd
        };

        #[cfg(not(target_os = "windows"))]
        let mut command = Command::new(exe);

        command
            .args(pre_args)
            .args(args)
            .envs(envs)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(CodexError::from)
    }

    fn capture_stderr(stderr: tokio::process::ChildStderr) -> JoinHandle<Vec<u8>> {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut buffer = Vec::new();
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 {
                    break;
                }
                log::warn!("Stderr: {}", line.trim());
                buffer.extend_from_slice(line.as_bytes());
                line.clear();
            }
            buffer
        })
    }

    fn serialize_config_overrides(config: &Value) -> Result<Vec<String>, CodexError> {
        let mut overrides = Vec::new();
        Self::flatten_config_overrides(config, "", &mut overrides)?;
        Ok(overrides)
    }

    fn flatten_config_overrides(
        value: &Value,
        prefix: &str,
        overrides: &mut Vec<String>,
    ) -> Result<(), CodexError> {
        let object = match value {
            Value::Object(map) => map,
            _ => {
                if prefix.is_empty() {
                    return Err(CodexError::InvalidConfigRoot);
                }
                overrides.push(format!(
                    "{}={}",
                    prefix,
                    Self::to_toml_value(value, prefix)?
                ));
                return Ok(());
            }
        };

        if prefix.is_empty() && object.is_empty() {
            return Ok(());
        }

        if !prefix.is_empty() && object.is_empty() {
            overrides.push(format!("{}={{}}", prefix));
            return Ok(());
        }

        for (key, child) in object {
            if key.is_empty() {
                return Err(CodexError::InvalidConfigKey);
            }
            if child.is_null() {
                continue;
            }
            let path = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };
            if child.is_object() {
                Self::flatten_config_overrides(child, &path, overrides)?;
            } else {
                overrides.push(format!("{}={}", path, Self::to_toml_value(child, &path)?));
            }
        }

        Ok(())
    }

    fn to_toml_value(value: &Value, path: &str) -> Result<String, CodexError> {
        match value {
            Value::String(value) => Ok(serde_json::to_string(value)?),
            Value::Number(num) => {
                let as_f64 = num
                    .as_f64()
                    .ok_or_else(|| CodexError::InvalidConfigNumber(path.into()))?;
                if !as_f64.is_finite() {
                    return Err(CodexError::InvalidConfigNumber(path.into()));
                }
                Ok(num.to_string())
            }
            Value::Bool(value) => Ok(if *value {
                "true".to_string()
            } else {
                "false".to_string()
            }),
            Value::Array(values) => {
                let mut rendered = Vec::with_capacity(values.len());
                for (index, item) in values.iter().enumerate() {
                    rendered.push(Self::to_toml_value(item, &format!("{}[{}]", path, index))?);
                }
                Ok(format!("[{}]", rendered.join(", ")))
            }
            Value::Object(map) => {
                let mut parts = Vec::new();
                for (key, child) in map {
                    if key.is_empty() {
                        return Err(CodexError::InvalidConfigKey);
                    }
                    if child.is_null() {
                        continue;
                    }
                    let toml_key = Self::format_toml_key(key);
                    parts.push(format!(
                        "{} = {}",
                        toml_key,
                        Self::to_toml_value(child, &format!("{}.{}", path, key))?
                    ));
                }
                Ok(format!("{{{}}}", parts.join(", ")))
            }
            Value::Null => Err(CodexError::InvalidConfigNull(path.into())),
        }
    }

    fn format_toml_key(key: &str) -> String {
        let is_bare = key
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
        if is_bare {
            key.to_string()
        } else {
            serde_json::to_string(key).unwrap_or_else(|_| format!("\"{}\"", key))
        }
    }
}
