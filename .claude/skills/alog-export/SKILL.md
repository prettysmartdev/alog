---
name: alog-export
description: Export alog entries to a Markdown report. Use when the user wants to review, share, or summarize stored alog entries.
---

# alog-export — Export Skill

Generate a Markdown report of alog entries matching the user's request.

## Command

```bash
alog export <output-path> [--project=<name>] [--category=<name>] [--session=<id>]
```

Pass `-` as the output path to write to stdout instead of a file.

## How to use

1. Understand what the user wants exported. They may ask for:
   - Entries from a specific session: `--session=<id>`
   - Entries from a specific project: `--project=<name>`
   - Entries of a specific category: `--category=<name>`
   - Any combination of the above

2. Determine the output destination:
   - If the user wants to see it in the conversation: use `-` (stdout)
   - If the user wants a file: use the path they specify, or suggest a sensible default like `$HOME/alog-export-$(date +%Y%m%d).md`

3. Run the export command with the appropriate flags.

4. If writing to stdout, present the output to the user directly.
   If writing to a file, confirm the path and offer to show a preview.

## Examples

```bash
# Show all entries for a project in the conversation
alog export - --project=myapp

# Export summaries from work items 21, 22, and 23 to a file
alog export ~/reports/wi-21-22-23.md --category=summaries

# Export everything from a specific session
alog export - --session=sess-20260327-abc123

# Export bugs from a specific project to a file
alog export ~/bugs-report.md --project=myapp --category=bugfix
```

## Guidelines

- Use `--category=summaries` to show high-level session summaries
- Combine filters to narrow results — the fewer entries, the more readable the report
- When the user asks for "entries from work item N", search for session IDs or project names that correspond to that work item using `alog recall` first if needed
