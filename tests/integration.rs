use std::process::Command;

fn run_with_stdin(input: &str) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_cstat");
    Command::new(bin)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .env("HOME", "/nonexistent")
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(input.as_bytes()).ok();
            }
            child.wait_with_output()
        })
        .unwrap()
}

#[test]
fn empty_stdin_exits_0() {
    let out = run_with_stdin("");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("cstat"));
    assert!(stdout.contains("no data"));
}

#[test]
fn invalid_json_stdin_exits_0() {
    let out = run_with_stdin("not json at all");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("cstat"));
    assert!(stdout.contains("no data"));
}

#[test]
fn minimal_json_exits_0() {
    let out = run_with_stdin("{}");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("cstat"));
    assert!(stdout.contains("no data"));
}

#[test]
fn partial_json_exits_0() {
    let input = r#"{"model": {"display_name": "Opus"}, "cwd": "/tmp/test"}"#;
    let out = run_with_stdin(input);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Opus"));
    assert!(stdout.contains("test"));
}

#[test]
fn missing_transcript_exits_0() {
    let input = r#"{"model": {"display_name": "X"}, "cwd": "/tmp/p", "transcript_path": "/nonexistent/transcript.jsonl"}"#;
    let out = run_with_stdin(input);
    assert!(out.status.success());
}

#[test]
fn stdout_never_contains_error_messages() {
    let out = run_with_stdin("");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("error"));
    assert!(!stdout.contains("panic"));
    assert!(!stdout.contains("Error"));
}

#[test]
fn stdout_ends_with_newline() {
    let out = run_with_stdin("");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.ends_with('\n'));
}

#[test]
fn two_line_output() {
    let input = r#"{"model": {"display_name": "Opus"}, "cwd": "/tmp/proj"}"#;
    let out = run_with_stdin(input);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("Opus"));
    assert!(lines[1].contains("proj"));
}
