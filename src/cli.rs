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

/// Supported ORM frameworks for code generation.
///
/// Each ORM generates different code structures optimized for their respective ecosystems:
/// - **Diesel**: Mature, compile-time SQL safety, macro-heavy approach
/// - **Sea-ORM**: Async-first, runtime SQL building, entity relationships
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, clap::ValueEnum, Default,
)]
pub enum OrmType {
    /// Generates Diesel table schemas, Queryable structs, and Insertable structs.
    /// Best for applications needing maximum compile-time safety and performance.
    #[default]
    Diesel,

    /// Generates Sea-ORM Entity models, ActiveModel structs, and migration files.
    /// Best for async applications with complex relationships and runtime flexibility.
    SeaOrm,
}

/// Supported database backends.
///
/// Each database has different capabilities and type mappings:
/// - **SQLite**: File-based, simple deployment, limited concurrent writes
/// - **PostgreSQL**: Advanced features, JSON support, excellent concurrency
/// - **MySQL**: High performance, wide adoption, good for large datasets
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, clap::ValueEnum, Default,
)]
pub enum DatabaseType {
    /// SQLite database - file-based, ACID compliant, no server required.
    /// Uses INTEGER for IDs, TEXT for strings, REAL for floats.
    #[default]
    Sqlite,

    /// PostgreSQL database - advanced open-source RDBMS.
    /// Uses UUID for IDs, native JSON/JSONB, full-text search, advanced indexing.
    Postgres,

    /// MySQL database - high-performance, widely adopted RDBMS.
    /// Uses INT/UNSIGNED for IDs, VARCHAR/TEXT for strings, various numeric types.
    Mysql,
}

/// Parses a header string in "key:value" format for CLI arguments.
///
/// Used internally by clap to validate and parse header arguments.
/// Headers must be provided as `key:value` pairs with no spaces around the colon.
///
/// # Examples
/// ```rust
/// # fn test_parse_header() {
/// let result = graphql_codegen_rust::cli::parse_header("Authorization:Bearer token123");
/// assert_eq!(result.unwrap(), ("Authorization".to_string(), "Bearer token123".to_string()));
/// # }
/// ```
///
/// # Errors
/// Returns an error if the string doesn't contain exactly one colon separator,
/// or if the key or value would be empty after trimming.
pub fn parse_header(s: &str) -> Result<(String, String), String> {
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
