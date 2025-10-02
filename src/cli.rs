use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "graphql-diesel-sync")]
#[command(version = "0.1.0")]
#[command(about = "Generate Diesel/Sea-ORM code from GraphQL schemas")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new GraphQL sync project
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
        /// Specific types to generate (comma-separated)
        #[arg(short, long)]
        types: Option<Vec<String>>,

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
        return Err("Header must be in key:value format".to_string());
    }
    Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
}
