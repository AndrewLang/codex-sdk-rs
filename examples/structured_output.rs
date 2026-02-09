use codex_sdk::{Codex, CodexOptions, ThreadOptions, TurnOptions};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let codex = Codex::new(CodexOptions::default())?;
    let thread = codex.start_thread(ThreadOptions::default());

    let schema = json!({
        "type": "object",
        "properties": {
            "summary": { "type": "string" },
            "status": { "type": "string", "enum": ["ok", "action_required"] }
        },
        "required": ["summary", "status"],
        "additionalProperties": false
    });

    let turn = thread
        .run(
            "Summarize repository status".into(),
            TurnOptions {
                output_schema: Some(schema),
                ..TurnOptions::default()
            },
        )
        .await?;

    println!("Final response: {}", turn.final_response);
    Ok(())
}
