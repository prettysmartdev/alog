# CLAUDE.md — alog

## Source of Truth

The `aspec/` directory is the **single source of truth** for this project. All decisions about architecture, features, CLI design, security, testing, and workflows must conform to the aspec. Read the relevant aspec documents before making any change.

## Project Overview

`alog` is a CLI logbook for AI agents — a tool that lets agents write and recall lightly-structured notes persistently.

- **Language:** Rust
- **CLI framework:** clap (derive)
- **Binary name:** `alog`
- **Binary type:** single statically-linked binary for macOS, Linux, and Windows

## aspec Index

| File | Contents |
|------|----------|
| `aspec/foundation.md` | Language, frameworks, best practices, personas |
| `aspec/architecture/design.md` | Architecture pattern, design principles, components |
| `aspec/architecture/security.md` | Security guidance |
| `aspec/uxui/cli.md` | CLI commands, flags, storage paths |
| `aspec/devops/localdev.md` | Local development workflows, documentation rules |
| `aspec/devops/operations.md` | Installation and runtime operations |
| `aspec/work-items/0000-template.md` | Template for new work items |

## CLI Commands (from aspec/uxui/cli.md)

```
alog write <category> <entry>
    --project=<name>   project to associate with this entry
    --replace=<id>     add new entry and delete the entry with this id

alog recall <category|all> <search_term>
    --project=<name>   restrict search to this project
    --count=<n>        maximum number of results
    --threshold=<n>    minimum % similarity (0–100)
```

## Storage Layout (from aspec/uxui/cli.md)

```
$HOME/.alog/
  config.json                              # global config
  logbook/
    global/                                # entries with no project
      <category>.json
    <projectname>/
      <category>.json
GITROOT/.alog.json                         # per-repo config
```

## Development Workflows (from aspec/devops/localdev.md)

```
make all      # build the alog binary (cargo build --release)
make install  # build + install to /usr/local/bin/alog
make test     # run all tests (cargo test)
```

## Code Standards (from aspec/foundation.md + aspec/architecture/design.md)

- Idiomatic, async Rust (`tokio`)
- Small, simple, modular components — prefer simplicity over conciseness
- Code should be understandable by an intermediate Rust programmer
- **Unit tests** in every module validating inputs/outputs
- **Integration tests** in `tests/` for component interactions
- **End-to-end tests** testing the CLI binary behavior

## Security (from aspec/architecture/security.md)

- Unix file permissions: directories `0o700`, files `0o600`
- Apply permissions immediately after creating any `.alog` directory or file

## Documentation Rules (from aspec/devops/localdev.md)

- After every work item, inspect and update `docs/` with comprehensive usage docs
- Do **not** create one document per work item
- Maintain a holistic, up-to-date documentation set that explains the tool in full

## Work Items

- Templates in `aspec/work-items/0000-template.md`
- Each work item must follow: User Stories → Implementation Details → Edge Cases → Test Considerations → Codebase Integration
- After implementing a work item: update `docs/`, run `make test`, verify the binary builds
