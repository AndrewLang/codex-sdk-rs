use codex_sdk::{Codex, CodexOptions, ThreadEvent, ThreadOptions, TurnOptions};
use futures::StreamExt;
use log::LevelFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_module("codex_sdk", LevelFilter::Debug)
        .try_init();

    let codex = Codex::new(CodexOptions::default())?;
    let thread = codex.start_thread(ThreadOptions::default());

    let streamed = thread.run_streamed(
        "Provide a good prompt template for a coding assistant\n".into(),
        TurnOptions::default(),
    )?;

    let mut events = streamed.events;
    while let Some(event) = events.next().await {
        match event? {
            ThreadEvent::ItemCompleted { item } => println!("Item: {item:?}"),
            ThreadEvent::TurnCompleted { usage } => println!("Usage: {usage:?}"),
            _ => {}
        }
    }

    Ok(())
}
