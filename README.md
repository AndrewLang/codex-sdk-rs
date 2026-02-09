# Codex Rust SDK

Embed the Codex agent in your Rust workflows and apps.

The SDK spawns the Codex CLI and exchanges JSONL events over stdin/stdout.

## Quickstart

The samples live in the examples/ folder. The code below mirrors those files.

Run a sample (PowerShell):

```powershell
cargo run --example streaming
```

```rust
use codex_sdk::{Codex, CodexOptions, ThreadOptions, TurnOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let codex = Codex::new(CodexOptions::default())?;
    let thread = codex.start_thread(ThreadOptions::default());
    let turn = thread.run("Diagnose the test failure".into(), TurnOptions::default()).await?;

    println!("Final response: {}", turn.final_response);
    Ok(())
}
```

## Streaming responses

The streaming sample is in examples/streaming.rs.

```rust
use codex_sdk::{Codex, CodexOptions, ThreadEvent, ThreadOptions, TurnOptions};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let codex = Codex::new(CodexOptions::default())?;
    let thread = codex.start_thread(ThreadOptions::default());
    let streamed = thread.run_streamed("Diagnose the failure".into(), TurnOptions::default())?;

    let mut events = streamed.events;
    while let Some(event) = events.next().await {
        match event? {
            ThreadEvent::ItemCompleted { item } => println!("item: {item:?}"),
            ThreadEvent::TurnCompleted { usage } => println!("usage: {usage:?}"),
            _ => {}
        }
    }

    Ok(())
}
```

## Structured output

The structured output sample is in examples/structured_output.rs.

```rust
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
```
