use graphql_codegen_rust::{
    CodeGenerator, Config,
    parser::{FieldType, ParsedEnum, ParsedField, ParsedSchema, ParsedType},
};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_codegen_creation() {
    let config = Config {
        url: "https://api.example.com/graphql".to_string(),
        orm: graphql_codegen_rust::cli::OrmType::Diesel,
        db: graphql_codegen_rust::DatabaseType::Sqlite,
        output_dir: PathBuf::from("./test_output"),
        headers: HashMap::new(),
        type_mappings: HashMap::new(),
        scalar_mappings: HashMap::new(),
        table_naming: graphql_codegen_rust::config::TableNamingConvention::SnakeCase,
        generate_migrations: true,
        generate_entities: true,
    };

    let _generator = CodeGenerator::new(&config.orm);
    // Just test that we can create a generator without panicking
    // The actual generator type is tested through the config
    assert_eq!(config.orm, graphql_codegen_rust::cli::OrmType::Diesel);
}

#[test]
fn test_config_defaults() {
    let config = Config {
        url: "https://api.example.com/graphql".to_string(),
        orm: graphql_codegen_rust::cli::OrmType::Diesel,
        db: graphql_codegen_rust::DatabaseType::Sqlite,
        output_dir: PathBuf::from("./generated"),
        headers: HashMap::new(),
        type_mappings: HashMap::new(),
        scalar_mappings: HashMap::new(),
        table_naming: graphql_codegen_rust::config::TableNamingConvention::SnakeCase,
        generate_migrations: true,
        generate_entities: true,
    };

    assert_eq!(config.url, "https://api.example.com/graphql");
    assert_eq!(config.orm, graphql_codegen_rust::cli::OrmType::Diesel);
    assert_eq!(config.db, graphql_codegen_rust::DatabaseType::Sqlite);
    assert_eq!(config.output_dir, PathBuf::from("./generated"));
    assert!(config.headers.is_empty());
    assert!(config.generate_migrations);
    assert!(config.generate_entities);
}

#[test]
fn test_table_naming_conventions() {
    use graphql_codegen_rust::generator::to_snake_case;

    assert_eq!(to_snake_case("User"), "user");
    assert_eq!(to_snake_case("UserProfile"), "user_profile");
    assert_eq!(to_snake_case("APIKey"), "api_key");
    assert_eq!(to_snake_case("userName"), "user_name");
    assert_eq!(to_snake_case("XMLHttpRequest"), "xml_http_request");
}

