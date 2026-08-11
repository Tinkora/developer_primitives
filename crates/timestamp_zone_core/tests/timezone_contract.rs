use timestamp_zone_core::{
    InstantInputKind, LocalResolution, TimeError, convert_instant, resolve_local_time,
    search_time_zones, time_zone_database_version,
};

#[test]
fn publishes_the_bundled_iana_release() {
    assert_eq!(time_zone_database_version(), "2026c");
}

#[test]
fn converts_one_instant_with_each_zone_in_requested_order() {
    let result = convert_instant(
        InstantInputKind::UnixSeconds,
        "1768478400",
        &["UTC", "Asia/Shanghai", "America/New_York"],
    )
    .unwrap();

    let zones: Vec<_> = result.zones.iter().map(|zone| zone.zone.as_str()).collect();
    assert_eq!(zones, ["UTC", "Asia/Shanghai", "America/New_York"]);
    assert_eq!(result.zones[0].offset, "+00:00");
    assert_eq!(result.zones[1].offset, "+08:00");
    assert_eq!(result.zones[2].offset, "-05:00");
    assert_eq!(result.zones[2].abbreviation, "EST");
    assert_eq!(result.zones[2].is_dst, Some(false));
}

#[test]
fn applies_winter_and_summer_iana_offsets() {
    let winter = convert_instant(
        InstantInputKind::UnixSeconds,
        "1768478400",
        &[
            "America/New_York",
            "Europe/London",
            "Australia/Sydney",
            "Asia/Shanghai",
            "Asia/Kolkata",
        ],
    )
    .unwrap();
    let summer = convert_instant(
        InstantInputKind::UnixSeconds,
        "1784116800",
        &["America/New_York", "Europe/London", "Australia/Sydney"],
    )
    .unwrap();

    assert_eq!(winter.zones[0].offset, "-05:00");
    assert_eq!(winter.zones[1].offset, "+00:00");
    assert_eq!(winter.zones[2].offset, "+11:00");
    assert_eq!(winter.zones[3].offset, "+08:00");
    assert_eq!(winter.zones[4].offset, "+05:30");
    assert_eq!(summer.zones[0].offset, "-04:00");
    assert_eq!(summer.zones[1].offset, "+01:00");
    assert_eq!(summer.zones[2].offset, "+10:00");
}

#[test]
fn reports_new_york_spring_gap_without_shifting_the_input() {
    let result = resolve_local_time("2026-03-08T02:30:00", "America/New_York").unwrap();

    assert_eq!(result.tzdb_version, "2026c");
    assert_eq!(
        result.resolution,
        LocalResolution::Gap {
            before_offset: "-05:00".into(),
            after_offset: "-04:00".into(),
        }
    );
}

#[test]
fn reports_both_new_york_fall_fold_candidates() {
    let result = resolve_local_time("2026-11-01T01:30:00", "America/New_York").unwrap();

    let LocalResolution::Fold { earlier, later } = result.resolution else {
        panic!("expected a fold result");
    };
    assert_eq!(earlier.unix_seconds, 1_793_511_000);
    assert_eq!(earlier.offset, "-04:00");
    assert_eq!(later.unix_seconds, 1_793_514_600);
    assert_eq!(later.offset, "-05:00");
    assert!(earlier.unix_seconds < later.unix_seconds);
}

#[test]
fn rejects_unknown_duplicate_and_excessive_zones() {
    let none = convert_instant(InstantInputKind::UnixSeconds, "0", &[]).unwrap_err();
    assert_eq!(none, TimeError::TimezoneLimitExceeded);

    let unknown =
        convert_instant(InstantInputKind::UnixSeconds, "0", &["Mars/Olympus"]).unwrap_err();
    assert_eq!(unknown, TimeError::InvalidTimezone);

    let duplicate =
        convert_instant(InstantInputKind::UnixSeconds, "0", &["UTC", "UTC"]).unwrap_err();
    assert_eq!(duplicate, TimeError::DuplicateTimezone);

    let maximum = convert_instant(
        InstantInputKind::UnixSeconds,
        "0",
        &[
            "UTC",
            "Etc/GMT",
            "Etc/GMT+1",
            "Etc/GMT+2",
            "Etc/GMT+3",
            "Etc/GMT+4",
            "Etc/GMT+5",
            "Etc/GMT+6",
        ],
    )
    .unwrap();
    assert_eq!(maximum.zones.len(), 8);

    let too_many = convert_instant(
        InstantInputKind::UnixSeconds,
        "0",
        &[
            "UTC",
            "Etc/GMT",
            "Etc/GMT+1",
            "Etc/GMT+2",
            "Etc/GMT+3",
            "Etc/GMT+4",
            "Etc/GMT+5",
            "Etc/GMT+6",
            "Etc/GMT+7",
        ],
    )
    .unwrap_err();
    assert_eq!(too_many, TimeError::TimezoneLimitExceeded);
}

#[test]
fn rejects_local_input_with_an_offset() {
    let error = resolve_local_time("2026-11-01T01:30:00-04:00", "America/New_York").unwrap_err();

    assert_eq!(error, TimeError::InvalidLocalDateTime);
}

#[test]
fn searches_zone_names_case_insensitively_in_stable_bounded_order() {
    let matches = search_time_zones("SHANGHAI").unwrap();
    assert_eq!(matches, ["Asia/Shanghai"]);

    let all = search_time_zones("").unwrap();
    assert_eq!(all.len(), 50);
    assert!(all.windows(2).all(|pair| pair[0] <= pair[1]));
}

#[test]
fn rejects_an_overlong_zone_search_filter() {
    let filter = "x".repeat(129);
    let error = search_time_zones(&filter).unwrap_err();

    assert_eq!(error, TimeError::InputTooLong);
}
