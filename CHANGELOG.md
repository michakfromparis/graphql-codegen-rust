# Changelog
## [Unreleased]



All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-10-03

### Added
- Initial release of GraphQL Rust Codegen
- **Dual schema support**: GraphQL introspection + SDL file parsing
- **Union and interface support**: Advanced GraphQL schema type handling
- **Relationship mapping**: Automatic foreign key detection and ORM relationships
- **Structured logging**: Verbosity levels and enhanced error handling
- Support for generating Diesel ORM code from GraphQL schemas
- Support for generating Sea-ORM code from GraphQL schemas
- YAML configuration support compatible with GraphQL Code Generator
- TOML configuration format
- Automatic schema introspection from GraphQL endpoints
- Migration generation for database schemas
- Entity generation for ORM models
- CLI interface with init and generate commands
- CLI verbosity options (-v, -vv, -vvv)
- Library interface for programmatic usage
- Comprehensive test suite
- Comprehensive documentation with guides, examples, and reference

### Changed
- Renamed from `graphql-diesel-sync` to `graphql-codegen-rust`
- Upgraded to Rust Edition 2024
- Updated MSRV to Rust 1.86

### Technical
- Added GitHub Actions CI/CD pipeline
- Added security auditing with `cargo-audit`
- Added license compliance checking with `cargo-deny`
- Added code coverage reporting with Codecov
- Added dependency management with Dependabot
- Added comprehensive linting and formatting checks
- Upgraded Sea-ORM from 0.12 to 1.1.16 (fixes SQLx security vulnerability)
- Updated CI to use Rust 1.86.0 for MSRV compatibility