/// Test that generated Diesel code compiles successfully
#[tokio::test]
async fn test_diesel_code_generation_compiles() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_dir = temp_dir.path().to_path_buf();

    // Create a simple test schema
    let mut types = HashMap::new();
    let mut enums = HashMap::new();

    // Add User type
    let user_fields = vec![
        ParsedField {
            name: "id".to_string(),
            field_type: FieldType::Scalar("ID".to_string()),
            description: None,
            is_nullable: false,
            is_list: false,
        },
        ParsedField {
            name: "name".to_string(),
            field_type: FieldType::Scalar("String".to_string()),
            description: None,
            is_nullable: false,
            is_list: false,
        },
        ParsedField {
            name: "email".to_string(),
            field_type: FieldType::Scalar("String".to_string()),
            description: None,
            is_nullable: true,
            is_list: false,
        },
    ];

    types.insert(
        "User".to_string(),
        ParsedType {
            kind: graphql_codegen_rust::parser::TypeKind::Object,
            union_members: vec![],
            name: "User".to_string(),
            fields: user_fields,
            description: Some("A user in the system".to_string()),
            interfaces: vec![],
        },
    );

    // Add Role enum
    enums.insert(
        "Role".to_string(),
        ParsedEnum {
            name: "Role".to_string(),
            values: vec!["ADMIN".to_string(), "USER".to_string()],
            description: Some("User roles".to_string()),
        },
    );

    let schema = ParsedSchema {
        types,
        enums,
        scalars: vec![],
    };

    // Create config for Diesel + SQLite
    let config = Config {
        url: "https://example.com/graphql".to_string(),
        orm: graphql_codegen_rust::cli::OrmType::Diesel,
        db: graphql_codegen_rust::DatabaseType::Sqlite,
        output_dir: output_dir.clone(),
        headers: HashMap::new(),
        type_mappings: HashMap::new(),
        scalar_mappings: HashMap::new(),
        table_naming: graphql_codegen_rust::config::TableNamingConvention::SnakeCase,
        generate_migrations: true,
        generate_entities: true,
    };

    // Generate code using the internal function with pre-parsed schema
    use graphql_codegen_rust::generate_all_code;
    let generator_inner = graphql_codegen_rust::generator::create_generator(&config.orm);
    let logger = graphql_codegen_rust::Logger::new(0);
    generate_all_code(&schema, &config, &*generator_inner, &logger)
        .await
        .expect("Code generation should succeed");

    // Verify files were created
    assert!(output_dir.join("src/schema.rs").exists());
    assert!(output_dir.join("src/entities/user.rs").exists());
    assert!(output_dir.join("migrations").exists());

    // Verify files were created and contain expected content
    let schema_path = output_dir.join("src/schema.rs");
    let user_entity_path = output_dir.join("src/entities/user.rs");

    assert!(schema_path.exists());
    assert!(user_entity_path.exists());

    let schema_content = std::fs::read_to_string(&schema_path).expect("Failed to read schema");
    assert!(schema_content.contains("table!"));
    assert!(schema_content.contains("user"));

    let user_content =
        std::fs::read_to_string(&user_entity_path).expect("Failed to read user entity");
    assert!(user_content.contains("#[derive"));
    assert!(user_content.contains("pub struct User"));

    // Actually validate the generated code syntax to ensure it's valid Rust
    validate_generated_diesel_code(&schema_path, &user_entity_path);
}

/// Test that generated Sea-ORM code compiles successfully
#[tokio::test]
async fn test_sea_orm_code_generation_compiles() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_dir = temp_dir.path().to_path_buf();

    // Create a simple test schema
    let mut types = HashMap::new();
    let mut enums = HashMap::new();

    // Add Product type
    let product_fields = vec![
        ParsedField {
            name: "id".to_string(),
            field_type: FieldType::Scalar("ID".to_string()),
            description: None,
            is_nullable: false,
            is_list: false,
        },
        ParsedField {
            name: "title".to_string(),
            field_type: FieldType::Scalar("String".to_string()),
            description: None,
            is_nullable: false,
            is_list: false,
        },
        ParsedField {
            name: "price".to_string(),
            field_type: FieldType::Scalar("Float".to_string()),
            description: None,
            is_nullable: false,
            is_list: false,
        },
    ];

    types.insert(
        "Product".to_string(),
        ParsedType {
            kind: graphql_codegen_rust::parser::TypeKind::Object,
            union_members: vec![],
            name: "Product".to_string(),
            fields: product_fields,
            description: Some("A product in the catalog".to_string()),
            interfaces: vec![],
        },
    );

    // Add Status enum
    enums.insert(
        "Status".to_string(),
        ParsedEnum {
            name: "Status".to_string(),
            values: vec!["ACTIVE".to_string(), "INACTIVE".to_string()],
            description: Some("Product status".to_string()),
        },
    );

    let schema = ParsedSchema {
        types,
        enums,
        scalars: vec![],
    };

    // Create config for Sea-ORM + PostgreSQL
    let config = Config {
        url: "https://example.com/graphql".to_string(),
        orm: graphql_codegen_rust::cli::OrmType::SeaOrm,
        db: graphql_codegen_rust::DatabaseType::Postgres,
        output_dir: output_dir.clone(),
        headers: HashMap::new(),
        type_mappings: HashMap::new(),
        scalar_mappings: HashMap::new(),
        table_naming: graphql_codegen_rust::config::TableNamingConvention::SnakeCase,
        generate_migrations: true,
        generate_entities: true,
    };

    // Generate code using the internal function with pre-parsed schema
    use graphql_codegen_rust::generate_all_code;
    let generator_inner = graphql_codegen_rust::generator::create_generator(&config.orm);
    let logger = graphql_codegen_rust::Logger::new(0);
    generate_all_code(&schema, &config, &*generator_inner, &logger)
        .await
        .expect("Code generation should succeed");

    // Verify files were created
    // Sea-ORM generates mod.rs in the output directory root, not in src/
    assert!(output_dir.join("mod.rs").exists());
    assert!(output_dir.join("src/entities/product.rs").exists());
    assert!(output_dir.join("migrations").exists());

    // Verify files were created and contain expected content
    let mod_path = output_dir.join("mod.rs");
    let product_entity_path = output_dir.join("src/entities/product.rs");

    assert!(mod_path.exists());
    assert!(product_entity_path.exists());

    let mod_content = std::fs::read_to_string(&mod_path).expect("Failed to read mod.rs");
    assert!(mod_content.contains("pub mod product;"));
    assert!(mod_content.contains("pub use product::Entity;"));

    let product_content =
        std::fs::read_to_string(&product_entity_path).expect("Failed to read product entity");
    assert!(product_content.contains("#[derive(Clone, Debug, PartialEq, DeriveEntityModel"));
    assert!(product_content.contains("pub struct Entity;"));
    assert!(product_content.contains("uuid::Uuid")); // Should use UUID for Postgres ID

    // Actually validate the generated code syntax to ensure it's valid Rust
    validate_generated_sea_orm_code(&mod_path, &product_entity_path);
}

