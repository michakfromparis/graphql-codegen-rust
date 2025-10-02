# 🤝 Contributing

Welcome! We're excited to have you contribute to GraphQL Rust Codegen. This guide covers development setup, contribution guidelines, and our development workflow.

## 🚀 Quick Start for Contributors

### Development Setup

1. **Clone and setup:**
```bash
git clone https://github.com/yourusername/graphql-codegen-rust.git
cd graphql-codegen-rust
make setup  # Installs all development tools
```

2. **Verify setup:**
```bash
make dev    # Runs full development workflow
```

3. **Start developing:**
```bash
make watch  # Auto-rebuild on changes
# or
cargo run -- --help  # Test your changes
```

### Development Workflow

We use a comprehensive Makefile for development:

```bash
# Full development cycle
make dev          # Format, lint, test, and document

# Individual tasks
make fmt          # Format code
make lint         # Run clippy (treats warnings as errors)
make test         # Run test suite
make doc          # Generate documentation

# Advanced workflows
make ci           # Full CI pipeline locally
make release-prep # Prepare for release
make audit        # Security vulnerability checks
```

## 🐛 Issue Reporting

### Bug Reports
- **Clear title** describing the issue
- **Steps to reproduce** with minimal example
- **Expected vs actual behavior**
- **Environment info:** Rust version, OS, ORM/database used
- **Code samples** or configuration files

### Feature Requests
- **Use case description** - why do you need this?
- **Proposed solution** - how should it work?
- **Alternative approaches** you've considered
- **Impact assessment** - breaking changes, complexity

### Questions
- Use **GitHub Discussions** for questions
- Check existing issues and documentation first
- Provide context about your use case

## 💻 Development Guidelines

### Code Style

We follow Rust's official style guidelines with some additional rules:

```rust
// ✅ Good: Clear, idiomatic Rust
pub fn generate_entity_struct(
    &self,
    type_name: &str,
    parsed_type: &ParsedType,
    config: &Config,
) -> anyhow::Result<String> {
    // Implementation
}

// ❌ Avoid: Non-idiomatic patterns
pub fn generateEntityStruct(
    &self,
    type_name: &str,
    parsed_type: &ParsedType,
    config: &Config
) -> Result<String, anyhow::Error> {
    // Implementation
}
```

### Error Handling

Use `anyhow` for error handling:

```rust
use anyhow::{Context, anyhow};

// ✅ Good: Descriptive error messages
pub fn parse_schema(&self, content: &str) -> anyhow::Result<ParsedSchema> {
    let document = graphql_parser::parse_schema(content)
        .context("Failed to parse GraphQL schema")?;

    self.parse_document(document)
        .context("Failed to process parsed schema")
}

// ❌ Avoid: Generic error messages
pub fn parse_schema(&self, content: &str) -> anyhow::Result<ParsedSchema> {
    let document = graphql_parser::parse_schema(content)?;
    self.parse_document(document)
}
```

### Testing

