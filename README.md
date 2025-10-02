# GraphQL Rust Codegen

**The missing piece for offline-first applications** 🚀

[![CI](https://github.com/michakfromparis/graphql-codegen-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/michakfromparis/graphql-codegen-rust/actions)
[![codecov](https://codecov.io/gh/michakfromparis/graphql-codegen-rust/branch/main/graph/badge.svg)](https://codecov.io/gh/michakfromparis/graphql-codegen-rust)
[![Crates.io](https://img.shields.io/crates/v/graphql-codegen-rust.svg)](https://crates.io/crates/graphql-codegen-rust)
[![Docs.rs](https://docs.rs/graphql-codegen-rust/badge.svg)](https://docs.rs/graphql-codegen-rust)

[![Rust](https://img.shields.io/badge/rust-1.86%2B-orange)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/michakfromparis/graphql-codegen-rust/blob/main/LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/michakfromparis/graphql-codegen-rust/blob/main/docs/contributing.md)

A powerful Rust CLI tool that transforms GraphQL schemas into production-ready database code. Built specifically for developers creating offline-first applications with Tauri, it bridges the gap between GraphQL APIs and local data persistence.

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

## 🚀 Quick Start

```bash
# Initialize your project
graphql-codegen-rust init \
  --url https://api.example.com/graphql \
  --orm diesel \
  --db sqlite \
  --output ./src/db
```

That's it! You'll get:
- **Database entities** with proper Rust types
- **Migration files** for schema setup
- **Relationship mappings** for foreign keys
- **Type-safe ORM code** ready for production

## 📚 Documentation

- **[Getting Started](docs/getting-started.md)** - Installation and basic usage
- **[Configuration](docs/configuration.md)** - TOML/YAML setup and options
- **[Examples](docs/examples.md)** - Real-world integration examples
- **[Comparisons](docs/comparisons.md)** - How it stacks up against similar tools
- **[Reference](docs/reference.md)** - Type mappings and generated structure
- **[Contributing](docs/contributing.md)** - Development setup and contribution guidelines

## ✨ Key Features

- **🔍 Dual Schema Support**: GraphQL introspection + SDL file parsing
- **🗄️ Multi-ORM Ready**: Diesel and Sea-ORM support out of the box
- **💾 Database Agnostic**: SQLite, PostgreSQL, and MySQL
- **🔄 Migration Generation**: Automatic SQL migration files
- **🔗 Smart Relationships**: Foreign key detection and ORM relationships
- **🎯 Type Safety**: Compile-time guarantees for your data layer
- **⚡ Performance**: Native Rust speed with zero runtime overhead
- **🔧 Tauri Native**: Seamless desktop app integration

## 🎯 Perfect For

- **Offline-first Tauri applications**
- **Local data synchronization workflows**
- **Type-safe database layer generation**
- **Rapid GraphQL-to-SQL prototyping**
- **Enterprise data persistence needs**

---

**Ready to build offline-first apps with confidence?** Dive into the [Getting Started](docs/getting-started.md) guide.

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  <strong>Built with ❤️ for the offline-first future</strong>
</p>
