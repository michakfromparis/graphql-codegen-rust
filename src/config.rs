use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::cli::{DatabaseType, OrmType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// GraphQL endpoint URL
    pub url: String,

    /// ORM type
    pub orm: OrmType,

    /// Database type
    pub db: DatabaseType,

    /// Output directory
    pub output_dir: PathBuf,

    /// Additional headers for requests
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Custom type mappings
    #[serde(default)]
    pub type_mappings: HashMap<String, String>,

    /// Custom scalar mappings
    #[serde(default)]
    pub scalar_mappings: HashMap<String, String>,

    /// Table naming convention
    #[serde(default)]
    pub table_naming: TableNamingConvention,

    /// Whether to generate migrations
    #[serde(default = "default_true")]
    pub generate_migrations: bool,

    /// Whether to generate entity structs
    #[serde(default = "default_true")]
    pub generate_entities: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableNamingConvention {
    /// Convert GraphQL type names to snake_case (default)
    #[serde(rename = "snake_case")]
    SnakeCase,
    /// Keep GraphQL type names as-is
    #[serde(rename = "pascal_case")]
    PascalCase,
}

impl Default for TableNamingConvention {
    fn default() -> Self {
        TableNamingConvention::SnakeCase
    }
}

impl Config {
    /// Load config from a TOML file
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save config to a TOML file
    pub fn save_to_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        let toml = toml::to_string_pretty(self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }

    /// Get the config file path for a given output directory
    pub fn config_path(output_dir: &PathBuf) -> PathBuf {
        output_dir.join("graphql-diesel-sync.toml")
    }
}

impl From<&crate::cli::Commands> for Config {
    fn from(cmd: &crate::cli::Commands) -> Self {
        match cmd {
            crate::cli::Commands::Init { url, orm, db, output, headers } => {
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
