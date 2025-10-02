# GraphQL Rust Codegen

A Rust CLI tool that generates ORM code from GraphQL schemas. Perfect for offline-first Tauri applications that need to sync GraphQL types to local SQLite/PostgreSQL databases.

## Features

- **GraphQL Schema Parsing**: Supports both GraphQL introspection and SDL schema files
- **Multi-ORM Support**: Generates code for both Diesel and Sea-ORM
- **Database Support**: Works with SQLite (default), PostgreSQL, and MySQL
- **Migration Generation**: Automatically creates SQL migration files
- **Union & Interface Support**: Handles GraphQL unions and interfaces in schema parsing
- **Relationship Mapping**: Automatic foreign key detection and relationship generation
- **Configuration Management**: Supports both TOML and YAML configs (compatible with GraphQL Code Generator)
- **Type Safety**: Generates strongly-typed Rust structs from GraphQL schemas
- **Tauri Integration**: Designed for seamless integration with Tauri app build processes

## Comparison to Cynic

| Feature | graphql-rust-codegen | cynic |
|---------|---------------------|-------|
| **Purpose** | Database/ORM code generation | GraphQL client code generation |
| **Output** | Diesel/Sea-ORM entities, migrations | Query builders, response types |
| **Use Case** | Offline-first apps, data persistence | API clients, GraphQL queries |
| **Architecture** | Database-first | Client-first |
| **Schema Source** | GraphQL schemas (introspection/SDL) | GraphQL schemas |
| **Runtime Dependencies** | ORM libraries (Diesel/Sea-ORM) | HTTP client + cynic runtime |
| **Tauri Integration** | Native support | Possible but not primary |
| **Migration Support** | ✅ Automatic SQL migrations | ❌ N/A |
| **Relationship Mapping** | ✅ Foreign keys, joins | ❌ N/A |
| **Database Types** | ✅ SQLite, PostgreSQL, MySQL | ❌ N/A |

**TL;DR**: cynic generates *client code* for making GraphQL requests, while graphql-rust-codegen generates *database code* for persisting GraphQL data locally.

## Installation

### From Source

```bash
git clone https://github.com/yourusername/graphql-codegen-rust.git
cd graphql-codegen-rust
cargo build --release
```

### From Crates.io (future)

```bash
cargo install graphql-codegen-rust
```

## Usage

### Simple Code Generation (Auto-detects config)

```bash
graphql-codegen-rust
```

Automatically detects `codegen.yml`, `codegen.yaml`, or `graphql-codegen-rust.toml` and generates code.

### Initialize a New Project

```bash
graphql-codegen-rust init \
  --url https://api.example.com/graphql \
  --orm diesel \
  --db sqlite \
  --output ./db
```

This will:
1. Introspect the GraphQL schema from the endpoint
2. Create a configuration file (`graphql-codegen-rust.toml`)
3. Generate Diesel schema definitions, entity structs, and migration files

### Explicit Code Generation

```bash
graphql-codegen-rust generate --config codegen.yml
```

Regenerates code from the specified configuration file.

### Tauri Integration

In your `package.json`, chain with TS codegen:

```json
{
  "scripts": {
    "codegen": "graphql-codegen --config codegen.yml && graphql-codegen-rust"
  }
}
```

This runs TS codegen first, then Rust codegen automatically.

## Configuration

The tool supports both TOML and YAML configurations. YAML configs are compatible with GraphQL Code Generator.

### YAML Configuration (Recommended for Tauri apps)

```yaml
# Compatible with GraphQL Code Generator
schema: https://api.example.com/graphql

# Optional: additional headers
# headers:
#   Authorization: "Bearer your-token-here"

# Rust codegen specific configuration
rust_codegen:
  orm: diesel
  db: sqlite
  output_dir: ./generated
  generate_migrations: true
  generate_entities: true
  table_naming: snake_case

  # Custom scalar mappings
  type_mappings:
    DateTime: "chrono::NaiveDateTime"
```

### TOML Configuration

```toml
url = "https://api.example.com/graphql"
orm = "Diesel"
db = "Sqlite"
output_dir = "./generated"
generate_migrations = true
generate_entities = true
table_naming = "snake_case"

[headers]
Authorization = "Bearer your-token-here"

[type_mappings]
# Custom scalar mappings
DateTime = "chrono::NaiveDateTime"
```

## Generated Structure

```
output_dir/
├── graphql-codegen-rust.toml  # or codegen.yml
├── src/
│   ├── schema.rs              # Diesel table! macros
│   └── entities/              # Entity structs
│       ├── user.rs
│       └── product.rs
└── migrations/
    ├── 001_create_users_table/
    │   ├── up.sql
    │   └── down.sql
    └── 002_create_products_table/
        ├── up.sql
        └── down.sql
```

