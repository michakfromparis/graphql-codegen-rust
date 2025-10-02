use graphql_parser::schema::*;
use std::collections::HashMap;

use crate::introspection::{Introspector, Schema as IntrospectionSchema};

#[derive(Debug, Clone)]
pub struct ParsedSchema {
    pub types: HashMap<String, ParsedType>,
    pub enums: HashMap<String, ParsedEnum>,
    pub scalars: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedType {
    pub name: String,
    pub fields: Vec<ParsedField>,
    pub description: Option<String>,
    pub interfaces: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedField {
    pub name: String,
    pub field_type: FieldType,
    pub description: Option<String>,
    pub is_nullable: bool,
    pub is_list: bool,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    Scalar(String),
    Reference(String),
    Enum(String),
}

#[derive(Debug, Clone)]
pub struct ParsedEnum {
    pub name: String,
    pub values: Vec<String>,
    pub description: Option<String>,
}

pub struct GraphQLParser {
    introspector: Introspector,
}

impl GraphQLParser {
    pub fn new() -> Self {
        Self {
            introspector: Introspector::new(),
        }
    }

    /// Parse schema from introspection
    pub async fn parse_from_introspection(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> anyhow::Result<ParsedSchema> {
        let schema = self.introspector.introspect_schema(url, headers).await?;
        self.parse_schema(schema)
    }

    /// Parse schema from SDL string
    pub fn parse_from_sdl(&self, _sdl: &str) -> anyhow::Result<ParsedSchema> {
        // TODO: Implement SDL parsing
        Err(anyhow::anyhow!("SDL parsing not yet implemented"))
    }

    /// Parse schema from SDL string (simplified version)
    pub fn parse_from_sdl_simple(&self, sdl: &str) -> anyhow::Result<ParsedSchema> {
        // For now, just return introspection-based parsing
        // TODO: Implement proper SDL parsing
        Err(anyhow::anyhow!("SDL parsing not yet implemented, use introspection"))
    }

    fn parse_schema(&self, schema: IntrospectionSchema) -> anyhow::Result<ParsedSchema> {
        let mut types = HashMap::new();
        let mut enums = HashMap::new();
        let mut scalars = Vec::new();

        for type_def in schema.types {
            if let Some(name) = &type_def.name {
                // Skip introspection types and built-in scalars
                if name.starts_with("__") || name == "String" || name == "Int" || name == "Float" || name == "Boolean" || name == "ID" {
                    if matches!(type_def.kind, crate::introspection::TypeKind::Scalar) && !name.starts_with("__") {
                        scalars.push(name.clone());
                    }
                    continue;
                }

                match type_def.kind {
                    crate::introspection::TypeKind::Object => {
                        if let Some(parsed_type) = self.parse_object_type(&type_def) {
                            types.insert(name.clone(), parsed_type);
                        }
                    }
                    crate::introspection::TypeKind::Enum => {
                        if let Some(parsed_enum) = self.parse_enum_type(&type_def) {
                            enums.insert(name.clone(), parsed_enum);
                        }
                    }
                    crate::introspection::TypeKind::Scalar => {
                        scalars.push(name.clone());
                    }
                    _ => {
                        // Skip interfaces, unions, input objects for now
                        // TODO: Add support for these types
                    }
                }
            }
        }

        Ok(ParsedSchema { types, enums, scalars })
    }

    // fn parse_document is removed for now - focusing on introspection
    // TODO: Re-implement SDL parsing when needed

    fn parse_object_type(&self, type_def: &crate::introspection::Type) -> Option<ParsedType> {
        let name = type_def.name.as_ref()?;
        let mut fields = Vec::new();

        if let Some(introspection_fields) = &type_def.fields {
            for field in introspection_fields {
                if let Some(parsed_field) = self.parse_field(&field) {
                    fields.push(parsed_field);
                }
            }
        }

        let interfaces = type_def.interfaces.as_ref()
            .map(|interfaces| {
                interfaces.iter()
                    .filter_map(|i| i.name.clone())
                    .collect()
            })
            .unwrap_or_default();

        Some(ParsedType {
            name: name.clone(),
            fields,
            description: type_def.description.clone(),
            interfaces,
        })
    }

    fn parse_field(&self, field: &crate::introspection::Field) -> Option<ParsedField> {
        let (field_type, is_nullable, is_list) = self.parse_type_ref(&field.type_)?;

        Some(ParsedField {
            name: field.name.clone(),
            field_type,
            description: field.description.clone(),
            is_nullable,
            is_list,
        })
    }

    fn parse_type_ref(&self, type_ref: &crate::introspection::TypeRef) -> Option<(FieldType, bool, bool)> {
        match type_ref.kind {
            Some(crate::introspection::TypeKind::NonNull) => {
                if let Some(of_type) = &type_ref.of_type {
                    let (field_type, _, is_list) = self.parse_type_ref(of_type)?;
                    Some((field_type, false, is_list))
                } else {
                    None
                }
            }
            Some(crate::introspection::TypeKind::List) => {
                if let Some(of_type) = &type_ref.of_type {
                    let (field_type, is_nullable, _) = self.parse_type_ref(of_type)?;
                    Some((field_type, is_nullable, true))
                } else {
                    None
                }
            }
            _ => {
                if let Some(name) = &type_ref.name {
                    let field_type = match name.as_str() {
                        "String" | "Int" | "Float" | "Boolean" => FieldType::Scalar(name.clone()),
                        "ID" => FieldType::Scalar("ID".to_string()),
                        _ => {
                            // Check if it's an enum (this is a simplification)
                            // In a real implementation, we'd need to check the schema
                            FieldType::Reference(name.clone())
                        }
                    };
                    Some((field_type, true, false))
                } else {
                    None
                }
            }
        }
    }

    fn parse_enum_type(&self, type_def: &crate::introspection::Type) -> Option<ParsedEnum> {
        let name = type_def.name.as_ref()?;
        let mut values = Vec::new();

        if let Some(enum_values) = &type_def.enum_values {
            for value in enum_values {
                values.push(value.name.clone());
            }
        }

        Some(ParsedEnum {
            name: name.clone(),
            values,
            description: type_def.description.clone(),
        })
    }

    // SDL parsing functions removed for now - focusing on introspection
    // TODO: Re-implement when needed
}
