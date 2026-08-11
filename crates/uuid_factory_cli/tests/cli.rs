use std::io::Write;
use std::process::{Command, Stdio};

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tinkora-id"))
}

#[test]
fn version_reports_package_version() {
    let output = command().arg("--version").output().unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "tinkora-id 0.2.0\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn generate_single_uuid_v4_as_text() {
    let output = command()
        .args(["generate", "--kind", "uuid-v4", "--count", "1"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let identifier = String::from_utf8(output.stdout).unwrap();
    let identifier = identifier.trim_end();
    assert_eq!(identifier.len(), 36);
    assert_eq!(&identifier[14..15], "4");
}

#[test]
fn generate_batch_preserves_text_line_order() {
    let output = command()
        .args(["generate", "--kind", "uuid-v7", "--count", "3"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let identifiers: Vec<_> = stdout.lines().collect();
    assert_eq!(identifiers.len(), 3);
    assert!(identifiers.iter().all(|identifier| identifier.len() == 36));
    assert!(
        identifiers
            .iter()
            .all(|identifier| &identifier[14..15] == "7")
    );
}

#[test]
fn generate_json_uses_versioned_schema() {
    let output = command()
        .args(["generate", "--kind", "ulid", "--count", "2", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["kind"], "ulid");
    assert_eq!(value["count"], 2);
    assert_eq!(value["identifiers"].as_array().unwrap().len(), 2);
}

#[test]
fn inspect_positional_identifier_as_text() {
    let input = "550e8400-e29b-41d4-a716-446655440000";
    let output = command().args(["inspect", input]).output().unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("kind: uuid\n"));
    assert!(stdout.contains(&format!("canonical: {input}\n")));
    assert!(stdout.contains("version: 4\n"));
    assert!(stdout.contains("variant: RFC4122\n"));
}

#[test]
fn inspect_reads_stdin_and_emits_json() {
    let input = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
    let mut child = command()
        .args(["inspect", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{input}\n").as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["input"], input);
    assert_eq!(value["canonical"], input);
    assert_eq!(value["kind"], "ulid");
    assert_eq!(value["timestamp_ms"], 1_469_922_850_259_u64);
}

#[test]
fn invalid_kind_is_a_usage_error() {
    let output = command()
        .args(["generate", "--kind", "uuid-v5"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("invalid value 'uuid-v5'"));
}

#[test]
fn invalid_count_is_an_operational_error() {
    let output = command()
        .args(["generate", "--kind", "uuid-v4", "--count", "0"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "BATCH_OUT_OF_RANGE: Batch count must be between 1 and 10000: 0\n"
    );
}

#[test]
fn invalid_identifier_is_an_operational_error() {
    let output = command().args(["inspect", "not-an-id"]).output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "INVALID_IDENTIFIER: Invalid identifier\n"
    );
}
