use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use fs_err as fs;

use crate::cli::OrmType;

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

/// YAML configuration format compatible with GraphQL Code Generator
#[cfg(feature = "yaml-codegen-config")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlConfig {
    /// Schema configuration (shared with GraphQL Code Generator)
    pub schema: SchemaConfig,
    /// Rust codegen specific configuration
    pub rust_codegen: Option<RustCodegenConfig>,
}

/// Schema configuration (compatible with GraphQL Code Generator)
#[cfg(feature = "yaml-codegen-config")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SchemaConfig {
    /// Simple URL string
    Url(String),
    /// Object with URL and headers
    Object {
        /// GraphQL endpoint URL
        url: String,
        /// Additional headers for requests
        #[serde(default)]
        headers: HashMap<String, String>,
    },
}

/// Rust codegen specific configuration
#[cfg(feature = "yaml-codegen-config")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustCodegenConfig {
    /// ORM type
    #[serde(default = "default_orm")]
    pub orm: OrmType,
    /// Database type
    #[serde(default = "default_db")]
    pub db: DatabaseType,
    /// Output directory
    #[serde(default = "default_output")]
    pub output_dir: PathBuf,
    /// Custom type mappings
    #[serde(default)]
    pub type_mappings: HashMap<String, String>,
    /// Custom scalar mappings
    #[serde(default)]
    pub scalar_mappings: HashMap<String, String>,
    /// Table naming convention
    #[serde(default)]
    pub table_naming: TableNamingConvention,
    /// Generate migrations
    #[serde(default = "default_true")]
    pub generate_migrations: bool,
    /// Generate entities
    #[serde(default = "default_true")]
    pub generate_entities: bool,
}

#[cfg(feature = "yaml-codegen-config")]
fn default_orm() -> OrmType {
    OrmType::Diesel
}

#[cfg(feature = "yaml-codegen-config")]
fn default_db() -> DatabaseType {
    DatabaseType::Sqlite
}

#[cfg(feature = "yaml-codegen-config")]
fn default_output() -> PathBuf {
    PathBuf::from("./generated")
}

#[cfg(feature = "yaml-codegen-config")]
impl Default for RustCodegenConfig {
    fn default() -> Self {
        Self {
            orm: default_orm(),
            db: default_db(),
            output_dir: default_output(),
            type_mappings: HashMap::new(),
            scalar_mappings: HashMap::new(),
            table_naming: TableNamingConvention::default(),
            generate_migrations: true,
            generate_entities: true,
        }
    }
}

