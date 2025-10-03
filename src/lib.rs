//! # GraphQL Code Generator for Rust ORMs
//!
//! Transform GraphQL schemas into production-ready Rust code for Diesel and Sea-ORM.
//! Automatically generates entities, migrations, and schema definitions from your
//! GraphQL API's introspection.
//!
//! ## Key Features
//!
//! - **Dual ORM Support**: Generate code for both [Diesel](https://diesel.rs) and [Sea-ORM](https://www.sea-ql.org/SeaORM)
//! - **Database Agnostic**: Support for SQLite, PostgreSQL, and MySQL
//! - **Type Safety**: Compile-time guarantees with full GraphQL type mapping
//! - **Migration Ready**: Automatic database migration generation
//! - **Introspection Powered**: Works with any GraphQL API that supports introspection
//! - **Flexible Configuration**: TOML and YAML config support (with feature flag)
//!
//! ## Quick Start
//!
//! ### 1. Install the CLI
//!
//! ```bash
//! cargo install graphql-codegen-rust
//! ```
//!
//! ### 2. Initialize your project
//!
//! ```bash
//! graphql-codegen-rust init --url https://api.example.com/graphql
//! ```
//!
//! ### 3. Generate code
//!
//! ```bash
//! graphql-codegen-rust generate
//! ```
//!
//! ## Library Usage
//!
//! For programmatic use in your Rust applications:
//!
//! ```rust,no_run
//! use graphql_codegen_rust::{CodeGenerator, Config};
//! use graphql_codegen_rust::cli::{OrmType, DatabaseType};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create configuration programmatically
//! let config = Config {
//!     url: "https://api.example.com/graphql".to_string(),
//!     orm: OrmType::Diesel,
//!     db: DatabaseType::Postgres,
//!     output_dir: "./generated".into(),
//!     headers: std::collections::HashMap::new(),
//!     type_mappings: std::collections::HashMap::new(),
//!     scalar_mappings: std::collections::HashMap::new(),
//!     table_naming: Default::default(),
//!     generate_migrations: true,
//!     generate_entities: true,
//! };
//!
//! // Generate code
//! let generator = CodeGenerator::new(&config.orm);
//! generator.generate_from_config(&config).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Configuration
//!
//! ### TOML Configuration (`graphql-codegen-rust.toml`)
//!
//! ```toml
//! url = "https://api.example.com/graphql"
//! orm = "Diesel"
//! db = "Postgres"
//! output_dir = "./generated"
//!
//! [headers]
//! Authorization = "Bearer your-token-here"
//!
//! [type_mappings]
//! # Custom type mappings if needed
//! ```
//!
//! ### YAML Configuration (`codegen.yml`) - *Requires `yaml-codegen-config` feature*
//!
//! ```yaml
//! schema:
//!   url: https://api.example.com/graphql
//!   headers:
//!     Authorization: Bearer your-token-here
//!
//! rust_codegen:
//!   orm: Diesel
//!   db: Postgres
//!   output_dir: ./generated
//! ```
//!
//! ## Generated Code Structure
//!
//! ```text
//! generated/
//! ├── src/
//! │   ├── schema.rs          # Diesel schema definitions
//! │   ├── entities/
//! │   │   ├── user.rs       # Entity structs and implementations
//! │   │   └── post.rs
//! │   └── mod.rs            # Sea-ORM module definitions
//! └── migrations/
//!     └── 0001_create_users_table/
//!         ├── up.sql
//!         └── down.sql
//! ```
//!
//! ## Error Handling
//!
//! The library uses [`anyhow`](https://docs.rs/anyhow) for error handling, providing
//! detailed error messages with context. Common error scenarios:
//!
//! - **Network errors**: GraphQL endpoint unreachable or authentication failures
//! - **Schema errors**: Invalid GraphQL schema or introspection disabled
//! - **Configuration errors**: Missing or invalid configuration files
//! - **Generation errors**: Unsupported GraphQL types or ORM constraints
//!
//! ## Feature Flags
//!
//! - `yaml-codegen-config`: Enable YAML configuration file support
//! - Default features include TOML support and both Diesel and Sea-ORM generators
//!
//! ## Requirements
//!
//! - Rust 1.86+
//! - A GraphQL API that supports introspection
//! - Appropriate database dependencies based on your ORM choice

