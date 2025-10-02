# 🚀 Getting Started

Welcome to GraphQL Rust Codegen! This guide will get you up and running with generating type-safe database code from your GraphQL schemas.

## 📦 Installation

### From Crates.io (Recommended)

```bash
cargo install graphql-codegen-rust
```

### From Source

```bash
git clone https://github.com/michakfromparis/graphql-codegen-rust.git
cd graphql-codegen-rust
cargo build --release
```

## 🎯 Quick Start

### Initialize a New Project

The fastest way to get started is with the `init` command:

```bash
graphql-codegen-rust init \
  --url https://api.example.com/graphql \
  --orm diesel \
  --db sqlite \
  --output ./src/db
```

This command will:
1. **Introspect** your GraphQL schema from the API endpoint
2. **Create** a configuration file (`graphql-codegen-rust.toml`)
3. **Generate** database entities, migrations, and relationship mappings

### Auto-Detection

If you already have a config file, simply run:

```bash
graphql-codegen-rust
```

It automatically detects `codegen.yml`, `codegen.yaml`, or `graphql-codegen-rust.toml`.

### Explicit Generation

For CI/CD or explicit control:

```bash
graphql-codegen-rust generate --config codegen.yml
```

## 🏗️ What You Get

After running code generation, you'll have:

```
output_dir/
├── graphql-codegen-rust.toml    # Your configuration
├── src/
│   ├── schema.rs                # Diesel table! macros
│   └── entities/                # Type-safe entity structs
│       ├── user.rs
│       ├── post.rs
│       └── category.rs
└── migrations/                  # SQL migration files
    ├── 001_create_users/
    │   ├── up.sql
    │   └── down.sql
    └── 002_create_posts/
        ├── up.sql
        └── down.sql
```

## 🛠️ Development Workflow

### Using Make (Recommended)

```bash
# Full development workflow
make dev

# Individual tasks
make test      # Run tests
make lint      # Code quality checks
make fmt       # Format code
make doc       # Generate docs
```

### Manual Commands

```bash
# Build
cargo build

# Test
cargo test

# Run with custom schema
cargo run -- init --url http://localhost:4000/graphql

# Generate with YAML support
cargo run --features yaml-codegen-config -- generate --config codegen.yml
```

## 🔧 Tauri Integration

### Package.json Script

Chain with your frontend codegen:

```json
{
  "scripts": {
    "codegen": "graphql-codegen --config codegen.yml && graphql-codegen-rust"
  }
}
```

### Build.rs Integration

Regenerate database code on every build:

```rust
// build.rs
use std::process::Command;

fn main() {
    // Keep database schema in sync
    Command::new("graphql-codegen-rust")
        .status()
        .expect("Failed to regenerate database code");

    tauri_build::build()
}
```

## 🚨 Troubleshooting

### Common Issues

**"GraphQL introspection failed"**
- Check your GraphQL endpoint URL
- Verify authentication headers are correct
- Ensure the GraphQL server allows introspection

**"No entities generated"**
- Confirm your schema has object types (not just queries/mutations)
- Check the output directory permissions
- Verify your ORM selection (diesel vs sea-orm)

**"Compilation errors"**
- Update your dependencies: `cargo update`
- Check Rust version compatibility (MSRV: 1.85+)
- Ensure generated code matches your ORM versions

### Debug Mode

Enable verbose output:

```bash
RUST_LOG=debug graphql-codegen-rust init --url https://api.example.com/graphql
```

## 📚 Next Steps

- **[Configuration Guide](configuration.md)** - Fine-tune code generation
- **[Examples](examples.md)** - Real-world integration patterns
- **[Reference](reference.md)** - Type mappings and limitations

Ready to dive deeper? Check out the [configuration options](configuration.md) to customize your generated code.
