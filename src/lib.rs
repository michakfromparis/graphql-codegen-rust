//! GraphQL to Rust ORM Code Generator
//!
//! This library provides functionality to generate Rust ORM code (Diesel or Sea-ORM)
//! from GraphQL schema introspection.
//!
//! # Example
//!
//! ```rust,no_run
//! use graphql_codegen_rust::{CodeGenerator, Config};
//! use std::path::PathBuf;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Load configuration
//! let config_path = PathBuf::from("codegen.yml");
//! let config = Config::from_file(&config_path)?;
//!
//! // Generate code
//! let generator = CodeGenerator::new(&config.orm);
//! generator.generate_from_config(&config).await?;
//! # Ok(())
//! # }
//! ```

pub mod cli;
pub mod config;
pub mod generator;
pub mod introspection;
pub mod parser;

pub use config::Config;
pub use generator::create_generator;

use std::path::Path;

/// Main code generator interface
pub struct CodeGenerator {
    inner: Box<dyn generator::CodeGenerator>,
}

impl CodeGenerator {
    /// Create a new code generator for the specified ORM
    pub fn new(orm: &cli::OrmType) -> Self {
        Self {
            inner: generator::create_generator(orm),
        }
    }

    /// Generate code from a configuration
    pub async fn generate_from_config(&self, config: &Config) -> anyhow::Result<()> {
        // Fetch and parse schema
        let parser = parser::GraphQLParser::new();
        let schema = parser
            .parse_from_introspection(&config.url, &config.headers)
            .await?;

        // Generate all code
        generate_all_code(&schema, config, &*self.inner).await
    }
}

/// Convenience function to generate code from a config file
pub async fn generate_from_config_file<P: AsRef<Path>>(config_path: P) -> anyhow::Result<()> {
    let path_buf = config_path.as_ref().to_path_buf();
    let config = Config::from_file(&path_buf)?;
    let generator = CodeGenerator::new(&config.orm);
    generator.generate_from_config(&config).await
}

async fn generate_all_code(
    schema: &parser::ParsedSchema,
    config: &Config,
    generator: &dyn generator::CodeGenerator,
) -> anyhow::Result<()> {
    // Create output directory structure
    std::fs::create_dir_all(&config.output_dir)?;
    let src_dir = config.output_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    // Generate schema file (for Diesel)
    if config.orm == cli::OrmType::Diesel {
        let schema_code = generator.generate_schema(schema, config)?;
        let schema_path = src_dir.join("schema.rs");
        std::fs::write(schema_path, schema_code)?;
        println!("Generated schema.rs");
    }

    // Generate entity files
    let entities = generator.generate_entities(schema, config)?;
    let entities_dir = src_dir.join("entities");
    std::fs::create_dir_all(&entities_dir)?;

    let entity_count = entities.len();
    for (filename, code) in entities {
        let entity_path = entities_dir.join(filename);
        std::fs::write(entity_path, code)?;
    }
    println!("Generated {} entity files", entity_count);

    // Generate migrations
    let migrations = generator.generate_migrations(schema, config)?;
    let migrations_dir = config.output_dir.join("migrations");
    std::fs::create_dir_all(&migrations_dir)?;

    let migration_count = migrations.len();
    for migration in migrations {
        let migration_dir = migrations_dir.join(&migration.name);
        std::fs::create_dir_all(&migration_dir)?;

        let up_path = migration_dir.join("up.sql");
        let down_path = migration_dir.join("down.sql");

        std::fs::write(up_path, migration.up_sql)?;
        std::fs::write(down_path, migration.down_sql)?;
    }
    println!("Generated {} migrations", migration_count);

    Ok(())
}