pub mod cli;
pub mod config;
pub mod generator;
pub mod integration;
pub mod introspection;
pub mod logger;
pub mod parser;

pub use cli::OrmType;
pub use cli::DatabaseType;
pub use config::Config;
pub use generator::create_generator;
pub use logger::Logger;

use std::path::Path;

use fs_err as fs;

/// High-level interface for generating Rust ORM code from GraphQL schemas.
///
/// The `CodeGenerator` provides a unified API for generating code regardless of the
/// underlying ORM (Diesel or Sea-ORM). It handles the complete code generation
/// pipeline: schema introspection, parsing, and code emission.
///
/// ## ORM Support
///
/// - **Diesel**: Generates table schemas, entity structs, and database migrations
/// - **Sea-ORM**: Generates entity models, active records, and migration files
///
/// ## Example
///
/// ```rust,no_run
/// use graphql_codegen_rust::{CodeGenerator, Config};
/// use graphql_codegen_rust::cli::{OrmType, DatabaseType};
///
/// # async fn example() -> anyhow::Result<()> {
/// let config = Config {
///     url: "https://api.example.com/graphql".to_string(),
///     orm: OrmType::Diesel,
///     db: DatabaseType::Postgres,
///     output_dir: "./generated".into(),
///     ..Default::default()
/// };
///
/// let generator = CodeGenerator::new(&config.orm);
/// generator.generate_from_config(&config).await?;
/// # Ok(())
/// # }
/// ```
pub struct CodeGenerator {
    inner: Box<dyn generator::CodeGenerator>,
}

impl CodeGenerator {
    /// Creates a new code generator for the specified ORM type.
    ///
    /// # Parameters
    /// - `orm`: The ORM to generate code for (Diesel or Sea-ORM)
    ///
    /// # Returns
    /// A configured `CodeGenerator` ready to produce ORM code.
    ///
    /// # Example
    /// ```rust
    /// use graphql_codegen_rust::{CodeGenerator, cli::OrmType};
    ///
    /// let generator = CodeGenerator::new(&OrmType::Diesel);
    /// ```
    pub fn new(orm: &cli::OrmType) -> Self {
        Self {
            inner: generator::create_generator(orm),
        }
    }

    /// Generates complete ORM code from a GraphQL configuration.
    ///
    /// This method orchestrates the full code generation pipeline:
    /// 1. Introspects the GraphQL schema from the configured endpoint
    /// 2. Parses the schema into an internal representation
    /// 3. Generates ORM-specific code (entities, migrations, schemas)
    /// 4. Writes generated files to the configured output directory
    ///
    /// # Parameters
    /// - `config`: Complete configuration including GraphQL endpoint, ORM type,
    ///   database settings, and output preferences
    ///
    /// # Returns
    /// - `Ok(())` on successful code generation
    /// - `Err(anyhow::Error)` with detailed context on failure
    ///
    /// # Errors
    /// This method can fail due to:
    /// - Network issues when accessing the GraphQL endpoint
    /// - Authentication failures (invalid headers)
    /// - Schema parsing errors (invalid GraphQL schema)
    /// - File system errors (permission issues, disk space)
    /// - Code generation constraints (unsupported GraphQL types)
    ///
    /// # Example
    /// ```rust,no_run
    /// use graphql_codegen_rust::{CodeGenerator, Config};
    /// use graphql_codegen_rust::cli::{OrmType, DatabaseType};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let config = Config {
    ///     url: "https://api.example.com/graphql".to_string(),
    ///     orm: OrmType::Diesel,
    ///     db: DatabaseType::Postgres,
    ///     output_dir: "./generated".into(),
    ///     ..Default::default()
    /// };
    ///
    /// let generator = CodeGenerator::new(&config.orm);
    /// generator.generate_from_config(&config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_from_config(&self, config: &Config) -> anyhow::Result<()> {
        // Fetch and parse schema
        let parser = parser::GraphQLParser::new();
        let schema = parser
            .parse_from_introspection(&config.url, &config.headers)
            .await?;

