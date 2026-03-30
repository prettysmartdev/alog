# Getting Started with alog

`alog` is a CLI logbook for AI agents. This guide covers installation, basic use, project initialisation, and setting up the Claude Code skill.

## Installation

### Install script (recommended)

```bash
curl -s https://prettysmart.dev/install/alog.sh | sh
```

This detects your platform and architecture, downloads the correct binary, and installs it to `/usr/local/bin/alog`.

Verify the install:

```bash
alog --help
```

### Alternative: download a pre-built binary

Download the binary for your platform from the [latest release](https://github.com/prettysmartdev/alog/releases/latest), extract it, and move it onto your `$PATH`:

```bash
# Example for Linux amd64
tar xzf alog-linux-amd64.tar.gz
sudo mv alog /usr/local/bin/alog
```

Available archives:

| Platform | Archive |
|----------|---------|
| Linux amd64 | `alog-linux-amd64.tar.gz` |
| Linux arm64 | `alog-linux-arm64.tar.gz` |
| macOS amd64 | `alog-macos-amd64.tar.gz` |
| macOS arm64 (Apple Silicon) | `alog-macos-arm64.tar.gz` |
| Windows amd64 | `alog-windows-amd64.zip` |

### Alternative: build from source

**Requirements:** Rust toolchain (stable) — install via [rustup](https://rustup.rs), `make`

```bash
git clone https://github.com/prettysmartdev/alog.git
cd alog

# Build the binary
make all

# Build and install to /usr/local/bin/alog
make install
```

## Initialising a Project

Run `alog init` once inside any git repository to configure alog for that project:

```bash
cd /path/to/your/repo
alog init
```

`alog init` does two things:

1. **Creates `.alog.json`** — a per-repo config file that sets a default similarity threshold for `alog recall`. The file is placed at `<git-root>/aspec/.alog.json` if an `aspec/` directory exists, otherwise at `<git-root>/.alog.json`.

2. **Optionally installs the Claude Code skills** — prompts you to download all three skill files from the alog repository into `.claude/skills/`. When installed, the skills teach Claude Code to recall notes before starting work, write notes automatically when it finishes, summarize sessions, and export entries on request.

Example output:

```
Created /path/to/repo/.alog.json
Download Claude Code skills (.claude/skills/) from the alog repository? [y/N] y
Installed /path/to/repo/.claude/skills/alog/SKILL.md
Installed /path/to/repo/.claude/skills/alog-summarize/SKILL.md
Installed /path/to/repo/.claude/skills/alog-export/SKILL.md
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
alog write <category> "<entry>" [--project=<name>] [--session=<id>]
```

Examples:

```bash
# Record a bug fix, scoped to a project
alog write bugfix "reqwest blocking client panics inside axum handler — use async client only" --project=myapi

# Record a useful pattern, globally (no project)
alog write patterns "Use #[derive(thiserror::Error)] with #[from] for automatic error conversion in Rust"

# Record a decision, tagged with a session identifier
alog write decisions "chose tokio over async-std — better ecosystem support for axum and sqlx" --project=myapi --session=sess-20260327-abc123
```

### Session tagging

The `--session` flag attaches a session identifier to an entry. This lets you group, filter, and export entries by session later. The session value can be any string up to 100 characters — a UUID, a date-based string, or any identifier that is stable across a single work session.

```bash
# Generate a session ID
SESSION_ID="$(date +%Y%m%d)-$(cat /proc/sys/kernel/random/uuid | cut -d- -f1)"

# Tag all entries in this session
alog write bugfix "fixed null check" --project=myapp --session=$SESSION_ID
alog write decisions "use postgres" --project=myapp --session=$SESSION_ID
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

## Exporting Notes

```bash
alog export <output-path> [--project=<name>] [--category=<name>] [--session=<id>]
```

The export command writes matching entries to a Markdown file. Pass `-` as the path to write to stdout instead.

Examples:

```bash
# Export all entries for a session to stdout
alog export - --session=sess-20260327-abc123

# Export all bugs for a project to a file
alog export ~/reports/myapi-bugs.md --project=myapi --category=bugfix

# Export session summaries to a file
alog export ~/reports/summaries.md --category=summaries

# Export everything for a project and session
alog export ~/reports/session-report.md --project=myapp --session=$SESSION_ID
```

Filters can be combined. Entries that match all specified filters are included. Omitting a filter means "any value" for that field.

The output is formatted as human-readable Markdown, grouped with headers showing the category and timestamp for each entry.

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

The repo includes three Claude Code skills in `.claude/skills/`:

| Skill | Purpose |
|-------|---------|
| `alog/` | Core skill — recall before tasks, write after findings, use sessions |
| `alog-summarize/` | Synthesize a session summary and write it to the `summaries` category |
| `alog-export/` | Generate Markdown reports from stored entries on user request |

When Claude Code loads these skills, Claude will:

1. **Recall** relevant notes before starting any non-trivial task
2. **Write** notes automatically after fixing bugs, making decisions, or hitting dead ends
3. **Scope** all notes to the current repo using `--project=<git-root-name>`
4. **Tag** every entry with a session identifier using `--session=<id>`
5. **Summarize** the session periodically and at the end using `/alog-summarize`
6. **Export** entries on request using `/alog-export`

Install skills via `alog init`, or copy the skill directories from `.claude/skills/` manually into your project's `.claude/skills/` directory.

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