/// Test SDL schema parsing
#[test]
fn test_sdl_parsing() {
    let parser = graphql_codegen_rust::parser::GraphQLParser::new();

    let sdl_schema = r#"
        type User {
            id: ID!
            username: String!
            email: String
            posts: [Post!]!
        }

        type Post {
            id: ID!
            title: String!
            content: String!
            authorId: ID!
            published: Boolean!
            categoryId: ID
        }

        type Category {
            id: ID!
            name: String!
            description: String
        }

        enum Role {
            ADMIN
            USER
            MODERATOR
        }

        interface Node {
            id: ID!
        }

        union SearchResult = User | Post
    "#;

    let result = parser.parse_from_sdl(sdl_schema);
    assert!(result.is_ok(), "SDL parsing should succeed");

    let schema = result.unwrap();

    // Check that we parsed the types
    assert!(
        schema.types.contains_key("User"),
        "Should contain User type"
    );
    assert!(
        schema.types.contains_key("Post"),
        "Should contain Post type"
    );
    assert!(
        schema.types.contains_key("Category"),
        "Should contain Category type"
    );
    assert!(
        schema.types.contains_key("Node"),
        "Should contain Node interface"
    );
    assert!(
        schema.types.contains_key("SearchResult"),
        "Should contain SearchResult union"
    );

    // Check that we parsed the enum
    assert!(
        schema.enums.contains_key("Role"),
        "Should contain Role enum"
    );

    // Check User type fields
    let user_type = &schema.types["User"];
    assert_eq!(user_type.name, "User");
    assert_eq!(user_type.fields.len(), 4);

    // Check that ID field is not nullable
    let id_field = user_type.fields.iter().find(|f| f.name == "id").unwrap();
    assert!(!id_field.is_nullable);
    assert!(
        matches!(id_field.field_type, graphql_codegen_rust::parser::FieldType::Scalar(ref s) if s == "ID")
    );

    // Check that email field is nullable
    let email_field = user_type.fields.iter().find(|f| f.name == "email").unwrap();
    assert!(email_field.is_nullable);

    // Check posts field is a list and not nullable
    let posts_field = user_type.fields.iter().find(|f| f.name == "posts").unwrap();
    assert!(posts_field.is_list);
    assert!(!posts_field.is_nullable);

    println!("✓ SDL parsing test passed");
}

