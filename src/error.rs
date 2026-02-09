use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodexError {
    #[error("unsupported platform: {0} ({1})")]
    UnsupportedPlatform(String, String),
    #[error("codex config overrides must be a plain object")]
    InvalidConfigRoot,
    #[error("codex config override keys must be non-empty strings")]
    InvalidConfigKey,
    #[error("codex config override at {0} must be a finite number")]
    InvalidConfigNumber(String),
    #[error("codex config override at {0} cannot be null")]
    InvalidConfigNull(String),
    #[error("unsupported codex config override value at {0}: {1}")]
    InvalidConfigValue(String, String),
    #[error("output schema must be a plain JSON object")]
    InvalidOutputSchema,
    #[error("failed to parse event: {0}")]
    InvalidEvent(String),
    #[error("codex exec exited with {0}: {1}")]
    ExecFailed(String, String),
    #[error("codex exec aborted")]
    Aborted,
    #[error("turn failed: {0}")]
    TurnFailed(String),
    #[error("child process missing {0}")]
    MissingChildStream(&'static str),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
