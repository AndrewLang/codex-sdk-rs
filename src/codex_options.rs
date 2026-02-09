use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use serde_json::Value;

pub type CodexConfigValue = Value;
pub type CodexConfigObject = serde_json::Map<String, Value>;

#[derive(Clone, Debug, Default)]
pub struct CodexOptions {
    pub codex_path_override: Option<PathBuf>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub config: Option<Value>,
    pub env: Option<HashMap<String, String>>,
}

impl fmt::Display for CodexOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let api_key = if self.api_key.is_some() {
            "Some([redacted])"
        } else {
            "None"
        };
        let config = self
            .config
            .as_ref()
            .map(|value| format!("Some({value})"))
            .unwrap_or_else(|| "None".to_string());
        let env = self
            .env
            .as_ref()
            .map(|vars| {
                let mut keys: Vec<&str> = vars.keys().map(String::as_str).collect();
                keys.sort_unstable();
                format!("Some(keys={keys:?})")
            })
            .unwrap_or_else(|| "None".to_string());

        write!(
            f,
            "CodexOptions {{ codex_path_override: {:?}, base_url: {:?}, api_key: {}, config: {}, env: {} }}",
            self.codex_path_override, self.base_url, api_key, config, env
        )
    }
}