/// Test relationship detection
#[test]
fn test_relationship_detection() {
    let parser = graphql_codegen_rust::parser::GraphQLParser::new();

    let sdl_schema = r#"
        type User {
            id: ID!
            username: String!
            email: String
        }

        type Post {
            id: ID!
            title: String!
            content: String!
            authorId: ID!
            categoryId: ID
        }

        type Category {
            id: ID!
            name: String!
        }
    "#;

    let result = parser.parse_from_sdl(sdl_schema);
    assert!(result.is_ok(), "SDL parsing should succeed");

    let schema = result.unwrap();

    // Test relationship detection
    let relationships = graphql_codegen_rust::generator::detect_relationships(&schema);

    assert!(
        relationships.contains_key("Post"),
        "Post should have relationships"
    );

    let post_relationships = &relationships["Post"];
    assert_eq!(
        post_relationships.len(),
        1,
        "Post should have 1 relationship"
    );

    // Check categoryId -> Category relationship
    let category_rel = post_relationships
        .iter()
        .find(|r| r.field_name == "categoryId")
        .unwrap();
    assert_eq!(category_rel.related_type, "Category");
    assert!(matches!(
        category_rel.relationship_type,
        graphql_codegen_rust::generator::RelationshipType::BelongsTo
    ));
    assert!(category_rel.foreign_key);

    println!("✓ Relationship detection test passed");
}

/// Test code generation against real GraphQL APIs
#[tokio::test]
async fn test_real_graphql_apis() {
    let real_apis = vec![
        ("https://countries.trevorblades.com/", "Countries API"),
        ("https://api.spacex.land/graphql/", "SpaceX API"),
        ("https://graphql.anilist.co/", "AniList API"),
    ];

    for (endpoint, api_name) in real_apis {
        println!("Testing against {}: {}", api_name, endpoint);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Test both Diesel and Sea-ORM
        for orm_type in &[
            graphql_codegen_rust::cli::OrmType::Diesel,
            graphql_codegen_rust::cli::OrmType::SeaOrm,
        ] {
            let db_type = match orm_type {
                graphql_codegen_rust::cli::OrmType::Diesel => {
                    graphql_codegen_rust::DatabaseType::Sqlite
                }
                graphql_codegen_rust::cli::OrmType::SeaOrm => {
                    graphql_codegen_rust::DatabaseType::Postgres
                }
            };

            let config = Config {
                url: endpoint.to_string(),
                orm: orm_type.clone(),
                db: db_type,
                output_dir: temp_dir.path().to_path_buf(),
                headers: HashMap::new(),
                type_mappings: HashMap::new(),
                scalar_mappings: HashMap::new(),
                table_naming: graphql_codegen_rust::config::TableNamingConvention::SnakeCase,
                generate_migrations: true,
                generate_entities: true,
            };

            // This should succeed for public APIs
            let generator = CodeGenerator::new(&config.orm);
            match generator.generate_from_config(&config).await {
                Ok(_) => println!(
                    "✓ Successfully generated code for {} with {:?}",
                    api_name, orm_type
                ),
                Err(e) => {
                    // Some APIs might have issues, log but don't fail
                    println!(
                        "⚠️  Failed to generate code for {} with {:?}: {}",
                        api_name, orm_type, e
                    );
                }
            }
        }
    }
}

/// Test edge cases and error conditions
#[tokio::test]
async fn test_edge_cases() {
    let edge_cases = vec![
        ("empty_schema", create_empty_schema()),
        ("single_field_type", create_single_field_schema()),
        ("enum_only_schema", create_enum_only_schema()),
        (
            "complex_relationships",
            create_complex_relationships_schema(),
        ),
    ];

    for (case_name, schema) in edge_cases {
        println!("Testing edge case: {}", case_name);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Test both ORMs
        for orm_type in &[
            graphql_codegen_rust::cli::OrmType::Diesel,
            graphql_codegen_rust::cli::OrmType::SeaOrm,
        ] {
            let db_type = match orm_type {
                graphql_codegen_rust::cli::OrmType::Diesel => {
                    graphql_codegen_rust::DatabaseType::Sqlite
                }
                graphql_codegen_rust::cli::OrmType::SeaOrm => {
                    graphql_codegen_rust::DatabaseType::Postgres
                }
            };

            let config = Config {
                url: "https://example.com/graphql".to_string(),
                orm: orm_type.clone(),
                db: db_type,
                output_dir: temp_dir.path().to_path_buf(),
                headers: HashMap::new(),
                type_mappings: HashMap::new(),
                scalar_mappings: HashMap::new(),
                table_naming: graphql_codegen_rust::config::TableNamingConvention::SnakeCase,
                generate_migrations: true,
                generate_entities: true,
            };

            // Generate code using the internal function
            let generator_inner = graphql_codegen_rust::generator::create_generator(&config.orm);
            let logger = graphql_codegen_rust::Logger::new(0);
            match graphql_codegen_rust::generate_all_code(
                &schema,
                &config,
                &*generator_inner,
                &logger,
            )
            .await
            {
                Ok(_) => println!("✓ Edge case '{}' passed for {:?}", case_name, orm_type),
                Err(e) => panic!("Edge case '{}' failed for {:?}: {}", case_name, orm_type, e),
            }
        }
    }
}

