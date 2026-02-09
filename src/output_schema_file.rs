use std::path::Path;
use std::path::PathBuf;

use serde_json::Value;
use tempfile::TempDir;

use crate::error::CodexError;

pub struct OutputSchemaFile {
    schema_path: Option<PathBuf>,
    _temp_dir: Option<TempDir>,
}

impl OutputSchemaFile {
    pub fn new(schema: Option<&Value>) -> Result<Self, CodexError> {
        match schema {
            None => {
                log::debug!("No output schema provided");
                Ok(Self {
                    schema_path: None,
                    _temp_dir: None,
                })
            }
            Some(value) => {
                if !value.is_object() {
                    return Err(CodexError::InvalidOutputSchema);
                }

                let temp_dir = tempfile::Builder::new()
                    .prefix("codex-output-schema-")
                    .tempdir()?;
                let schema_path = temp_dir.path().join("schema.json");
                std::fs::write(&schema_path, serde_json::to_vec(value)?)?;
                log::debug!("Wrote output schema to {:?}", schema_path);

                Ok(Self {
                    schema_path: Some(schema_path),
                    _temp_dir: Some(temp_dir),
                })
            }
        }
    }

    pub fn schema_path(&self) -> Option<&Path> {
        self.schema_path.as_deref()
    }
}
