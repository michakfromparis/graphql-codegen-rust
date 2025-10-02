use std::collections::HashMap;
use std::path::PathBuf;

use crate::cli::{DatabaseType, OrmType};
use crate::config::Config;
use crate::parser::{ParsedEnum, ParsedField, ParsedSchema, ParsedType};

pub mod diesel;
pub mod sea_orm;

pub trait CodeGenerator {
    fn generate_schema(&self, schema: &ParsedSchema, config: &Config) -> anyhow::Result<String>;
    fn generate_entities(&self, schema: &ParsedSchema, config: &Config) -> anyhow::Result<HashMap<String, String>>;
    fn generate_migrations(&self, schema: &ParsedSchema, config: &Config) -> anyhow::Result<Vec<MigrationFile>>;
}

#[derive(Debug)]
pub struct MigrationFile {
    pub name: String,
    pub up_sql: String,
    pub down_sql: String,
}

pub fn create_generator(orm: &OrmType) -> Box<dyn CodeGenerator> {
    match orm {
        OrmType::Diesel => Box::new(diesel::DieselGenerator::new()),
        OrmType::SeaOrm => Box::new(sea_orm::SeaOrmGenerator::new()),
    }
}

pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_upper = false;

    for (i, ch) in s.char_indices() {
        if ch.is_uppercase() {
            if i > 0 && !prev_was_upper {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
            prev_was_upper = true;
        } else {
            result.push(ch);
            prev_was_upper = false;
        }
    }

    result
}

pub fn rust_type_for_field(field: &ParsedField, db_type: &DatabaseType, scalar_mappings: &HashMap<String, String>) -> String {
    match &field.field_type {
        crate::parser::FieldType::Scalar(scalar_type) => {
            match scalar_type.as_str() {
                "ID" => {
                    match db_type {
                        DatabaseType::Sqlite => "i32".to_string(),
                        DatabaseType::Postgres => "uuid::Uuid".to_string(),
                        DatabaseType::Mysql => "u32".to_string(),
                    }
                }
                "String" => "String".to_string(),
                "Int" => "i32".to_string(),
                "Float" => "f64".to_string(),
                "Boolean" => "bool".to_string(),
                custom => {
                    scalar_mappings.get(custom)
                        .cloned()
                        .unwrap_or_else(|| "String".to_string())
                }
            }
        }
        crate::parser::FieldType::Reference(type_name) => {
            // For references, we'll assume they're other entities
            // In a real implementation, we'd need to handle foreign keys
            match db_type {
                DatabaseType::Sqlite => "i32".to_string(),
                DatabaseType::Postgres => "uuid::Uuid".to_string(),
                DatabaseType::Mysql => "u32".to_string(),
            }
        }
        crate::parser::FieldType::Enum(enum_name) => enum_name.clone(),
    }
}

pub fn diesel_column_type_for_field(field: &ParsedField, db_type: &DatabaseType, scalar_mappings: &HashMap<String, String>) -> String {
    match &field.field_type {
        crate::parser::FieldType::Scalar(scalar_type) => {
            match scalar_type.as_str() {
                "ID" => {
                    match db_type {
                        DatabaseType::Sqlite => "Integer".to_string(),
                        DatabaseType::Postgres => "Uuid".to_string(),
                        DatabaseType::Mysql => "Unsigned<Integer>".to_string(),
                    }
                }
                "String" => "Text".to_string(),
                "Int" => "Integer".to_string(),
                "Float" => "Double".to_string(),
                "Boolean" => "Bool".to_string(),
                custom => {
                    scalar_mappings.get(custom)
                        .cloned()
                        .unwrap_or_else(|| "Text".to_string())
                }
            }
        }
        crate::parser::FieldType::Reference(_) => {
            // Foreign key
            match db_type {
                DatabaseType::Sqlite => "Integer".to_string(),
                DatabaseType::Postgres => "Uuid".to_string(),
                DatabaseType::Mysql => "Unsigned<Integer>".to_string(),
            }
        }
        crate::parser::FieldType::Enum(_) => "Text".to_string(),
    }
}

pub fn sql_type_for_field(field: &ParsedField, db_type: &DatabaseType, scalar_mappings: &HashMap<String, String>) -> String {
    match &field.field_type {
        crate::parser::FieldType::Scalar(scalar_type) => {
            match scalar_type.as_str() {
                "ID" => {
                    match db_type {
                        DatabaseType::Sqlite => "INTEGER".to_string(),
                        DatabaseType::Postgres => "UUID".to_string(),
                        DatabaseType::Mysql => "INT UNSIGNED".to_string(),
                    }
                }
                "String" => "TEXT".to_string(),
                "Int" => "INTEGER".to_string(),
                "Float" => "REAL".to_string(),
                "Boolean" => {
                    match db_type {
                        DatabaseType::Sqlite => "INTEGER".to_string(),
                        DatabaseType::Postgres => "BOOLEAN".to_string(),
                        DatabaseType::Mysql => "TINYINT(1)".to_string(),
                    }
                }
                custom => {
                    scalar_mappings.get(custom)
                        .cloned()
                        .unwrap_or_else(|| "TEXT".to_string())
                }
            }
        }
        crate::parser::FieldType::Reference(_) => {
            // Foreign key
            match db_type {
                DatabaseType::Sqlite => "INTEGER".to_string(),
                DatabaseType::Postgres => "UUID".to_string(),
                DatabaseType::Mysql => "INT UNSIGNED".to_string(),
            }
        }
        crate::parser::FieldType::Enum(_) => "TEXT".to_string(),
    }
}