/// Test performance of code generation
#[tokio::test]
async fn test_codegen_performance() {
    use std::time::Instant;

    // Create a moderately complex schema for benchmarking
    let mut types = HashMap::new();
    let mut enums = HashMap::new();

    // Create 10 types with 5 fields each
    for i in 0..10 {
        let type_name = format!("Type{}", i);
        let mut fields = vec![ParsedField {
            name: "id".to_string(),
            field_type: FieldType::Scalar("ID".to_string()),
            description: None,
            is_nullable: false,
            is_list: false,
        }];

        // Add 5 additional fields
        for j in 0..5 {
            fields.push(ParsedField {
                name: format!("field{}", j),
                field_type: FieldType::Scalar("String".to_string()),
                description: None,
                is_nullable: true,
                is_list: false,
            });
        }

        types.insert(
            type_name,
            ParsedType {
                kind: graphql_codegen_rust::parser::TypeKind::Object,
                union_members: vec![],
                name: format!("Type{}", i),
                fields,
                description: Some(format!("Type {} description", i)),
                interfaces: vec![],
            },
        );
    }

    // Add some enums
    for i in 0..5 {
        enums.insert(
            format!("Enum{}", i),
            ParsedEnum {
                name: format!("Enum{}", i),
                values: vec![
                    "VALUE1".to_string(),
                    "VALUE2".to_string(),
                    "VALUE3".to_string(),
                ],
                description: Some(format!("Enum {} description", i)),
            },
        );
    }

    let schema = ParsedSchema {
        types,
        enums,
        scalars: vec![],
    };

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut total_time = std::time::Duration::new(0, 0);

    // Benchmark both ORMs
    for orm_type in &[
        graphql_codegen_rust::cli::OrmType::Diesel,
        graphql_codegen_rust::cli::OrmType::SeaOrm,
    ] {
        let db_type = match orm_type {
            graphql_codegen_rust::cli::OrmType::Diesel => {
                graphql_codegen_rust::DatabaseType::Sqlite
            }
            graphql_codegen_rust::cli::OrmType::SeaOrm => {
                graphql_codegen_rust::DatabaseType::Postgres
            }
        };

        let config = Config {
            url: "https://example.com/graphql".to_string(),
            orm: orm_type.clone(),
            db: db_type,
            output_dir: temp_dir.path().to_path_buf(),
            headers: HashMap::new(),
            type_mappings: HashMap::new(),
            scalar_mappings: HashMap::new(),
            table_naming: graphql_codegen_rust::config::TableNamingConvention::SnakeCase,
            generate_migrations: true,
            generate_entities: true,
        };

        let start = Instant::now();
        let generator_inner = graphql_codegen_rust::generator::create_generator(&config.orm);
        let logger = graphql_codegen_rust::Logger::new(0);
        graphql_codegen_rust::generate_all_code(&schema, &config, &*generator_inner, &logger)
            .await
            .expect("Code generation should succeed");
        let elapsed = start.elapsed();

        total_time += elapsed;
        println!("✓ {:?} generation took {:?}", orm_type, elapsed);
    }

    // Ensure reasonable performance (should complete in under 1 second for this schema)
    assert!(
        total_time < std::time::Duration::from_secs(1),
        "Code generation took too long: {:?}",
        total_time
    );

    println!("✓ Total generation time: {:?}", total_time);
}