/// Configuration for GraphQL code generation.
///
/// The `Config` struct defines all parameters needed to generate Rust ORM code
/// from a GraphQL schema. It supports both programmatic creation and loading
/// from configuration files (TOML or YAML).
///
/// ## Required Fields
///
/// - `url`: GraphQL endpoint URL that supports introspection
/// - `orm`: ORM to generate code for (Diesel or Sea-ORM)
/// - `db`: Target database (SQLite, PostgreSQL, or MySQL)
/// - `output_dir`: Directory where generated code will be written
///
/// ## Optional Fields
///
/// All other fields have sensible defaults and are typically configured
/// through configuration files rather than programmatically.
///
/// ## Configuration Files
///
/// ### TOML Format (`graphql-codegen-rust.toml`)
/// ```toml
/// url = "https://api.example.com/graphql"
/// orm = "Diesel"
/// db = "Postgres"
/// output_dir = "./generated"
///
/// [headers]
/// Authorization = "Bearer token"
///
/// [type_mappings]
/// "MyCustomType" = "String"
/// ```
///
/// ### YAML Format (`codegen.yml`) - *Requires `yaml-codegen-config` feature*
/// ```yaml
/// schema:
///   url: https://api.example.com/graphql
///   headers:
///     Authorization: Bearer token
///
/// rust_codegen:
///   orm: Diesel
///   db: Postgres
///   output_dir: ./generated
/// ```
///
/// ## Example
///
/// ```rust
/// use graphql_codegen_rust::{Config, DatabaseType, OrmType};
/// use std::collections::HashMap;
///
/// let config = Config {
///     url: "https://api.example.com/graphql".to_string(),
///     orm: OrmType::Diesel,
///     db: DatabaseType::Postgres,
///     output_dir: "./generated".into(),
///     headers: HashMap::from([
///         ("Authorization".to_string(), "Bearer token".to_string())
///     ]),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// URL of the GraphQL endpoint that supports introspection.
    ///
    /// This must be a GraphQL API that responds to introspection queries.
    /// The endpoint should be accessible and may require authentication headers.
    ///
    /// # Examples
    /// - `"https://api.github.com/graphql"` (GitHub's public API)
    /// - `"https://api.example.com/graphql"` (your custom API)
    /// - `"http://localhost:4000/graphql"` (local development)
    pub url: String,

    /// ORM framework to generate code for.
    ///
    /// Determines the structure and style of generated code:
    /// - `OrmType::Diesel`: Generates table schemas and Queryable structs
    /// - `OrmType::SeaOrm`: Generates Entity models and ActiveModel structs
    pub orm: OrmType,

    /// Target database backend.
    ///
    /// Affects type mappings and SQL generation:
    /// - `DatabaseType::Sqlite`: Uses INTEGER for IDs, TEXT for strings
    /// - `DatabaseType::Postgres`: Uses UUID for IDs, native JSON support
    /// - `DatabaseType::Mysql`: Uses INT for IDs, MEDIUMTEXT for large content
    pub db: DatabaseType,

    /// Directory where generated code will be written.
    ///
    /// The directory will be created if it doesn't exist. Generated files include:
    /// - `src/schema.rs` (Diesel table definitions)
    /// - `src/entities/*.rs` (Entity structs)
    /// - `src/mod.rs` (Sea-ORM module definitions)
    /// - `migrations/` (Database migration files)
    pub output_dir: PathBuf,

    /// Additional HTTP headers to send with GraphQL requests.
    ///
    /// Common headers include authentication tokens, API keys, or content-type specifications.
    /// Headers are sent with both introspection queries and any follow-up requests.
    ///
    /// # Examples
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("Authorization".to_string(), "Bearer token123".to_string());
    /// headers.insert("X-API-Key".to_string(), "key456".to_string());
    /// ```
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Custom type mappings for GraphQL types to Rust types.
    ///
    /// Maps GraphQL type names to custom Rust types. Useful for:
    /// - Custom scalar types (DateTime, UUID, etc.)
    /// - Domain-specific types
    /// - Third-party library types
    ///
    /// If a GraphQL type is not found in this map, default mappings are used
    /// based on the database type and built-in GraphQL scalars.
    ///
    /// # Examples
    /// ```toml
    /// [type_mappings]
    /// "DateTime" = "chrono::DateTime<chrono::Utc>"
    /// "UUID" = "uuid::Uuid"
    /// "Email" = "String"  # Simple string wrapper
    /// ```
    #[serde(default)]
    pub type_mappings: HashMap<String, String>,

    /// Custom scalar type mappings for GraphQL scalars.
    ///
    /// Similar to `type_mappings` but specifically for GraphQL scalar types.
    /// These are applied before the built-in scalar mappings.
    ///
    /// # Examples
    /// ```toml
    /// [scalar_mappings]
    /// "Date" = "chrono::NaiveDate"
    /// "Timestamp" = "i64"
    /// ```
    #[serde(default)]
    pub scalar_mappings: HashMap<String, String>,

    /// Naming convention for database tables and columns.
    ///
    /// Controls how GraphQL type/field names are converted to database identifiers.
    /// - `TableNamingConvention::SnakeCase`: `UserProfile` → `user_profile`
    /// - `TableNamingConvention::PascalCase`: `UserProfile` → `UserProfile`
    ///
    /// SnakeCase is recommended for most databases.
    #[serde(default)]
    pub table_naming: TableNamingConvention,

    /// Whether to generate database migration files.
    ///
    /// When enabled, creates SQL migration files in the `migrations/` directory
    /// that can be applied to set up the database schema. Each GraphQL type
    /// gets its own migration with CREATE TABLE statements.
    ///
    /// Default: `true`
    #[serde(default = "default_true")]
    pub generate_migrations: bool,

    /// Whether to generate Rust entity/model structs.
    ///
    /// When enabled, creates Rust structs that represent the GraphQL types:
    /// - Diesel: `Queryable` structs for reading data
    /// - Sea-ORM: `Model` structs with relationships
    ///
    /// Default: `true`
    #[serde(default = "default_true")]
    pub generate_entities: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TableNamingConvention {
    /// Convert GraphQL type names to snake_case (default)
    #[serde(rename = "snake_case")]
    #[default]
    SnakeCase,
    /// Keep GraphQL type names as-is
    #[serde(rename = "pascal_case")]
    PascalCase,
}

