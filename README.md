# GraphQL Rust Codegen

A Rust CLI tool that generates ORM code from GraphQL schemas. Perfect for offline-first Tauri applications that need to sync GraphQL types to local SQLite/PostgreSQL databases.

## Features

- **GraphQL Schema Introspection**: Fetches and parses GraphQL schemas from remote endpoints
- **Multi-ORM Support**: Generates code for both Diesel and Sea-ORM
- **Database Support**: Works with SQLite (default), PostgreSQL, and MySQL
- **Migration Generation**: Automatically creates SQL migration files
- **Configuration Management**: Supports both TOML and YAML configs (compatible with GraphQL Code Generator)
- **Type Safety**: Generates strongly-typed Rust structs from GraphQL schemas
- **Tauri Integration**: Designed for seamless integration with Tauri app build processes

## Installation

### From Source

```bash
git clone https://github.com/yourusername/graphql-rust-codegen.git
cd graphql-rust-codegen
cargo build --release
```

### From Crates.io (future)

```bash
cargo install graphql-rust-codegen
```

## Usage

### Simple Code Generation (Auto-detects config)

```bash
graphql-rust-codegen
```

Automatically detects `codegen.yml`, `codegen.yaml`, or `graphql-rust-codegen.toml` and generates code.

### Initialize a New Project

```bash
graphql-rust-codegen init \
  --url https://api.example.com/graphql \
  --orm diesel \
  --db sqlite \
  --output ./db
```

This will:
1. Introspect the GraphQL schema from the endpoint
2. Create a configuration file (`graphql-rust-codegen.toml`)
3. Generate Diesel schema definitions, entity structs, and migration files

### Explicit Code Generation

```bash
graphql-rust-codegen generate --config codegen.yml
```

Regenerates code from the specified configuration file.

### Tauri Integration

In your `package.json`, chain with TS codegen:

```json
{
  "scripts": {
    "codegen": "graphql-codegen --config codegen.yml && graphql-rust-codegen"
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
├── graphql-rust-codegen.toml  # or codegen.yml
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
graphql-rust-codegen init \
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
    "codegen": "graphql-codegen --config codegen.yml && graphql-rust-codegen"
  }
}
```

Or in `build.rs`:

```rust
use std::process::Command;

fn main() {
    // Regenerate database code before build
    Command::new("graphql-rust-codegen")
        .status()
        .expect("Failed to regenerate database code");

    tauri_build::build()
}
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running

```bash
cargo run -- init --url http://localhost:4000/graphql
```

### With YAML Support

```bash
cargo run --features yaml-codegen-config -- init --url http://localhost:4000/graphql
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
- [ ] Sea-ORM migration generation
- [ ] Plugin system
- [ ] GraphQL subscription support
- [ ] CI/CD and releases
