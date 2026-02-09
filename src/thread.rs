use std::pin::Pin;
use std::sync::{Arc, Mutex};

use async_stream::try_stream;
use futures::{Stream, StreamExt};

use crate::codex_options::CodexOptions;
use crate::error::CodexError;
use crate::events::{ThreadError, ThreadEvent, Usage};
use crate::exec::{CodexExec, CodexExecArgs};
use crate::items::ThreadItem;
use crate::output_schema_file::OutputSchemaFile;
use crate::thread_options::ThreadOptions;
use crate::turn_options::TurnOptions;

#[derive(Clone, Debug)]
pub struct Turn {
    pub items: Vec<ThreadItem>,
    pub final_response: String,
    pub usage: Option<Usage>,
}

pub type RunResult = Turn;

pub type ThreadEventStream = Pin<Box<dyn Stream<Item = Result<ThreadEvent, CodexError>> + Send>>;

pub struct StreamedTurn {
    pub events: ThreadEventStream,
}

pub type RunStreamedResult = StreamedTurn;

#[derive(Clone, Debug, PartialEq)]
pub enum UserInput {
    Text { text: String },
    LocalImage { path: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    Text(String),
    Structured(Vec<UserInput>),
}

impl From<&str> for Input {
    fn from(value: &str) -> Self {
        Input::Text(value.to_string())
    }
}

impl From<String> for Input {
    fn from(value: String) -> Self {
        Input::Text(value)
    }
}

#[derive(Clone, Debug)]
pub struct Thread {
    exec: CodexExec,
    options: CodexOptions,
    id: Arc<Mutex<Option<String>>>,
    thread_options: ThreadOptions,
}

impl Thread {
    pub(crate) fn new(
        exec: CodexExec,
        options: CodexOptions,
        thread_options: ThreadOptions,
        id: Option<String>,
    ) -> Self {
        Self {
            exec,
            options,
            id: Arc::new(Mutex::new(id)),
            thread_options,
        }
    }

    pub fn id(&self) -> Option<String> {
        self.id.lock().ok().and_then(|guard| guard.clone())
    }

    pub fn run_streamed(
        &self,
        input: Input,
        turn_options: TurnOptions,
    ) -> Result<StreamedTurn, CodexError> {
        let events = self.run_streamed_internal(input, turn_options)?;
        Ok(StreamedTurn { events })
    }

    fn run_streamed_internal(
        &self,
        input: Input,
        turn_options: TurnOptions,
    ) -> Result<ThreadEventStream, CodexError> {
        log::debug!("Running thread with input: {:?}", input);
        log::debug!("Thread options: {:?}", self.thread_options);

        let schema_file = OutputSchemaFile::new(turn_options.output_schema.as_ref())?;
        log::debug!(
            "Output schema path: {:?}",
            schema_file.schema_path().map(|path| path.to_path_buf())
        );

        let (prompt, images) = Self::normalize_input(&input);
        log::debug!("Normalized input {}, images: {}", prompt, images.len());

        let thread_id = self.id();
        log::debug!("Thread id: {:?}", thread_id);

        let exec_args = CodexExecArgs {
            input: prompt,
            base_url: self.options.base_url.clone(),
            api_key: self.options.api_key.clone(),
            thread_id,
            images: if images.is_empty() {
                None
            } else {
                Some(images)
            },
            model: self.thread_options.model.clone(),
            sandbox_mode: self.thread_options.sandbox_mode.clone(),
            working_directory: self.thread_options.working_directory.clone(),
            additional_directories: self.thread_options.additional_directories.clone(),
            skip_git_repo_check: self.thread_options.skip_git_repo_check,
            output_schema_file: schema_file.schema_path().map(|path| path.to_path_buf()),
            model_reasoning_effort: self.thread_options.model_reasoning_effort.clone(),
            cancel: turn_options.cancel.clone(),
            network_access_enabled: self.thread_options.network_access_enabled,
            web_search_mode: self.thread_options.web_search_mode.clone(),
            web_search_enabled: self.thread_options.web_search_enabled,
            approval_policy: self.thread_options.approval_policy.clone(),
        };
        log::debug!("Exec args: {}", exec_args);

        let mut lines = self.exec.run(exec_args)?;
        let thread_id_handle = self.id.clone();

        let stream = try_stream! {
            let _schema_guard = schema_file;
            while let Some(line) = lines.next().await {
                let line = line?;
                let parsed: ThreadEvent = serde_json::from_str(&line)
                    .map_err(|_| CodexError::InvalidEvent(line.clone()))?;

                log::debug!("Received event: {}", Self::event_type(&parsed));

                if let ThreadEvent::ThreadStarted { thread_id } = &parsed {
                    if let Ok(mut guard) = thread_id_handle.lock() {
                        *guard = Some(thread_id.clone());
                    }
                    log::debug!("Thread started: {}", thread_id);
                }
                yield parsed;
            }
        };

        Ok(Box::pin(stream))
    }

    pub async fn run(&self, input: Input, turn_options: TurnOptions) -> Result<Turn, CodexError> {
        let mut events = self.run_streamed_internal(input, turn_options)?;
        let mut items = Vec::new();
        let mut final_response = String::new();
        let mut usage: Option<Usage> = None;
        let mut turn_failure: Option<ThreadError> = None;

        while let Some(event) = events.next().await {
            let event = event?;
            match event {
                ThreadEvent::ItemCompleted { item } => {
                    if let ThreadItem::AgentMessage { text, .. } = &item {
                        final_response = text.clone();
                    }
                    items.push(item);
                }
                ThreadEvent::TurnCompleted { usage: event_usage } => {
                    usage = Some(event_usage);
                    log::debug!("Turn completed");
                }
                ThreadEvent::TurnFailed { error } => {
                    turn_failure = Some(error);
                    log::debug!("Turn failed");
                    break;
                }
                _ => {}
            }
        }

        if let Some(error) = turn_failure {
            return Err(CodexError::TurnFailed(error.message));
        }

        Ok(Turn {
            items,
            final_response,
            usage,
        })
    }

    #[doc(hidden)]
    pub fn normalize_input(input: &Input) -> (String, Vec<String>) {
        match input {
            Input::Text(text) => (text.clone(), Vec::new()),
            Input::Structured(items) => {
                let mut prompt_parts = Vec::new();
                let mut images = Vec::new();
                for item in items {
                    match item {
                        UserInput::Text { text } => prompt_parts.push(text.clone()),
                        UserInput::LocalImage { path } => images.push(path.clone()),
                    }
                }
                (prompt_parts.join("\n\n"), images)
            }
        }
    }

    fn event_type(event: &ThreadEvent) -> &'static str {
        match event {
            ThreadEvent::ThreadStarted { .. } => "thread.started",
            ThreadEvent::TurnStarted => "turn.started",
            ThreadEvent::TurnCompleted { .. } => "turn.completed",
            ThreadEvent::TurnFailed { .. } => "turn.failed",
            ThreadEvent::ItemStarted { .. } => "item.started",
            ThreadEvent::ItemUpdated { .. } => "item.updated",
            ThreadEvent::ItemCompleted { .. } => "item.completed",
            ThreadEvent::ThreadErrorEvent { .. } => "error",
        }
    }
}