impl Config {
    /// Load config from a file (auto-detects YAML or TOML)
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to read config file '{}': {}\n\nEnsure the file exists and you have read permissions.",
                path.display(),
                e
            )
        })?;

        // Check if it's YAML (starts with schema: or has .yml/.yaml extension)
        if path
            .extension()
            .is_some_and(|ext| ext == "yml" || ext == "yaml")
            || contents.trim().starts_with("schema:")
        {
            #[cfg(feature = "yaml-codegen-config")]
            {
                Self::from_yaml_str(&contents)
            }
            #[cfg(not(feature = "yaml-codegen-config"))]
            {
                Err(anyhow::anyhow!(
                    "YAML config support not enabled.\n\nTo use YAML config files, rebuild with:\n  cargo build --features yaml-codegen-config\n\nAlternatively, use TOML format with 'graphql-codegen-rust.toml'"
                ))
            }
        } else {
            Self::from_toml_str(&contents)
        }
    }

    /// Load config from TOML string
    pub fn from_toml_str(contents: &str) -> anyhow::Result<Self> {
        let config: Config = toml::from_str(contents).map_err(|e| {
            anyhow::anyhow!(
                "Invalid TOML config format: {}\n\nExpected format:\n  url = \"https://api.example.com/graphql\"\n  orm = \"Diesel\"\n  db = \"Sqlite\"\n  output_dir = \"./generated\"\n  [headers]\n  Authorization = \"Bearer <token>\"\n\nSee documentation for complete configuration options.",
                e
            )
        })?;
        Ok(config)
    }

    /// Load config from YAML string
    #[cfg(feature = "yaml-codegen-config")]
    pub fn from_yaml_str(contents: &str) -> anyhow::Result<Self> {
        let yaml_config: YamlConfig = serde_yaml::from_str(contents).map_err(|e| {
            anyhow::anyhow!(
                "Invalid YAML config format: {}\n\nExpected format:\n  schema:\n    url: https://api.example.com/graphql\n    headers:\n      Authorization: Bearer <token>\n  rust_codegen:\n    orm: Diesel\n    db: Sqlite\n    output_dir: ./generated\n\nSee documentation for complete configuration options.",
                e
            )
        })?;

        // Extract schema info
        let (url, headers) = match yaml_config.schema {
            SchemaConfig::Url(url) => (url, HashMap::new()),
            SchemaConfig::Object { url, headers } => (url, headers),
        };

        // Use rust_codegen section if present, otherwise defaults
        let rust_config = yaml_config.rust_codegen.unwrap_or_default();

        Ok(Config {
            url,
            orm: rust_config.orm,
            db: rust_config.db,
            output_dir: rust_config.output_dir,
            headers,
            type_mappings: rust_config.type_mappings,
            scalar_mappings: rust_config.scalar_mappings,
            table_naming: rust_config.table_naming,
            generate_migrations: rust_config.generate_migrations,
            generate_entities: rust_config.generate_entities,
        })
    }

    /// Save config to a TOML file
    pub fn save_to_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        let toml = toml::to_string_pretty(self)?;
        fs::write(path, toml)?;
        Ok(())
    }

    /// Get the config file path for a given output directory
    pub fn config_path(output_dir: &std::path::Path) -> PathBuf {
        output_dir.join("graphql-codegen-rust.toml")
    }

    /// Auto-detect config file in current directory
    pub fn auto_detect_config() -> anyhow::Result<PathBuf> {
        // Try codegen.yml first
        let yaml_path = PathBuf::from("codegen.yml");
        if yaml_path.exists() {
            return Ok(yaml_path);
        }

        // Try codegen.yaml
        let yaml_path = PathBuf::from("codegen.yaml");
        if yaml_path.exists() {
            return Ok(yaml_path);
        }

        // Try TOML file
        let toml_path = PathBuf::from("graphql-codegen-rust.toml");
        if toml_path.exists() {
            return Ok(toml_path);
        }

        Err(anyhow::anyhow!(
            "No config file found in current directory.\n\nExpected one of:\n  - codegen.yml\n  - codegen.yaml\n  - graphql-codegen-rust.toml\n\nTo create a new project, run:\n  graphql-codegen-rust init --url <your-graphql-endpoint>\n\nTo specify a config file explicitly, run:\n  graphql-codegen-rust generate --config <path-to-config>"
        ))
    }
}

impl From<&crate::cli::Commands> for Config {
    fn from(cmd: &crate::cli::Commands) -> Self {
        match cmd {
            crate::cli::Commands::Init {
                url,
                orm,
                db,
                output,
                headers,
            } => {
                let headers_map = headers.iter().cloned().collect();

                Config {
                    url: url.clone(),
                    orm: orm.clone(),
                    db: db.clone(),
                    output_dir: output.clone(),
                    headers: headers_map,
                    type_mappings: HashMap::new(),
                    scalar_mappings: HashMap::new(),
                    table_naming: TableNamingConvention::default(),
                    generate_migrations: true,
                    generate_entities: true,
                }
            }
            _ => unreachable!("Config can only be created from Init command"),
        }
    }
}
