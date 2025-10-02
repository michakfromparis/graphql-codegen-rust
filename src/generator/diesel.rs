use std::collections::HashMap;

use crate::cli::DatabaseType;
use crate::config::Config;
use crate::generator::{
    CodeGenerator, MigrationFile, diesel_column_type_for_field, rust_type_for_field,
    sql_type_for_field, to_snake_case,
};
use crate::parser::{ParsedEnum, ParsedSchema, ParsedType};

pub struct DieselGenerator;

impl DieselGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DieselGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for DieselGenerator {
    fn generate_schema(&self, schema: &ParsedSchema, config: &Config) -> anyhow::Result<String> {
        let mut output = String::new();

        // Add imports
        output.push_str("use diesel::prelude::*;\n\n");

        // Generate table! macros for each type
        for (type_name, parsed_type) in &schema.types {
            output.push_str(&self.generate_table_macro(type_name, parsed_type, config)?);
            output.push('\n');
        }

        // Generate enum types if needed
        for (enum_name, parsed_enum) in &schema.enums {
            output.push_str(&self.generate_enum_type(enum_name, parsed_enum)?);
            output.push('\n');
        }

        Ok(output)
    }

    fn generate_entities(
        &self,
        schema: &ParsedSchema,
        config: &Config,
    ) -> anyhow::Result<HashMap<String, String>> {
        let mut entities = HashMap::new();

        for (type_name, parsed_type) in &schema.types {
            let entity_code = self.generate_entity_struct(type_name, parsed_type, config)?;
            entities.insert(format!("{}.rs", to_snake_case(type_name)), entity_code);
        }

        Ok(entities)
    }

    fn generate_migrations(
        &self,
        schema: &ParsedSchema,
        config: &Config,
    ) -> anyhow::Result<Vec<MigrationFile>> {
        let mut migrations = Vec::new();

        for (type_name, parsed_type) in &schema.types {
            let migration = self.generate_table_migration(type_name, parsed_type, config)?;
            migrations.push(migration);
        }

        Ok(migrations)
    }
}

impl DieselGenerator {
    fn generate_table_macro(
        &self,
        type_name: &str,
        parsed_type: &ParsedType,
        config: &Config,
    ) -> anyhow::Result<String> {
        let table_name = to_snake_case(type_name);
        let mut output = format!("table! {{\n    {} (", table_name);

        // Primary key - assume first field named 'id' or add one
        let id_field = parsed_type
            .fields
            .iter()
            .find(|f| f.name == "id")
            .or_else(|| parsed_type.fields.first());

        if let Some(id_field) = id_field {
            output.push_str(&format!("{}\n    ) {{\n", id_field.name));
        } else {
            output.push_str("id\n    ) {\n");
        }

        // Generate columns
        for field in &parsed_type.fields {
            let column_name = to_snake_case(&field.name);
            let column_type =
                diesel_column_type_for_field(field, &config.db, &config.type_mappings);

            let nullable = if field.is_nullable { "" } else { ".not_null()" };
            output.push_str(&format!(
                "        {} -> {}{},\n",
                column_name, column_type, nullable
            ));
        }

        output.push_str("    }\n}\n");
        Ok(output)
    }

    fn generate_entity_struct(
        &self,
        type_name: &str,
        parsed_type: &ParsedType,
        config: &Config,
    ) -> anyhow::Result<String> {
        let struct_name = type_name.to_string();
        let table_name = to_snake_case(type_name);

        let mut output = String::new();

        // Add imports
        output.push_str("#[macro_use]\nextern crate diesel;\n\n");
        output.push_str("use diesel::prelude::*;\n");
        output.push_str(&format!("use super::{}::*;\n\n", table_name));

        // Generate the struct
        output.push_str("#[derive(Queryable, Debug)]\n");
        output.push_str(&format!("pub struct {} {{\n", struct_name));

        for field in &parsed_type.fields {
            let field_name = to_snake_case(&field.name);
            let field_type = rust_type_for_field(field, &config.db, &config.type_mappings);
            output.push_str(&format!("    pub {}: {},\n", field_name, field_type));
        }

        output.push_str("}\n\n");

        // Generate Insertable struct
        output.push_str("#[derive(Insertable)]\n");
        output.push_str(&format!("#[table_name = \"{}\"]\n", table_name));
        output.push_str(&format!("pub struct New{} {{\n", struct_name));

        for field in &parsed_type.fields {
            if field.name != "id" {
                // Skip id for inserts
                let field_name = to_snake_case(&field.name);
                let field_type = rust_type_for_field(field, &config.db, &config.type_mappings);
                output.push_str(&format!("    pub {}: {},\n", field_name, field_type));
            }
        }

        output.push_str("}\n");

        Ok(output)
    }

    fn generate_enum_type(
        &self,
        enum_name: &str,
        parsed_enum: &ParsedEnum,
    ) -> anyhow::Result<String> {
        let mut output = String::new();

        if let Some(description) = &parsed_enum.description {
            output.push_str(&format!("/// {}\n", description));
        }

        output.push_str("#[derive(Debug, Clone, PartialEq, Eq, Hash)]\n");
        output.push_str("#[derive(diesel::deserialize::FromSqlRow, diesel::serialize::ToSql)]\n");
        output.push_str("#[sql_type = \"diesel::sql_types::Text\"]\n");
        output.push_str(&format!("pub enum {} {{\n", enum_name));

        for value in &parsed_enum.values {
            output.push_str(&format!("    {},\n", value));
        }

        output.push_str("}\n");

        Ok(output)
    }

    fn generate_table_migration(
        &self,
        type_name: &str,
        parsed_type: &ParsedType,
        config: &Config,
    ) -> anyhow::Result<MigrationFile> {
        let table_name = to_snake_case(type_name);
        let migration_name = format!("create_{}_table", table_name);

        let mut up_sql = format!("CREATE TABLE {} (\n", table_name);

        let mut columns = Vec::new();

        // Add id column if not present
        let has_id = parsed_type.fields.iter().any(|f| f.name == "id");
        if !has_id {
            let id_type = match config.db {
                DatabaseType::Sqlite => "INTEGER PRIMARY KEY AUTOINCREMENT",
                DatabaseType::Postgres => "UUID PRIMARY KEY DEFAULT gen_random_uuid()",
                DatabaseType::Mysql => "INT UNSIGNED PRIMARY KEY AUTO_INCREMENT",
            };
            columns.push(format!("    id {}", id_type));
        }

        for field in &parsed_type.fields {
            let column_name = to_snake_case(&field.name);
            let sql_type = sql_type_for_field(field, &config.db, &config.type_mappings);

            let nullable = if field.is_nullable { "" } else { " NOT NULL" };
            let primary_key = if field.name == "id" {
                " PRIMARY KEY"
            } else {
                ""
            };

            columns.push(format!(
                "    {} {}{}{}",
                column_name, sql_type, nullable, primary_key
            ));
        }

        up_sql.push_str(&columns.join(",\n"));
        up_sql.push_str("\n);");

        // Add indexes for foreign keys (simplified)
        for field in &parsed_type.fields {
            if let crate::parser::FieldType::Reference(_) = &field.field_type {
                let column_name = to_snake_case(&field.name);
                up_sql.push_str(&format!(
                    "\n\nCREATE INDEX idx_{}_{} ON {} ({});",
                    table_name, column_name, table_name, column_name
                ));
            }
        }

        let down_sql = format!("DROP TABLE {};", table_name);

        Ok(MigrationFile {
            name: migration_name,
            up_sql,
            down_sql,
        })
    }
}
