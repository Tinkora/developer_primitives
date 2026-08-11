use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use timestamp_zone_core::{
    InstantInputKind, LocalResolution, TimeConversion, TimeError, convert_instant,
    resolve_local_time, search_time_zones, time_zone_database_version,
};

#[derive(Parser)]
#[command(name = "tinkora-time", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Convert {
        #[command(flatten)]
        input: InstantInput,
        #[arg(long = "zone", required = true)]
        zones: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    Resolve {
        #[arg(long)]
        local: String,
        #[arg(long)]
        zone: String,
        #[arg(long)]
        json: bool,
    },
    Zones {
        #[arg(long)]
        filter: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args)]
#[group(
    id = "instant_input",
    required = true,
    multiple = false,
    args = ["unix_seconds", "unix_milliseconds", "rfc3339"]
)]
struct InstantInput {
    #[arg(long)]
    unix_seconds: Option<String>,
    #[arg(long)]
    unix_milliseconds: Option<String>,
    #[arg(long)]
    rfc3339: Option<String>,
}

impl InstantInput {
    fn into_parts(self) -> (InstantInputKind, String) {
        match (self.unix_seconds, self.unix_milliseconds, self.rfc3339) {
            (Some(value), None, None) => (InstantInputKind::UnixSeconds, value),
            (None, Some(value), None) => (InstantInputKind::UnixMilliseconds, value),
            (None, None, Some(value)) => (InstantInputKind::Rfc3339, value),
            _ => unreachable!("Clap validates the instant input group"),
        }
    }
}

#[derive(Serialize)]
struct ZoneSearchOutput {
    schema_version: u32,
    tzdb_version: String,
    filter: String,
    zones: Vec<String>,
}

fn run(cli: Cli) -> Result<(), TimeError> {
    match cli.command {
        Command::Convert { input, zones, json } => {
            let (kind, input) = input.into_parts();
            let zone_names: Vec<_> = zones.iter().map(String::as_str).collect();
            let result = convert_instant(kind, &input, &zone_names)?;
            if json {
                print_json(&result)?;
            } else {
                print_conversion(&result);
            }
        }
        Command::Resolve { local, zone, json } => {
            let result = resolve_local_time(&local, &zone)?;
            if json {
                print_json(&result)?;
            } else {
                print_resolution(&result);
            }
        }
        Command::Zones { filter, json } => {
            let filter = filter.unwrap_or_default();
            let output = ZoneSearchOutput {
                schema_version: 1,
                tzdb_version: time_zone_database_version().to_string(),
                zones: search_time_zones(&filter)?,
                filter,
            };
            if json {
                print_json(&output)?;
            } else {
                println!("tzdb_version: {}", output.tzdb_version);
                println!("filter: {}", output.filter);
                for zone in output.zones {
                    println!("zone: {zone}");
                }
            }
        }
    }

    Ok(())
}

fn print_json(value: &impl Serialize) -> Result<(), TimeError> {
    let serialized = serde_json::to_string(value).map_err(|_| TimeError::SerializationFailed)?;
    println!("{serialized}");
    Ok(())
}

fn print_conversion(result: &TimeConversion) {
    println!("schema_version: {}", result.schema_version);
    println!("tzdb_version: {}", result.tzdb_version);
    println!("unix_seconds: {}", result.instant.unix_seconds);
    println!("unix_milliseconds: {}", result.instant.unix_milliseconds);
    println!("utc_rfc3339: {}", result.instant.utc_rfc3339);
    for zone in &result.zones {
        println!("zone: {}", zone.zone);
        println!("local_datetime: {}", zone.local_datetime);
        println!("offset: {}", zone.offset);
        println!("abbreviation: {}", zone.abbreviation);
        println!("is_dst: {}", format_dst(zone.is_dst));
    }
}

fn print_resolution(result: &timestamp_zone_core::LocalTimeResult) {
    println!("schema_version: {}", result.schema_version);
    println!("tzdb_version: {}", result.tzdb_version);
    println!("zone: {}", result.zone);
    println!("local_datetime: {}", result.local_datetime);
    match &result.resolution {
        LocalResolution::Unambiguous { instant } => {
            println!("status: UNAMBIGUOUS");
            print_candidate("instant", instant);
        }
        LocalResolution::Gap {
            before_offset,
            after_offset,
        } => {
            println!("status: GAP");
            println!("before_offset: {before_offset}");
            println!("after_offset: {after_offset}");
        }
        LocalResolution::Fold { earlier, later } => {
            println!("status: FOLD");
            print_candidate("earlier", earlier);
            print_candidate("later", later);
        }
    }
}

fn print_candidate(label: &str, candidate: &timestamp_zone_core::CandidateInstant) {
    println!("{label}.unix_seconds: {}", candidate.unix_seconds);
    println!("{label}.unix_milliseconds: {}", candidate.unix_milliseconds);
    println!("{label}.utc_rfc3339: {}", candidate.utc_rfc3339);
    println!("{label}.offset: {}", candidate.offset);
    println!("{label}.abbreviation: {}", candidate.abbreviation);
    println!("{label}.is_dst: {}", format_dst(candidate.is_dst));
}

fn format_dst(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("{}: {error}", error.code());
        std::process::exit(1);
    }
}
