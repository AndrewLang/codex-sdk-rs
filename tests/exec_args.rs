use pretty_assertions::assert_eq;
use serde_json::json;

use codex_sdk::{CodexExec, CodexExecArgs};

#[test]
fn config_overrides_become_toml_flags() {
    let exec = CodexExec::new(
        Some("codex".into()),
        None,
        Some(json!({
            "approval_policy": "never",
            "sandbox_workspace_write": { "network_access": true },
            "retry_budget": 3,
            "tool_rules": { "allow": ["git status", "git diff"] },
        })),
    )
    .expect("exec");

    let args = CodexExecArgs {
        input: "hello".to_string(),
        ..Default::default()
    };

    let spec = exec.build_command(&args).expect("command spec");
    assert_pair(&spec.args, "--config", "approval_policy=\"never\"");
    assert_pair(
        &spec.args,
        "--config",
        "sandbox_workspace_write.network_access=true",
    );
    assert_pair(&spec.args, "--config", "retry_budget=3");
    assert_pair(
        &spec.args,
        "--config",
        "tool_rules.allow=[\"git status\", \"git diff\"]",
    );
}

#[test]
fn resume_args_come_before_images() {
    let exec = CodexExec::new(Some("codex".into()), None, None).expect("exec");
    let args = CodexExecArgs {
        input: "hello".to_string(),
        thread_id: Some("thread-id".to_string()),
        images: Some(vec!["img.png".to_string()]),
        ..Default::default()
    };

    let spec = exec.build_command(&args).expect("command spec");
    let resume_index = spec.args.iter().position(|arg| arg == "resume");
    let image_index = spec.args.iter().position(|arg| arg == "--image");

    assert_eq!(resume_index.is_some(), true);
    assert_eq!(image_index.is_some(), true);
    assert!(resume_index < image_index);
}

fn assert_pair(args: &[String], key: &str, value: &str) {
    let mut found = false;
    for i in 0..args.len().saturating_sub(1) {
        if args[i] == key && args[i + 1] == value {
            found = true;
            break;
        }
    }
    assert!(found, "pair {key} {value} missing");
}
