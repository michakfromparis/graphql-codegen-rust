use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "graphql-codegen-rust")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Generate Rust ORM code from GraphQL schemas")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Increase verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new GraphQL codegen project
    Init {
        /// GraphQL endpoint URL
        #[arg(short, long)]
        url: String,

        /// ORM to generate code for
        #[arg(short, long, value_enum, default_value = "diesel")]
        orm: OrmType,

        /// Database backend
        #[arg(short, long, value_enum, default_value = "sqlite")]
        db: DatabaseType,

        /// Output directory for generated code
        #[arg(long, default_value = "./generated")]
        output: PathBuf,

        /// Additional headers for GraphQL requests (key:value pairs)
        #[arg(short = 'H', long, value_parser = parse_header)]
        headers: Vec<(String, String)>,
    },

    /// Generate code from existing configuration
    Generate {
        /// Config file path (auto-detects codegen.yml or TOML)
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Output directory (overrides config)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, clap::ValueEnum)]
pub enum OrmType {
    Diesel,
    SeaOrm,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, clap::ValueEnum)]
pub enum DatabaseType {
    Sqlite,
    Postgres,
    Mysql,
}

fn parse_header(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid header format '{}'. Headers must be in 'key:value' format.\nExample: --header 'Authorization:Bearer token123'",
            s
        ));
    }
    let key = parts[0].trim();
    let value = parts[1].trim();

    if key.is_empty() {
        return Err("Header key cannot be empty. Format: 'key:value'".to_string());
    }
    if value.is_empty() {
        return Err("Header value cannot be empty. Format: 'key:value'".to_string());
    }

    Ok((key.to_string(), value.to_string()))
}
