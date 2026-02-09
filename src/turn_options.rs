use std::fmt;

use serde_json::Value;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, Default)]
pub struct TurnOptions {
    pub output_schema: Option<Value>,
    pub cancel: Option<CancellationToken>,
}

impl fmt::Display for TurnOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output_schema = self
            .output_schema
            .as_ref()
            .map(|value| format!("Some({value})"))
            .unwrap_or_else(|| "None".to_string());
        let cancel = if self.cancel.is_some() {
            "Some(<cancellation_token>)"
        } else {
            "None"
        };

        write!(
            f,
            "TurnOptions {{ output_schema: {}, cancel: {} }}",
            output_schema, cancel
        )
    }
}
