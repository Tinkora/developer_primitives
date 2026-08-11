use std::process::Command;

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tinkora-time"))
}

#[test]
fn version_reports_package_version() {
    let output = command().arg("--version").output().unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "tinkora-time 0.2.0\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn convert_json_preserves_requested_zone_order() {
    let output = command()
        .args([
            "convert",
            "--unix-seconds",
            "0",
            "--zone",
            "UTC",
            "--zone",
            "Asia/Shanghai",
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["instant"]["utc_rfc3339"], "1970-01-01T00:00:00Z");
    assert_eq!(value["zones"][0]["zone"], "UTC");
    assert_eq!(value["zones"][1]["zone"], "Asia/Shanghai");
}

#[test]
fn convert_accepts_a_negative_unix_millisecond_value_after_a_space() {
    let output = command()
        .args([
            "convert",
            "--unix-milliseconds",
            "-1",
            "--zone",
            "UTC",
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["instant"]["unix_seconds"], -1);
    assert_eq!(value["instant"]["unix_milliseconds"], -1);
    assert_eq!(value["instant"]["utc_rfc3339"], "1969-12-31T23:59:59.999Z");
}

#[test]
fn convert_rfc3339_writes_a_human_readable_result() {
    let output = command()
        .args([
            "convert",
            "--rfc3339",
            "2026-11-01T01:30:00-04:00",
            "--zone",
            "America/New_York",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("utc_rfc3339: 2026-11-01T05:30:00Z"));
    assert!(stdout.contains("zone: America/New_York"));
    assert!(stdout.contains("offset: -04:00"));
}

#[test]
fn resolve_json_reports_both_fold_candidates() {
    let output = command()
        .args([
            "resolve",
            "--local",
            "2026-11-01T01:30:00",
            "--zone",
            "America/New_York",
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["resolution"]["status"], "FOLD");
    assert_eq!(
        value["resolution"]["earlier"]["unix_seconds"],
        1_793_511_000
    );
    assert_eq!(value["resolution"]["later"]["unix_seconds"], 1_793_514_600);
}

#[test]
fn zone_search_is_case_insensitive_and_bounded() {
    let output = command()
        .args(["zones", "--filter", "SHANGHAI", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert!(
        value["zones"]
            .as_array()
            .unwrap()
            .iter()
            .any(|zone| zone == "Asia/Shanghai")
    );
    assert!(value["zones"].as_array().unwrap().len() <= 50);
}

#[test]
fn zone_lookup_returns_the_canonical_exact_name() {
    let output = command()
        .args(["zones", "--name", "asia/shanghai", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["zones"], serde_json::json!(["Asia/Shanghai"]));
}

#[test]
fn zone_lookup_rejects_an_unknown_exact_name_with_a_stable_error_code() {
    let output = command()
        .args(["zones", "--name", "Mars/Olympus"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "INVALID_TIMEZONE: Invalid IANA time zone\n"
    );
}

#[test]
fn zone_search_without_a_filter_returns_the_bounded_default_set() {
    let output = command().args(["zones", "--json"]).output().unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let zones = value["zones"].as_array().unwrap();
    assert_eq!(zones.len(), 50);
    let names: Vec<_> = zones.iter().map(|zone| zone.as_str().unwrap()).collect();
    assert!(names.windows(2).all(|pair| pair[0] <= pair[1]));
}

#[test]
fn operational_failure_uses_a_stable_error_code() {
    let output = command()
        .args([
            "convert",
            "--unix-seconds",
            "not-a-timestamp",
            "--zone",
            "UTC",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "INVALID_TIMESTAMP: Invalid timestamp\n"
    );
}

#[test]
fn multiple_instant_flags_are_a_usage_error() {
    let output = command()
        .args([
            "convert",
            "--unix-seconds",
            "0",
            "--unix-milliseconds",
            "0",
            "--zone",
            "UTC",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("cannot be used with")
    );
}

#[test]
fn convert_without_a_zone_is_a_usage_error() {
    let output = command()
        .args(["convert", "--unix-seconds", "0"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("required arguments were not provided")
    );
}