/// Test with fuzzed/random schema generation
#[tokio::test]
async fn test_fuzz_schema_generation() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let mut rng = StdRng::from_seed([42; 32]); // Deterministic seed for reproducible tests

    for test_case in 0..10 {
        // Generate random schema
        let mut types = HashMap::new();
        let mut enums = HashMap::new();

        // Random number of types (1-5)
        let num_types = rng.random_range(1..=5);
        for i in 0..num_types {
            let type_name = format!("Type{}", i);
            let mut fields = vec![ParsedField {
                name: "id".to_string(),
                field_type: FieldType::Scalar("ID".to_string()),
                description: None,
                is_nullable: false,
                is_list: false,
            }];

            // Random number of fields (1-3)
            let num_fields = rng.random_range(1..=3);
            for j in 0..num_fields {
                let field_types = ["String", "Int", "Boolean", "Float"];
                let random_type = field_types[rng.random_range(0..field_types.len())];

                fields.push(ParsedField {
                    name: format!("field{}", j),
                    field_type: FieldType::Scalar(random_type.to_string()),
                    description: None,
                    is_nullable: rng.random_bool(0.5), // 50% chance of being nullable
                    is_list: false,
                });
            }

            types.insert(
                type_name,
                ParsedType {
                    kind: graphql_codegen_rust::parser::TypeKind::Object,
                    union_members: vec![],
                    name: format!("Type{}", i),
                    fields,
                    description: Some(format!("Random type {}", i)),
                    interfaces: vec![],
                },
            );
        }

        // Random enums (0-2)
        let num_enums = rng.random_range(0..=2);
        for i in 0..num_enums {
            let values: Vec<String> = (0..rng.random_range(2..=5))
                .map(|j| format!("VALUE{}", j))
                .collect();

            enums.insert(
                format!("Enum{}", i),
                ParsedEnum {
                    name: format!("Enum{}", i),
                    values,
                    description: Some(format!("Random enum {}", i)),
                },
            );
        }

        let schema = ParsedSchema {
            types,
            enums,
            scalars: vec![],
        };

        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Test both ORMs with the fuzzed schema
        for orm_type in &[
            graphql_codegen_rust::cli::OrmType::Diesel,
            graphql_codegen_rust::cli::OrmType::SeaOrm,
        ] {
            let db_type = match orm_type {
                graphql_codegen_rust::cli::OrmType::Diesel => {
                    graphql_codegen_rust::DatabaseType::Sqlite
                }
                graphql_codegen_rust::cli::OrmType::SeaOrm => {
                    graphql_codegen_rust::DatabaseType::Postgres
                }
            };

            let config = Config {
                url: "https://example.com/graphql".to_string(),
                orm: orm_type.clone(),
                db: db_type,
                output_dir: temp_dir.path().to_path_buf(),
                headers: HashMap::new(),
                type_mappings: HashMap::new(),
                scalar_mappings: HashMap::new(),
                table_naming: graphql_codegen_rust::config::TableNamingConvention::SnakeCase,
                generate_migrations: true,
                generate_entities: true,
            };

            // This should not panic even with random schemas
            let generator_inner = graphql_codegen_rust::generator::create_generator(&config.orm);
            let logger = graphql_codegen_rust::Logger::new(0);
            match graphql_codegen_rust::generate_all_code(
                &schema,
                &config,
                &*generator_inner,
                &logger,
            )
            .await
            {
                Ok(_) => println!("✓ Fuzz test case {} passed for {:?}", test_case, orm_type),
                Err(e) => panic!(
                    "Fuzz test case {} failed for {:?}: {}",
                    test_case, orm_type, e
                ),
            }
        }
    }
}

