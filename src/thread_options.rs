use std::fmt;

#[derive(Clone, Debug)]
pub enum ApprovalMode {
    Never,
    OnRequest,
    OnFailure,
    Untrusted,
}

impl ApprovalMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApprovalMode::Never => "never",
            ApprovalMode::OnRequest => "on-request",
            ApprovalMode::OnFailure => "on-failure",
            ApprovalMode::Untrusted => "untrusted",
        }
    }
}

impl fmt::Display for ApprovalMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug)]
pub enum SandboxMode {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

impl SandboxMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SandboxMode::ReadOnly => "read-only",
            SandboxMode::WorkspaceWrite => "workspace-write",
            SandboxMode::DangerFullAccess => "danger-full-access",
        }
    }
}

impl fmt::Display for SandboxMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug)]
pub enum ModelReasoningEffort {
    Minimal,
    Low,
    Medium,
    High,
    XHigh,
}

impl ModelReasoningEffort {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelReasoningEffort::Minimal => "minimal",
            ModelReasoningEffort::Low => "low",
            ModelReasoningEffort::Medium => "medium",
            ModelReasoningEffort::High => "high",
            ModelReasoningEffort::XHigh => "xhigh",
        }
    }
}

impl fmt::Display for ModelReasoningEffort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug)]
pub enum WebSearchMode {
    Disabled,
    Cached,
    Live,
}

impl WebSearchMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            WebSearchMode::Disabled => "disabled",
            WebSearchMode::Cached => "cached",
            WebSearchMode::Live => "live",
        }
    }
}

impl fmt::Display for WebSearchMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Default)]
pub struct ThreadOptions {
    pub model: Option<String>,
    pub sandbox_mode: Option<SandboxMode>,
    pub working_directory: Option<String>,
    pub skip_git_repo_check: Option<bool>,
    pub model_reasoning_effort: Option<ModelReasoningEffort>,
    pub network_access_enabled: Option<bool>,
    pub web_search_mode: Option<WebSearchMode>,
    pub web_search_enabled: Option<bool>,
    pub approval_policy: Option<ApprovalMode>,
    pub additional_directories: Option<Vec<String>>,
}

impl fmt::Display for ThreadOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ThreadOptions {{ model: {:?}, sandbox_mode: {}, working_directory: {:?}, skip_git_repo_check: {:?}, model_reasoning_effort: {}, network_access_enabled: {:?}, web_search_mode: {}, web_search_enabled: {:?}, approval_policy: {}, additional_directories: {:?} }}",
            self.model,
            Self::format_option(self.sandbox_mode.as_ref()),
            self.working_directory,
            self.skip_git_repo_check,
            Self::format_option(self.model_reasoning_effort.as_ref()),
            self.network_access_enabled,
            Self::format_option(self.web_search_mode.as_ref()),
            self.web_search_enabled,
            Self::format_option(self.approval_policy.as_ref()),
            self.additional_directories,
        )
    }
}

impl ThreadOptions {
    fn format_option<T: fmt::Display>(value: Option<&T>) -> String {
        value
            .map(|value| format!("Some({value})"))
            .unwrap_or_else(|| "None".to_string())
    }
}
