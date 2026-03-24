# Project Foundation

Name: alog
Type: CLI
Purpose: A logbook for AI agents.

# Technical Foundation

## Languages and Frameworks

### CLI
Language: Rust
Frameworks: clap
Guidance:
- The `alog` CLI should compile to a single, statically linked binary for macOS, Linux, and Windows.
- Idiomatic, async Rust code
- Small, easily understood modules and crates
- Prefer simplicity (understandable by an intermediate Rust programmer) over complex code that is concise.

# Best Practices
- Organize code in small, simple, modular components
- Each component should contain unit tests that validate its behaviour in terms of inputs and outputs
- The overall codebase should contain integration tests that validate the interation between components that are used together

# Personas

### Persona 1:
Name: agent
Purpose: user of the `alog` CLI tool in a macOS, linux, or Windows environment.
Use-cases:
- executing `alog <>` commands
RBAC:
- allowed: all
- disallowed: none