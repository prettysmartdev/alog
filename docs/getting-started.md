# Getting Started with alog

`alog` is a CLI logbook for AI agents. This guide covers installation, basic use, project initialisation, and setting up the Claude Code skill.

## Requirements

- Rust toolchain (stable) — install via [rustup](https://rustup.rs)
- `make`

## Build and Install

```bash
# Build the binary
make all

# Build and install to /usr/local/bin/alog
make install
```

Verify the install:

```bash
alog --help
```

## Initialising a Project

Run `alog init` once inside any git repository to configure alog for that project:

```bash
cd /path/to/your/repo
alog init
```

`alog init` does two things:

1. **Creates `.alog.json`** — a per-repo config file that sets a default similarity threshold for `alog recall`. The file is placed at `<git-root>/aspec/.alog.json` if an `aspec/` directory exists, otherwise at `<git-root>/.alog.json`.

2. **Optionally installs the Claude Code skill** — prompts you to download `.claude/skills/alog.md` from the alog repository. When installed, the skill teaches Claude Code to recall notes before starting work and write notes automatically when it finishes.

Example output:

```
Created /path/to/repo/.alog.json
Download .claude/skills/alog.md from the alog repository? [y/N] y
Installed /path/to/repo/.claude/skills/alog.md
```

### `.alog.json` format

```json
{
  "defaultSimilarityThreshold": 25
}
```

`defaultSimilarityThreshold` sets the minimum percentage similarity (0–100) that a note must reach to be returned by `alog recall`. The `--threshold` flag always overrides this value.

## Writing Notes

```bash
alog write <category> "<entry>" [--project=<name>]
```

Examples:

```bash
# Record a bug fix, scoped to a project
alog write bugfix "reqwest blocking client panics inside axum handler — use async client only" --project=myapi

# Record a useful pattern, globally (no project)
alog write patterns "Use #[derive(thiserror::Error)] with #[from] for automatic error conversion in Rust"

# Record a decision
alog write decisions "chose tokio over async-std — better ecosystem support for axum and sqlx" --project=myapi
```

## Recalling Notes

```bash
alog recall <category|all> "<search term>" [--project=<name>] [--count=<n>] [--threshold=<n>]
```

Examples:

```bash
# Search all categories in a project
alog recall all "authentication" --project=myapi

# Narrow results by similarity and count
alog recall bugfix "panic runtime" --project=myapi --count=5 --threshold=70

# Search globally across all projects
alog recall warnings "async"
```

`--threshold` accepts 0–100 (minimum % similarity). `--count` caps the number of results.

### Default threshold from `.alog.json`

When `--threshold` is not given, `alog recall` reads `defaultSimilarityThreshold` from `.alog.json` in the current git root (or its `aspec/` subdirectory) and applies it automatically. Run `alog init` to create this file. The `--threshold` flag always takes priority over the config value.

## Replacing a Stale Note

`alog recall` returns an `id` with each result. Pass that id to `--replace` to overwrite an outdated entry:

```bash
alog write decisions "switched from diesel to sqlx — async support" --project=myapi --replace=<id>
```

This adds the new entry and deletes the old one atomically.

## Storage Layout

Notes are stored as JSON files on disk:

```
$HOME/.alog/
  config.json                   # global config
  logbook/
    global/                     # notes with no --project
      <category>.json
    <projectname>/              # notes scoped to a project
      <category>.json
```

Per-repo configuration lives at `<git-root>/.alog.json` (or `<git-root>/aspec/.alog.json` when an `aspec/` directory is present).

Directories are created with `0700` permissions; files with `0600`.

## Claude Code Skills Integration

The repo includes a Claude Code skill at `.claude/skills/alog.md`. When Claude Code loads this skill, Claude will:

1. **Recall** relevant notes before starting any non-trivial task
2. **Write** notes automatically after fixing bugs, making decisions, or hitting dead ends
3. **Scope** all notes to the current repo using `--project=<git-root-name>`

Install the skill via `alog init`, or copy `.claude/skills/alog.md` manually into your project's `.claude/skills/` directory.

### How project names are derived

Claude derives the project name from the git root directory:

```bash
basename $(git rev-parse --show-toplevel)
```

This value is used for every `--project=` flag, keeping notes organized per-repo without manual input.

### Skill activation

Skills in `.claude/skills/` are loaded automatically when Claude Code starts in that directory. No additional configuration is needed — clone the repo and start a session.

## Running Tests

```bash
make test
```

This runs all unit, integration, and end-to-end tests via `cargo test`.
