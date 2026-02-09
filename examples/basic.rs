use codex_sdk::{Codex, CodexOptions, ThreadOptions, TurnOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let codex = Codex::new(CodexOptions::default())?;
    let thread = codex.start_thread(ThreadOptions::default());

    let turn = thread
        .run(
            "Provide a good prompt template for a coding assistant".into(),
            TurnOptions::default(),
        )
        .await?;

    println!("Final response: {}", turn.final_response);
    println!("Items: {}", turn.items.len());
    Ok(())
}
