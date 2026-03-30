---
name: alog-summarize
description: Synthesize a summary of all work done during the current session and write it to alog under the "summaries" category.
---

# alog-summarize — Session Summary Skill

Write a concise summary of everything accomplished during this work session and save it to alog. **Run this automatically after completing any user prompt that involved file edits** — do not wait for the user to invoke it.

## Steps

1. Identify the project name:
   ```bash
   PROJECT=$(.claude/skills/alog/get-project.sh)
   ```

2. Use the current Claude session ID as the session value (it is available as `$CLAUDE_SESSION_ID` in hooks, or use the value you were given at session start). If no session ID is available, generate a short unique string such as the current date/time: `date +%Y%m%d-%H%M`.

3. Synthesize a summary covering:
   - What was asked / the goal
   - What was implemented or changed (be specific: file names, functions, commands)
   - Key decisions made and why
   - Problems encountered and how they were resolved
   - Anything left incomplete or that needs follow-up

4. Write or update the summary:
   - **First summary this session** — write normally and save the returned entry ID:
     ```bash
     alog write summaries "<summary>" --project=<name> --session=<session-id>
     ```
   - **Subsequent summaries in the same session** — use `--replace=<previous-id>` so only one summary entry exists per session. The previous ID is returned by the prior `alog write` call:
     ```bash
     alog write summaries "<updated summary covering everything so far>" --project=<name> --session=<session-id> --replace=<previous-id>
     ```

## Guidelines

- Be specific and self-contained — this summary will be read by a future agent with no context
- Include file paths, function names, error messages, and command output where relevant
- Keep it to a few focused sentences or short bullet points; avoid padding
- Maintain **one summary per session** — use `--replace` to update rather than creating multiple entries
- Each updated summary should cover the full session so far, not just the latest task

## When to use

- **Automatically after completing any user prompt that involved file edits** — write or update the session summary as part of finishing every task, without being asked
- At the end of a long work session or before handing off to another agent
- When the user explicitly asks for a session summary
