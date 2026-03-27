---
name: alog-summarize
description: Synthesize a summary of all work done during the current session and write it to alog under the "summaries" category.
---

# alog-summarize — Session Summary Skill

Write a concise summary of everything you did during this work session and save it to alog.

## Steps

1. Identify the project name:
   ```bash
   basename $(git rev-parse --show-toplevel)
   ```

2. Use the current Claude session ID as the session value (it is available as `$CLAUDE_SESSION_ID` in hooks, or use the value you were given at session start). If no session ID is available, generate a short unique string such as the current date/time: `date +%Y%m%d-%H%M`.

3. Synthesize a summary covering:
   - What was asked / the goal
   - What was implemented or changed (be specific: file names, functions, commands)
   - Key decisions made and why
   - Problems encountered and how they were resolved
   - Anything left incomplete or that needs follow-up

4. Write the summary to alog:
   ```bash
   alog write summaries "<your summary here>" --project=<projectname> --session=<session-id>
   ```

## Guidelines

- Be specific and self-contained — this summary will be read by a future agent with no context
- Include file paths, function names, error messages, and command output where relevant
- Keep it to a few focused sentences or short bullet points; avoid padding
- Write one summary per session; if the session covered multiple independent tasks, write one entry per task

## When to use

- At the end of a long work session
- Before handing off to another agent
- When the user asks for a session summary
- Periodically after completing a significant chunk of work (i.e. if a task results in big code and docs changes, summarize after each major category of work)