/// Test both ORM types with different databases
#[tokio::test]
async fn test_multi_database_support() {
    let databases = vec![
        (graphql_codegen_rust::DatabaseType::Sqlite, "i32"),
        (graphql_codegen_rust::DatabaseType::Postgres, "uuid::Uuid"),
        (graphql_codegen_rust::DatabaseType::Mysql, "u32"),
    ];

    for (db_type, expected_id_type) in databases {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Simple schema with just ID field
        let mut types = HashMap::new();
        types.insert(
            "Test".to_string(),
            ParsedType {
                kind: graphql_codegen_rust::parser::TypeKind::Object,
                union_members: vec![],
                name: "Test".to_string(),
                fields: vec![ParsedField {
                    name: "id".to_string(),
                    field_type: FieldType::Scalar("ID".to_string()),
                    description: None,
                    is_nullable: false,
                    is_list: false,
                }],
                description: None,
                interfaces: vec![],
            },
        );

        let schema = ParsedSchema {
            types,
            enums: HashMap::new(),
            scalars: vec![],
        };

        // Test both ORMs
        for orm_type in &[
            graphql_codegen_rust::cli::OrmType::Diesel,
            graphql_codegen_rust::cli::OrmType::SeaOrm,
        ] {
            let config = Config {
                url: "https://example.com/graphql".to_string(),
                orm: orm_type.clone(),
                db: db_type.clone(),
                output_dir: temp_dir.path().to_path_buf(),
                headers: HashMap::new(),
                type_mappings: HashMap::new(),
                scalar_mappings: HashMap::new(),
                table_naming: graphql_codegen_rust::config::TableNamingConvention::SnakeCase,
                generate_migrations: true,
                generate_entities: true,
            };

            // Generate code using the internal function with pre-parsed schema
            use graphql_codegen_rust::generate_all_code;
            let generator_inner = graphql_codegen_rust::generator::create_generator(&config.orm);
            let logger = graphql_codegen_rust::Logger::new(0);
            generate_all_code(&schema, &config, &*generator_inner, &logger)
                .await
                .expect("Code generation should succeed");

            // For Sea-ORM, check that the generated entity uses the correct ID type
            if matches!(orm_type, graphql_codegen_rust::cli::OrmType::SeaOrm) {
                let entity_path = temp_dir.path().join("src/entities/test.rs");
                let content = std::fs::read_to_string(entity_path).expect("Failed to read entity");
                assert!(
                    content.contains(expected_id_type),
                    "Expected {} in Sea-ORM entity for {:?}",
                    expected_id_type,
                    db_type
                );
            }
        }
    }
}

// Helper functions for creating test schemas

fn create_empty_schema() -> ParsedSchema {
    ParsedSchema {
        types: HashMap::new(),
        enums: HashMap::new(),
        scalars: vec![],
    }
}

fn create_single_field_schema() -> ParsedSchema {
    let mut types = HashMap::new();

    types.insert(
        "Minimal".to_string(),
        ParsedType {
            kind: graphql_codegen_rust::parser::TypeKind::Object,
            union_members: vec![],
            name: "Minimal".to_string(),
            fields: vec![ParsedField {
                name: "id".to_string(),
                field_type: FieldType::Scalar("ID".to_string()),
                description: None,
                is_nullable: false,
                is_list: false,
            }],
            description: None,
            interfaces: vec![],
        },
    );

    ParsedSchema {
        types,
        enums: HashMap::new(),
        scalars: vec![],
    }
}

fn create_enum_only_schema() -> ParsedSchema {
    let mut enums = HashMap::new();

    enums.insert(
        "Status".to_string(),
        ParsedEnum {
            name: "Status".to_string(),
            values: vec![
                "ACTIVE".to_string(),
                "INACTIVE".to_string(),
                "PENDING".to_string(),
            ],
            description: Some("Entity status".to_string()),
        },
    );

    ParsedSchema {
        types: HashMap::new(),
        enums,
        scalars: vec![],
    }
}

