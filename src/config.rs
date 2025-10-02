use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use fs_err as fs;

use crate::cli::{DatabaseType, OrmType};

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
        let contents = fs::read_to_string(path)?;

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
                    "YAML config support not enabled. Rebuild with --features yaml-codegen-config"
                ))
            }
        } else {
            Self::from_toml_str(&contents)
        }
    }

    /// Load config from TOML string
    pub fn from_toml_str(contents: &str) -> anyhow::Result<Self> {
        let config: Config = toml::from_str(contents)?;
        Ok(config)
    }

    /// Load config from YAML string
    #[cfg(feature = "yaml-codegen-config")]
    pub fn from_yaml_str(contents: &str) -> anyhow::Result<Self> {
        let yaml_config: YamlConfig = serde_yaml::from_str(contents)?;

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
            "No config file found. Expected codegen.yml, codegen.yaml, or graphql-codegen-rust.toml"
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