## GraphQL Type Mapping

| GraphQL Type | Rust Type (SQLite) | Rust Type (PostgreSQL) | SQL Type |
|-------------|-------------------|----------------------|----------|
| `ID` | `i32` | `uuid::Uuid` | `INTEGER` / `UUID` |
| `String` | `String` | `String` | `TEXT` |
| `Int` | `i32` | `i32` | `INTEGER` |
| `Float` | `f64` | `f64` | `REAL` |
| `Boolean` | `bool` | `bool` | `INTEGER` / `BOOLEAN` |

## Examples

### Vendure Integration

```bash
graphql-codegen-rust init \
  --url https://demo.vendure.io/shop-api \
  --orm diesel \
  --db sqlite \
  --output ./src/db
```

### Tauri App Integration

Add to your `package.json`:

```json
{
  "scripts": {
    "codegen": "graphql-codegen --config codegen.yml && graphql-codegen-rust"
  }
}
```

Or in `build.rs`:

```rust
use std::process::Command;

fn main() {
    // Regenerate database code before build
    Command::new("graphql-codegen-rust")
        .status()
        .expect("Failed to regenerate database code");

    tauri_build::build()
}
```

## Development

### Quick Start

Use the provided Makefile for common development tasks:

```bash
# Full development workflow (recommended)
make dev

# Or run individual tasks
make test      # Run tests
make lint      # Run clippy
make fmt       # Format code
make doc       # Build docs
```

### Available Commands

See all available commands:
```bash
make help
```

### Manual Commands

If you prefer running cargo directly:

```bash
# Building
cargo build

# Testing
cargo test

# Running
cargo run -- init --url http://localhost:4000/graphql

# With YAML support
cargo run --features yaml-codegen-config -- init --url http://localhost:4000/graphql
```

## CI/CD

This project uses GitHub Actions for continuous integration and deployment:

- **CI Pipeline**: Runs on every push/PR with comprehensive testing across multiple platforms (Linux, macOS, Windows) and Rust versions
- **Code Quality**: Automated formatting checks and clippy linting with Rust Edition 2024
- **Code Coverage**: Automated coverage reporting with Codecov
- **Release Automation**: Automated publishing to crates.io when version tags are pushed
- **Dependency Management**: Automated dependency updates via Dependabot
- **Security Auditing**: Automated vulnerability scanning with `cargo-audit`
- **License Checking**: Automated license compliance with `cargo-deny`
- **Dependency Review**: Automated security scanning via GitHub's dependency review

### Running Security Checks Locally

All security and compliance checks work with the current MSRV (Rust 1.85+):

```bash
# Security vulnerability scanning
cargo install cargo-audit
cargo audit

# License and dependency compliance
cargo install cargo-deny --version 0.18.3
cargo deny check

# Code coverage (requires llvm-tools)
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace
```

### Publishing a New Release

Use the Makefile for streamlined release management:

```bash
# Prepare for release (run all checks)
make release-prep

# Bump version automatically (requires cargo-bump)
make version-patch  # or version-minor / version-major

# Or manual process:
# 1. Update version in Cargo.toml
# 2. Update CHANGELOG.md
# 3. Commit changes
# 4. Create and push version tag:
git tag v1.0.0
git push origin v1.0.0
```

The release workflow will automatically:
- Run full test suite
- Build and verify package
- Publish to crates.io
- Create GitHub release

### Available Release Commands

```bash
make release-prep    # Prepare for release (all checks)
make release-check   # Validate release package
make version-patch   # Bump patch version (1.0.0 -> 1.0.1)
make version-minor   # Bump minor version (1.0.0 -> 1.1.0)
make version-major   # Bump major version (1.0.0 -> 2.0.0)
```

## Limitations

- Currently focuses on object types and basic relationships
- Unions, interfaces, and complex nested types are not yet fully supported
- Subscriptions are ignored
- Custom scalars require manual mapping in configuration

## Contributing

Contributions are welcome! Areas for improvement:

- SDL parsing support (currently introspection-only)
- Union/interface support
- Better relationship handling
- More ORM integrations
- Plugin system for custom generators

## License

MIT License

## Roadmap

- [x] YAML configuration support (compatible with GraphQL Code Generator)
- [ ] SDL file parsing support
- [ ] Union and interface type generation
- [ ] Advanced relationship mapping
- [x] Sea-ORM migration generation
- [ ] Plugin system
- [ ] GraphQL subscription support
- [ ] CI/CD and releases
