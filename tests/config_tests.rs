use graphql_rust_codegen::Config;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_config_from_toml_string() {
    let toml_content = r#"
url = "https://api.example.com/graphql"
orm = "Diesel"
db = "Sqlite"
output_dir = "./generated"
generate_migrations = true
generate_entities = true

[headers]
Authorization = "Bearer token123"
"#;

    let config = Config::from_toml_str(toml_content).unwrap();

    assert_eq!(config.url, "https://api.example.com/graphql");
    assert_eq!(config.orm, graphql_rust_codegen::cli::OrmType::Diesel);
    assert_eq!(config.db, graphql_rust_codegen::cli::DatabaseType::Sqlite);
    assert_eq!(config.output_dir, PathBuf::from("./generated"));
    assert_eq!(
        config.headers.get("Authorization"),
        Some(&"Bearer token123".to_string())
    );
    assert!(config.generate_migrations);
    assert!(config.generate_entities);
}

#[cfg(feature = "yaml-codegen-config")]
#[test]
fn test_config_from_yaml_string() {
    let yaml_content = r#"
schema: https://api.example.com/graphql
rust_codegen:
  orm: Diesel
  db: Sqlite
  output_dir: ./generated
  generate_migrations: true
  generate_entities: true
  headers:
    Authorization: "Bearer token123"
"#;

    let config = Config::from_yaml_str(yaml_content).unwrap();

    assert_eq!(config.url, "https://api.example.com/graphql");
    assert_eq!(config.orm, graphql_rust_codegen::cli::OrmType::Diesel);
    assert_eq!(config.db, graphql_rust_codegen::cli::DatabaseType::Sqlite);
    assert_eq!(config.output_dir, PathBuf::from("./generated"));
    assert_eq!(
        config.headers.get("Authorization"),
        Some(&"Bearer token123".to_string())
    );
    assert!(config.generate_migrations);
    assert!(config.generate_entities);
}

#[cfg(feature = "yaml-codegen-config")]
#[test]
fn test_config_yaml_minimal_schema() {
    let yaml_content = r#"
schema: https://api.example.com/graphql
"#;

    let config = Config::from_yaml_str(yaml_content).unwrap();

    assert_eq!(config.url, "https://api.example.com/graphql");
    assert_eq!(config.orm, graphql_rust_codegen::cli::OrmType::Diesel); // default
    assert_eq!(config.db, graphql_rust_codegen::cli::DatabaseType::Sqlite); // default
    assert_eq!(config.output_dir, PathBuf::from("./generated")); // default
    assert!(config.headers.is_empty());
}

#[cfg(feature = "yaml-codegen-config")]
#[test]
fn test_config_yaml_schema_with_headers() {
    let yaml_content = r#"
schema:
  url: https://api.example.com/graphql
  headers:
    Authorization: "Bearer token123"
    X-API-Key: "key456"
"#;

    let config = Config::from_yaml_str(yaml_content).unwrap();

    assert_eq!(config.url, "https://api.example.com/graphql");
    assert_eq!(
        config.headers.get("Authorization"),
        Some(&"Bearer token123".to_string())
    );
    assert_eq!(config.headers.get("X-API-Key"), Some(&"key456".to_string()));
}

#[test]
fn test_config_auto_detect() {
    // This test assumes no config files exist in the test environment
    let result = Config::auto_detect_config();
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No config file found")
    );
}

#[test]
fn test_config_from_file_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.toml");

    let toml_content = r#"
url = "https://api.example.com/graphql"
orm = "SeaOrm"
db = "Postgres"
output_dir = "./custom_output"
generate_migrations = false
generate_entities = true

[headers]
Authorization = "Bearer token123"
"#;

    fs::write(&config_path, toml_content).unwrap();

    let config = Config::from_file(&config_path).unwrap();

    assert_eq!(config.url, "https://api.example.com/graphql");
    assert_eq!(config.orm, graphql_rust_codegen::cli::OrmType::SeaOrm);
    assert_eq!(config.db, graphql_rust_codegen::cli::DatabaseType::Postgres);
    assert_eq!(config.output_dir, PathBuf::from("./custom_output"));
    assert_eq!(
        config.headers.get("Authorization"),
        Some(&"Bearer token123".to_string())
    );
    assert!(!config.generate_migrations);
    assert!(config.generate_entities);
}

#[cfg(feature = "yaml-codegen-config")]
#[test]
fn test_config_from_file_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test.yml");

    let yaml_content = r#"
schema:
  url: https://api.example.com/graphql
  headers:
    Authorization: "Bearer token123"
rust_codegen:
  orm: Diesel
  db: Sqlite
  output_dir: ./yaml_output
  generate_migrations: true
  generate_entities: false
"#;

    fs::write(&config_path, yaml_content).unwrap();

    let config = Config::from_file(&config_path).unwrap();

    assert_eq!(config.url, "https://api.example.com/graphql");
    assert_eq!(config.orm, graphql_rust_codegen::cli::OrmType::Diesel);
    assert_eq!(config.db, graphql_rust_codegen::cli::DatabaseType::Sqlite);
    assert_eq!(config.output_dir, PathBuf::from("./yaml_output"));
    assert_eq!(
        config.headers.get("Authorization"),
        Some(&"Bearer token123".to_string())
    );
    assert!(config.generate_migrations);
    assert!(!config.generate_entities);
}

#[cfg(feature = "yaml-codegen-config")]
#[test]
fn test_config_from_file_auto_detect_yaml() {
    let temp_dir = TempDir::new().unwrap();

    // Change to temp directory for this test
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    let yaml_content = r#"
schema: https://api.example.com/graphql
"#;
    fs::write("codegen.yml", yaml_content).unwrap();

    let config_path = Config::auto_detect_config().unwrap();
    assert_eq!(config_path, PathBuf::from("codegen.yml"));

    let config = Config::from_file(&config_path).unwrap();
    assert_eq!(config.url, "https://api.example.com/graphql");

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}
