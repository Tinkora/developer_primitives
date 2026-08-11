use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::io::Read;
use uuid_factory_core::{
    CoreError, IdKind, MAX_IDENTIFIER_INPUT_LEN, batch_generate, inspect_identifier,
};

#[derive(Parser)]
#[command(name = "tinkora-id", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Generate {
        #[arg(long, value_enum)]
        kind: Kind,
        #[arg(long, default_value_t = 1)]
        count: u32,
        #[arg(long)]
        json: bool,
    },
    Inspect {
        identifier: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Kind {
    UuidV4,
    UuidV7,
    Ulid,
}

impl Kind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::UuidV4 => "uuid-v4",
            Self::UuidV7 => "uuid-v7",
            Self::Ulid => "ulid",
        }
    }
}

impl From<Kind> for IdKind {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::UuidV4 => Self::UuidV4,
            Kind::UuidV7 => Self::UuidV7,
            Kind::Ulid => Self::Ulid,
        }
    }
}

#[derive(Serialize)]
struct GenerateOutput {
    schema_version: u32,
    kind: &'static str,
    count: u32,
    identifiers: Vec<String>,
}

fn run(cli: Cli) -> Result<(), CoreError> {
    match cli.command {
        Command::Generate { kind, count, json } => {
            let identifiers = batch_generate(kind.into(), count)?;
            if json {
                let output = GenerateOutput {
                    schema_version: 1,
                    kind: kind.as_str(),
                    count,
                    identifiers,
                };
                let serialized =
                    serde_json::to_string(&output).map_err(|_| CoreError::SerializationFailed)?;
                println!("{serialized}");
            } else {
                println!("{}", identifiers.join("\n"));
            }
        }
        Command::Inspect { identifier, json } => {
            let input = match identifier {
                Some(value) => value,
                None => {
                    let mut value = String::new();
                    std::io::stdin()
                        .take((MAX_IDENTIFIER_INPUT_LEN + 3) as u64)
                        .read_to_string(&mut value)
                        .map_err(|_| CoreError::InvalidIdentifier)?;
                    value.trim_end_matches(['\r', '\n']).to_string()
                }
            };
            let inspection = inspect_identifier(&input)?;
            if json {
                let serialized = serde_json::to_string(&inspection)
                    .map_err(|_| CoreError::SerializationFailed)?;
                println!("{serialized}");
            } else {
                println!("kind: {}", inspection.kind);
                println!("canonical: {}", inspection.canonical);
                if let Some(version) = inspection.version {
                    println!("version: {version}");
                }
                if let Some(variant) = inspection.variant {
                    println!("variant: {variant}");
                }
                if let Some(timestamp_ms) = inspection.timestamp_ms {
                    println!("timestamp_ms: {timestamp_ms}");
                }
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("{}: {error}", error.code());
        std::process::exit(1);
    }
}
