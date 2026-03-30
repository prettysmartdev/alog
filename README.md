<p align="center">
  <strong>A notebook for your agents.</strong> <br>
  Help with memory, recall, and predictable workflows. <br>
  Journaling is good for your agents' mental health.
  <br>
  <br>
  <img src="./docs/alog_logo.svg" width="420" alt="ALOG">
</p>

[![Tests](https://github.com/prettysmartdev/alog/actions/workflows/test.yml/badge.svg)](https://github.com/prettysmartdev/alog/actions/workflows/test.yml)

---

# What is alog?

`alog` is a CLI logbook for AI agents. Agents write notes during a session — bugs found, patterns observed, approaches that worked or failed — and recall them in future sessions using fuzzy search. It gives agents a persistent memory that survives context resets.

Notes are stored as local JSON files. No server, no account, no dependencies.

## Usage

**Write a note:**
```bash
alog write bugfix "tokio runtime panicked — was calling .unwrap() on a blocking read inside async fn. Fix: use tokio::task::spawn_blocking." --project=myapi
```

**Recall notes:**
```bash
alog recall bugfix "tokio panic" --project=myapi
alog recall all "authentication" --project=myapi --count=5 --threshold=70
```

**Replace a stale note** (id is returned by `alog recall`):
```bash
alog write decisions "switched from sqlx to diesel — better compile-time guarantees" --project=myapi --replace=abc123
```

## Categories

| Category | Use for |
|----------|---------|
| `bugfix` | Root cause and fix for a bug |
| `whatworks` | Approaches and patterns that succeeded |
| `problems` | Dead ends and failures |
| `patterns` | Recurring code idioms in a codebase |
| `decisions` | Architectural decisions and rationale |
| `warnings` | Gotchas and sharp edges |
| `deps` | Dependency quirks and version notes |
| `perf` | Performance findings |
| `tests` | Testing patterns and structure |
| `setup` | Environment and toolchain notes |

## Claude Code Skills Integration

`alog` ships with three Claude Code skills installed by `alog init`:

| Skill | Purpose |
|-------|---------|
| `.claude/skills/alog/` | Core: recall before tasks, write after findings, tag with sessions |
| `.claude/skills/alog-summarize/` | Synthesize session summaries into the `summaries` category |
| `.claude/skills/alog-export/` | Generate Markdown reports on request |

When active, Claude automatically recalls relevant notes before starting work, writes notes after fixing bugs or making decisions, and scopes all notes to the current repo using `--project=<git-root-name>`. The skills give Claude a persistent working memory across sessions without any manual prompting.

## Storage

```
$HOME/.alog/
  config.json
  logbook/
    global/
      <category>.json
    <projectname>/
      <category>.json
```

Per-repo config lives at `GITROOT/.alog.json`.

## Installation

```bash
curl -s https://prettysmart.dev/install/alog.sh | sh
```

This detects your platform and architecture, downloads the correct binary, and installs it to `/usr/local/bin/alog`. For alternative install methods (manual binary download, build from source) see **[docs/getting-started.md](docs/getting-started.md)**.

## Getting Started

See **[docs/getting-started.md](docs/getting-started.md)** for a full walkthrough.