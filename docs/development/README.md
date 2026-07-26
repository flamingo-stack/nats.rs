# Development Documentation

Welcome to the development section of the **nats.rs** documentation. This section covers everything you need to contribute to, extend, or operate the library in a development context.

## Contents

| Document | Description |
|---|---|
| [Environment Setup](setup/environment.md) | IDE configuration, editor extensions, and recommended development tools |
| [Local Development](setup/local-development.md) | Cloning, building, running examples, and debugging locally |
| [Architecture Overview](architecture/README.md) | High-level design, core components, data flow diagrams |
| [Security Guidelines](security/README.md) | Authentication patterns, TLS, secrets management, and secure coding guidelines |
| [Testing](testing/README.md) | Test structure, running tests, writing new tests, coverage |
| [Contributing Guidelines](contributing/guidelines.md) | Code style, branch naming, commit conventions, PR process |

## Quick Navigation

### I want to set up my development environment

→ Start with [Environment Setup](setup/environment.md), then [Local Development](setup/local-development.md).

### I want to understand how the library works

→ Read the [Architecture Overview](architecture/README.md) for diagrams and component descriptions, then explore the reference docs under `docs/reference/architecture/`.

### I want to write and run tests

→ See [Testing](testing/README.md) for test organisation, how to run the suite, and how to write new integration tests.

### I want to contribute a feature or fix

→ Read [Contributing Guidelines](contributing/guidelines.md) for the full process.

### I'm working with authentication or TLS

→ See [Security Guidelines](security/README.md) for patterns, best practices, and credential management.

## About This Project

`nats.rs` is the Flamingo-maintained fork of the official NATS Rust async client. It provides the `async-nats` crate — a Tokio-based library for building services that communicate over NATS. The fork adds OpenFrame-specific JWT token refresh support on top of the upstream feature set.

Repository: [https://github.com/flamingo-stack/nats.rs](https://github.com/flamingo-stack/nats.rs)

## Community

All support, questions, and community discussion happen on the **OpenMSP Slack**:

> Join at [https://www.openmsp.ai/](https://www.openmsp.ai/)