        // Create a silent logger for the public API
        let logger = Logger::new(0);

        // Generate all code
        generate_all_code(&schema, config, &*self.inner, &logger).await
    }
}

/// Generates ORM code directly from a configuration file path.
///
/// This is a convenience function that combines configuration loading and code generation
/// into a single call. It automatically detects the configuration format (TOML or YAML)
/// and creates the appropriate code generator.
///
/// # Supported Configuration Formats
///
/// - **TOML**: `graphql-codegen-rust.toml` or any `.toml` file
/// - **YAML**: `codegen.yml`, `codegen.yaml` (requires `yaml-codegen-config` feature)
///
/// # Parameters
/// - `config_path`: Path to the configuration file. Can be any type that converts to `Path`.
///
/// # Returns
/// - `Ok(())` on successful code generation
/// - `Err(anyhow::Error)` with context about what failed
///
/// # Errors
/// This function can fail due to:
/// - Configuration file not found or unreadable
/// - Invalid configuration format or content
/// - Network issues during GraphQL introspection
/// - Code generation failures
///
/// # Example
/// ```rust,no_run
/// use graphql_codegen_rust::generate_from_config_file;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Generate from TOML config
/// generate_from_config_file("graphql-codegen-rust.toml").await?;
///
/// // Generate from YAML config (if feature enabled)
/// generate_from_config_file("codegen.yml").await?;
/// # Ok(())
/// # }
/// ```
pub async fn generate_from_config_file<P: AsRef<Path>>(config_path: P) -> anyhow::Result<()> {
    let path_buf = config_path.as_ref().to_path_buf();
    let config = Config::from_file(&path_buf)?;
    let generator = CodeGenerator::new(&config.orm);
    generator.generate_from_config(&config).await
}

pub async fn generate_all_code(
    schema: &parser::ParsedSchema,
    config: &Config,
    generator: &dyn generator::CodeGenerator,
    logger: &Logger,
) -> anyhow::Result<()> {
    // Create output directory structure
    logger.trace("Creating output directory structure...");
    fs::create_dir_all(&config.output_dir)?;
    let src_dir = config.output_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // Generate schema file
    logger.trace("Generating schema file...");
    let schema_code = generator.generate_schema(schema, config)?;
    if config.orm == cli::OrmType::Diesel {
        let schema_path = src_dir.join("schema.rs");
        fs::write(schema_path, schema_code)?;
        logger.info("Generated schema.rs");
    } else if config.orm == cli::OrmType::SeaOrm {
        // Sea-ORM generates a mod.rs file at the root
        let mod_path = config.output_dir.join("mod.rs");
        fs::write(mod_path, schema_code)?;
        logger.info("Generated mod.rs");
    }

    // Generate entity files
    logger.trace("Generating entity files...");
    let entities = generator.generate_entities(schema, config)?;
    let entities_dir = src_dir.join("entities");
    fs::create_dir_all(&entities_dir)?;

    let entity_count = entities.len();
    for (filename, code) in entities {
        let entity_path = entities_dir.join(filename);
        fs::write(entity_path, code)?;
    }
    logger.info(&format!("Generated {} entity files", entity_count));

    // Generate migrations
    logger.trace("Generating migration files...");
    let migrations = generator.generate_migrations(schema, config)?;
    let migrations_dir = config.output_dir.join("migrations");
    fs::create_dir_all(&migrations_dir)?;

    let migration_count = migrations.len();
    for migration in migrations {
        let migration_dir = migrations_dir.join(&migration.name);
        fs::create_dir_all(&migration_dir)?;

        let up_path = migration_dir.join("up.sql");
        let down_path = migration_dir.join("down.sql");

        fs::write(up_path, migration.up_sql)?;
        fs::write(down_path, migration.down_sql)?;
    }
    logger.info(&format!("Generated {} migrations", migration_count));

    Ok(())
}