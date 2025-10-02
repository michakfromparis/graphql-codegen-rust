# Security Policy

## Supported Versions

We take security seriously. The following versions of `graphql-codegen-rust` are currently being supported with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in this project, please report it responsibly. We appreciate your help in keeping our users safe.

### How to Report

Please **DO NOT** create a public GitHub issue for security vulnerabilities.

Instead, please email github@michak.net with the following information:

- A clear description of the vulnerability
- Steps to reproduce the issue
- Potential impact and severity
- Any suggested fixes or mitigations

### Response Timeline

- **Acknowledgment**: We will acknowledge receipt within 48 hours
- **Investigation**: We will investigate and provide an initial assessment within 7 days
- **Fix**: We will work on a fix and provide regular updates
- **Disclosure**: Once fixed, we will coordinate disclosure timing with you

## Security Best Practices

When using this tool, consider:

1. **Network Security**: Only run against trusted GraphQL endpoints
2. **Credential Management**: Never commit API keys or sensitive configuration
3. **Generated Code Review**: Always review generated code before using in production
4. **Dependency Updates**: Keep dependencies updated to receive security patches

## Security Features

This project includes several security-focused features:

- **Dependency Auditing**: Automated security audits via GitHub Actions
- **Code Quality**: Strict clippy linting and security-focused warnings
- **No Network Requests**: Generated code contains no runtime network calls
- **Input Validation**: GraphQL schema parsing includes basic validation

Thank you for helping keep the Rust ecosystem secure!