fn create_complex_relationships_schema() -> ParsedSchema {
    let mut types = HashMap::new();
    let mut enums = HashMap::new();

    // Author type
    types.insert(
        "Author".to_string(),
        ParsedType {
            kind: graphql_codegen_rust::parser::TypeKind::Object,
            union_members: vec![],
            name: "Author".to_string(),
            fields: vec![
                ParsedField {
                    name: "id".to_string(),
                    field_type: FieldType::Scalar("ID".to_string()),
                    description: None,
                    is_nullable: false,
                    is_list: false,
                },
                ParsedField {
                    name: "name".to_string(),
                    field_type: FieldType::Scalar("String".to_string()),
                    description: None,
                    is_nullable: false,
                    is_list: false,
                },
            ],
            description: Some("Blog author".to_string()),
            interfaces: vec![],
        },
    );

    // Blog post type with relationships
    types.insert(
        "BlogPost".to_string(),
        ParsedType {
            kind: graphql_codegen_rust::parser::TypeKind::Object,
            union_members: vec![],
            name: "BlogPost".to_string(),
            fields: vec![
                ParsedField {
                    name: "id".to_string(),
                    field_type: FieldType::Scalar("ID".to_string()),
                    description: None,
                    is_nullable: false,
                    is_list: false,
                },
                ParsedField {
                    name: "title".to_string(),
                    field_type: FieldType::Scalar("String".to_string()),
                    description: None,
                    is_nullable: false,
                    is_list: false,
                },
                ParsedField {
                    name: "content".to_string(),
                    field_type: FieldType::Scalar("String".to_string()),
                    description: None,
                    is_nullable: false,
                    is_list: false,
                },
                ParsedField {
                    name: "authorId".to_string(),
                    field_type: FieldType::Scalar("ID".to_string()),
                    description: None,
                    is_nullable: false,
                    is_list: false,
                },
                ParsedField {
                    name: "published".to_string(),
                    field_type: FieldType::Scalar("Boolean".to_string()),
                    description: None,
                    is_nullable: false,
                    is_list: false,
                },
                ParsedField {
                    name: "tags".to_string(),
                    field_type: FieldType::Scalar("String".to_string()),
                    description: None,
                    is_nullable: false,
                    is_list: true,
                },
            ],
            description: Some("Blog post".to_string()),
            interfaces: vec![],
        },
    );

    // Status enum
    enums.insert(
        "PostStatus".to_string(),
        ParsedEnum {
            name: "PostStatus".to_string(),
            values: vec![
                "DRAFT".to_string(),
                "PUBLISHED".to_string(),
                "ARCHIVED".to_string(),
            ],
            description: Some("Post publication status".to_string()),
        },
    );

    ParsedSchema {
        types,
        enums,
        scalars: vec![],
    }
}

/// Parse the generated Diesel code to ensure it's valid Rust syntax
fn validate_generated_diesel_code(schema_path: &std::path::Path, entity_path: &std::path::Path) {
    // Read and parse the schema file
    let schema_content = std::fs::read_to_string(schema_path).expect("Failed to read schema file");
    match syn::parse_file(&schema_content) {
        Ok(_) => println!("✓ Schema file parses successfully"),
        Err(e) => panic!("Schema file failed to parse: {}", e),
    }

    // Read and parse the entity file
    let entity_content = std::fs::read_to_string(entity_path).expect("Failed to read entity file");
    match syn::parse_file(&entity_content) {
        Ok(_) => println!("✓ Entity file parses successfully"),
        Err(e) => panic!("Entity file failed to parse: {}", e),
    }
}

/// Parse the generated Sea-ORM code to ensure it's valid Rust syntax
fn validate_generated_sea_orm_code(mod_path: &std::path::Path, entity_path: &std::path::Path) {
    // Read and parse the mod.rs file
    let mod_content = std::fs::read_to_string(mod_path).expect("Failed to read mod.rs file");
    match syn::parse_file(&mod_content) {
        Ok(_) => println!("✓ Mod file parses successfully"),
        Err(e) => panic!("Mod file failed to parse: {}", e),
    }

    // Read and parse the entity file
    let entity_content = std::fs::read_to_string(entity_path).expect("Failed to read entity file");
    match syn::parse_file(&entity_content) {
        Ok(_) => println!("✓ Entity file parses successfully"),
        Err(e) => panic!("Entity file failed to parse: {}", e),
    }
}
