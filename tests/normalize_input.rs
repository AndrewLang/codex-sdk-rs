use pretty_assertions::assert_eq;

use codex_sdk::{Input, Thread, UserInput};

#[test]
fn normalize_input_combines_text_and_collects_images() {
    let input = Input::Structured(vec![
        UserInput::Text {
            text: "Describe file changes".to_string(),
        },
        UserInput::Text {
            text: "Focus on impacted tests".to_string(),
        },
        UserInput::LocalImage {
            path: "./image.png".to_string(),
        },
    ]);

    let (prompt, images) = Thread::normalize_input(&input);
    assert_eq!(prompt, "Describe file changes\n\nFocus on impacted tests");
    assert_eq!(images, vec!["./image.png".to_string()]);
}
