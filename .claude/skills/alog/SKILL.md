---
name: alog
description: Use the alog CLI to write and recall persistent notes about your work. Invoke this skill whenever you complete a task, discover something notable, or want to search past findings before starting new work.
---

# alog — AI Logbook Skill

`alog` is a persistent logbook CLI for AI agents. Use it to record findings and recall them across sessions.

## Commands

```
alog write <category> "<entry>"  [--project=<name>]  [--session=<id>]
alog recall <category|all> "<search term>"  [--project=<name>]  [--count=<n>]  [--threshold=<n>]
alog export <output-path>  [--project=<name>]  [--category=<name>]  [--session=<id>]
```

## Determining the project name

Always use `--project` to scope notes to the current repo. Derive the project name from the git root directory name:

```bash
# Get current project name
basename $(git rev-parse --show-toplevel)
```

Use that value for every `--project=` flag.

## Using sessions

Tag every log entry with a session identifier so entries can be grouped and exported by session later.

Use a stable session ID throughout a single work session. A good session ID is the Claude session ID if available, or a short unique string such as:

```bash
# Generate a session ID based on date and a short random suffix
SESSION_ID="$(date +%Y%m%d)-$(cat /proc/sys/kernel/random/uuid | cut -d- -f1)"
```

Pass this as `--session=$SESSION_ID` on every `alog write` call during the session.

## Categories

Choose the most specific category that fits:

| Category | When to use |
|----------|-------------|
| `bugfix` | A bug was found and fixed — record root cause, symptoms, and the fix |
| `whatworks` | An approach, pattern, or technique that succeeded — record what and why |
| `problems` | A blocker, failure, or dead end encountered — record what failed and why |
| `patterns` | A recurring code pattern, idiom, or convention observed in this codebase |
| `decisions` | An architectural or design decision made — record rationale and tradeoffs |
| `warnings` | Footguns, gotchas, or sharp edges discovered — record what to avoid |
| `deps` | Dependency behavior, quirks, or version-specific notes |
| `perf` | Performance findings — what was slow, what helped, what to measure |
| `tests` | Testing patterns, what's hard to test, or how the test suite is structured |
| `setup` | Environment, toolchain, or configuration notes |
| `summaries` | End-of-session summaries — use `/alog-summarize` to write these |

## When to write notes

Write proactively — don't wait to be asked. Log findings:

- **After fixing a bug** — record the root cause and fix with category `bugfix`
- **After an approach succeeds** — record what worked with category `whatworks`
- **After hitting a dead end** — record what failed with category `problems`
- **When you notice a pattern** — record it with category `patterns`
- **When you make a design call** — record the rationale with category `decisions`
- **When you find a gotcha** — record it with category `warnings`

## When to recall notes

Search alog **before starting any non-trivial task** — there may be prior findings that change your approach:

```bash
# Before investigating a bug
alog recall all "error message or symptom" --project=myproject

# Before choosing an approach
alog recall patterns "relevant keyword" --project=myproject

# Before touching a tricky area
alog recall warnings "module or subsystem name" --project=myproject
```

If results are noisy, narrow with `--threshold=70` (minimum 70% similarity) or `--count=5`.

## Session summaries

After a long work session (every 10–20 tool calls, or when the session ends), use `/alog-summarize` to write a summary of what was accomplished. This creates a `summaries` entry that can be exported later for human review.

## Exporting entries

Use `/alog-export` to generate Markdown reports from stored entries. For example:
- "Give me summaries of today's sessions" → export `--category=summaries --session=<id>`
- "Show me all bugs found in this project" → export `--project=myproject --category=bugfix`

## Entry writing guidelines

- Be specific and self-contained — a future agent has no session context
- Include relevant identifiers: file paths, function names, error text, crate names
- State *why* something works or fails, not just *what* happened
- Keep entries concise — one finding per entry; use multiple writes for multiple findings

## Example workflow

```bash
# Set session ID at the start of a work session
SESSION_ID="$(date +%Y%m%d)-abc123"
PROJECT=$(basename $(git rev-parse --show-toplevel))

# Before starting work — search for prior knowledge
alog recall all "authentication middleware" --project=$PROJECT --count=5

# After fixing a bug
alog write bugfix "tokio runtime panicked with 'cannot block the async runtime' — was calling .unwrap() on a blocking read inside an async fn. Fix: wrap with tokio::task::spawn_blocking." --project=$PROJECT --session=$SESSION_ID

# After discovering a pattern
alog write patterns "Error types in this codebase use thiserror derive macros with #[from] for automatic conversion. See src/errors.rs." --project=$PROJECT --session=$SESSION_ID

# At the end of the session, write a summary
alog write summaries "Implemented session tagging and export for alog. Added --session flag to write, new export subcommand with --category/--project/--session filters, and two Claude skills (alog-summarize, alog-export). All 48 tests pass." --project=$PROJECT --session=$SESSION_ID
```

## Consistency reminders

- Log findings **during** the task, not just at the end — insights are freshest in the moment
- A two-sentence entry written immediately is more valuable than a perfect entry written never
- If you recall entries that are stale or wrong, overwrite with `--replace=<id>` (the id is returned by `alog recall`)
- Prefer multiple narrow entries over one sprawling entry
- Always tag entries with `--session` so they can be grouped and exported later
