use std::path::Path;
use anyhow::Result;
use fs_err as fs;

use crate::logger::Logger;

/// Integration module for working with existing GraphQL Code Generator setups
pub struct Integration;

/// Configuration for the integration process
pub struct IntegrationConfig {
    pub output_dir: std::path::PathBuf,
    pub add_scripts: bool,
    pub force: bool,
}

impl Integration {
    /// Integrate with an existing Tauri + GraphQL Code Generator project
    pub async fn integrate_with_existing_project(
        config: IntegrationConfig,
        logger: &Logger,
    ) -> Result<()> {
        // Check if codegen.yml exists
        let codegen_path = Self::find_codegen_config()?;
        logger.info("Found GraphQL Code Generator configuration");

        // Read existing config
        let contents = fs::read_to_string(&codegen_path)?;
        let mut yaml_config: serde_yaml::Value = serde_yaml::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("Failed to parse existing codegen.yml: {}", e))?;

        // Check if rust_codegen section already exists
        if yaml_config.get("rust_codegen").is_some() && !config.force {
            logger.info("Rust codegen section already exists in codegen.yml");
            logger.info("Use --force to overwrite existing configuration");
        } else {
            // Add rust_codegen section with sensible defaults
            logger.info("Adding rust_codegen section to codegen.yml");
            Self::add_rust_codegen_section(&mut yaml_config, &config.output_dir)?;

            // Write back the updated config
            let updated_yaml = serde_yaml::to_string(&yaml_config)
                .map_err(|e| anyhow::anyhow!("Failed to serialize updated config: {}", e))?;
            fs::write(&codegen_path, updated_yaml)?;
            logger.success("Updated codegen.yml with Rust codegen configuration");
        }

        // Optionally add scripts to package.json
        if config.add_scripts {
            Self::add_scripts_to_package_json(logger)?;
        }

    logger.success("Integration complete!");
    logger.info("Run 'graphql-codegen-rust generate' to generate your Rust database code.");

        Ok(())
    }

    /// Find the GraphQL Code Generator config file
    fn find_codegen_config() -> Result<std::path::PathBuf> {
        let codegen_yml = Path::new("codegen.yml");
        if codegen_yml.exists() {
            return Ok(codegen_yml.to_path_buf());
        }

        let codegen_yaml = Path::new("codegen.yaml");
        if codegen_yaml.exists() {
            return Ok(codegen_yaml.to_path_buf());
        }

        Err(anyhow::anyhow!(
            "No GraphQL Code Generator config found. Expected 'codegen.yml' or 'codegen.yaml' in current directory.\n\nPlease ensure you have GraphQL Code Generator set up first, then run this integration command."
        ))
    }

    /// Add rust_codegen section to YAML config
    fn add_rust_codegen_section(
        yaml_config: &mut serde_yaml::Value,
        output_dir: &Path,
    ) -> Result<()> {
        use serde_yaml::Value;

        let rust_codegen = serde_yaml::Mapping::from_iter([
            ("orm".into(), Value::String("diesel".to_string())),
            ("db".into(), Value::String("sqlite".to_string())),
            ("output_dir".into(), Value::String(output_dir.join("db").to_string_lossy().to_string())),
            ("generate_migrations".into(), Value::Bool(true)),
            ("generate_entities".into(), Value::Bool(true)),
        ]);

        yaml_config["rust_codegen"] = Value::Mapping(rust_codegen);
        Ok(())
    }

    /// Add codegen scripts to package.json
    fn add_scripts_to_package_json(logger: &Logger) -> Result<()> {
        let package_json_path = Path::new("package.json");
        if !package_json_path.exists() {
            logger.warning("package.json not found, skipping script addition");
            return Ok(());
        }

        let contents = fs::read_to_string(package_json_path)?;
        let mut package_json: serde_json::Value = serde_json::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("Failed to parse package.json: {}", e))?;

        // Ensure scripts object exists
        if !package_json["scripts"].is_object() {
            package_json["scripts"] = serde_json::json!({});
        }

        let scripts = package_json["scripts"].as_object_mut().unwrap();

        // Add our scripts (don't overwrite existing ones)
        if !scripts.contains_key("codegen:rust") {
            scripts.insert("codegen:rust".to_string(), serde_json::json!("graphql-codegen-rust generate"));
            logger.info("Added 'codegen:rust' script to package.json");
        }

        if !scripts.contains_key("codegen:all") {
            scripts.insert("codegen:all".to_string(), serde_json::json!("npm run codegen && npm run codegen:rust"));
            logger.info("Added 'codegen:all' script to package.json");
        }

        // Write back
        let updated_json = serde_json::to_string_pretty(&package_json)
            .map_err(|e| anyhow::anyhow!("Failed to serialize package.json: {}", e))?;
        fs::write(package_json_path, updated_json)?;
        logger.success("Updated package.json with new scripts");

        Ok(())
    }

}
