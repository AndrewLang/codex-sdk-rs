use std::fs;

use pretty_assertions::assert_eq;
use serde_json::json;

use codex_sdk::OutputSchemaFile;

#[test]
fn output_schema_file_is_written_and_cleaned() {
    let schema = json!({
        "type": "object",
        "properties": { "answer": { "type": "string" } },
        "required": ["answer"],
        "additionalProperties": false,
    });

    let schema_path = {
        let file = OutputSchemaFile::new(Some(&schema)).expect("schema file");
        let path = file.schema_path().expect("schema path").to_path_buf();
        let contents = fs::read_to_string(&path).expect("read schema file");
        let parsed: serde_json::Value = serde_json::from_str(&contents).expect("json");
        assert_eq!(parsed, schema);
        path
    };

    assert_eq!(schema_path.exists(), false);
}