- **Unit tests** for individual functions
- **Integration tests** for end-to-end functionality
- **Snapshot tests** for generated code stability
- **Property-based tests** for complex logic

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foreign_key_detection() {
        let field = ParsedField {
            name: "authorId".to_string(),
            field_type: FieldType::Scalar("ID".to_string()),
            ..Default::default()
        };

        assert_eq!(is_foreign_key_field(&field), Some("Author".to_string()));
    }

    #[tokio::test]
    async fn test_code_generation() {
        // Integration test
        let schema = create_test_schema();
        let config = create_test_config();

        let result = generate_entities(&schema, &config).await;
        assert!(result.is_ok());
    }
}
```

### Documentation

Every public API must have documentation:

```rust
/// Generates entity struct code for the given GraphQL type.
///
/// This function creates a complete Rust struct with derives,
/// fields, and relationships based on the parsed GraphQL type.
///
/// # Arguments
/// * `type_name` - The name of the GraphQL type
/// * `parsed_type` - The parsed GraphQL type information
/// * `config` - Code generation configuration
///
/// # Returns
/// Returns the generated Rust code as a string, or an error if generation fails.
///
/// # Examples
/// ```
/// let code = generator.generate_entity_struct("User", &parsed_type, &config)?;
/// println!("{}", code);
/// ```
pub fn generate_entity_struct(
    &self,
    type_name: &str,
    parsed_type: &ParsedType,
    config: &Config,
) -> anyhow::Result<String> {
    // Implementation
}
```

## 🎯 Contribution Areas

### High Priority (Good for newcomers)

- **📖 Documentation improvements** - Fix typos, add examples, clarify confusing sections
- **🧪 Test coverage** - Add tests for uncovered code paths
- **🐛 Bug fixes** - Small, focused bug fixes
- **⚡ Performance optimizations** - Speed up code generation or runtime

### Medium Priority

- **🔧 CLI improvements** - Better error messages, new commands
- **📊 Metrics and monitoring** - Add generation statistics
- **🔒 Security hardening** - Input validation, secure defaults
- **🗃️ Database support** - Additional database drivers

### Advanced Features

- **🔗 Relationship mapping** - Many-to-many, polymorphic associations
- **🎭 Union/Interface support** - Advanced type handling
- **🔌 Plugin system** - Extensible code generation
- **📡 GraphQL subscriptions** - Real-time data sync
- **⚡ Query optimization** - Compile-time query analysis

## 📋 Pull Request Process

### 1. Preparation
```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Make changes
# Add tests
# Run full development workflow
make dev

# Commit changes
git add .
git commit -m "feat: add your feature description"
```

### 2. Pull Request
- **Title:** `feat:`, `fix:`, `docs:`, `refactor:`, `test:` prefixes
- **Description:** Clear explanation of changes and why they're needed
- **Tests:** Include tests for new functionality
- **Breaking changes:** Clearly marked and explained

### 3. Code Review
- Address review feedback promptly
- Keep discussions focused and productive
- Be open to refactoring suggestions

## 🏗️ Architecture Overview

### Core Components

```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Public API
├── cli.rs            # Command-line interface
├── config.rs         # Configuration handling
├── parser.rs         # GraphQL schema parsing
├── generator.rs      # Code generation utilities
└── generator/
    ├── diesel.rs     # Diesel ORM code generation
    └── sea_orm.rs    # Sea-ORM code generation
```

### Key Design Principles

1. **Separation of Concerns** - Parser, generator, and CLI are independent
2. **Extensibility** - Easy to add new ORMs or output formats
3. **Type Safety** - Leverage Rust's type system throughout
4. **Performance** - Compile-time code generation, runtime efficiency
5. **Developer Experience** - Clear error messages, helpful documentation

## 🔍 Code Quality Standards

### Clippy Compliance
- All warnings treated as errors
- No `#[allow(clippy::...)]` without justification
- Follows Rust best practices

### Testing Standards
- Minimum 80% code coverage
- Integration tests for end-to-end functionality
- Property-based tests for complex logic
- Fuzz testing for schema parsing

### Documentation Standards
- All public APIs documented
- Examples in documentation
- Troubleshooting guides
- Architecture decision records

## 🚀 Release Process

Contributors with write access can trigger releases:

```bash
# Bump version
make version-patch  # or version-minor / version-major

# Prepare release
make release-prep

# Push tag (triggers automated release)
git push origin v1.2.3
```

## 📞 Getting Help

- **Issues:** Bug reports and feature requests
- **Discussions:** Questions and community support
- **Discord:** Real-time chat (if available)
- **Documentation:** Check docs/ folder first

## 🙏 Recognition

Contributors are recognized in:
- CHANGELOG.md for each release
- GitHub contributors list
- Future contributor acknowledgments

---

Thank you for contributing to GraphQL Rust Codegen! Your efforts help make offline-first applications better for everyone. 🎉
