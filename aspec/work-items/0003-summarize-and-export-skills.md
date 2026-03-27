# Work Item: Feature

Title: summarize and export skills
Issue: issuelink

## Summary:
- add the ability to summarize an entire agent work session into alog, and to export into a file.

## User Stories

### User Story 1:
As a: user

I want to:
run `/alog-summarize` and `/alog-export` inside my agent

So I can:
trigger summary creation into alog, and export all alog activity from a session into a file for human review


## Implementation Details:
- add a new `--session` flag to `alog write` which adds session metadata to any log written. The session value can be anything, including a number, a UUID, or a random string. Cap it at 100 chars. In the JSON logfile, add it as a new field with each entry. Update the Claude skill to use this session with Claude's session ID.
- add a new `alog export` subcommand which exports alog entries into a markdown file, formatted nicely. The command should take `--category`, `--project` or `--session` flags to filter what gets exported, plus a filename to export to. 
	- e.g. `alog export $HOME/file/name.md --session=asdf --project=amux --category=bugs`, which would cause alog to create a markdown file at the given path and write all log entries that match the provided filters. Format the markdown nicely so it's human readable, without too much fluff or extra formatting.
	- if `-` is passed as the filename argument, write the output to stdout instead of a file
- add two new claude skills, `alog-summarize` and `alog-export`
	- the summarize command should prompt the agent to synthesize a summary of all the work that was done during the current session, and write it to an alog entry using the `summaries` category, and the appropriate --project and --session flags. Update the main alog skill to direct Claude to do this regularly after a long work session.
	- the export command should allow the user to request any kind of report about what is stored by alog, like "/alog-export give me summaries of work items 21, 22, and 23" and the Claude skill should direct Claude to run `alog export` to give the user what they want in nice Markdown.


## Edge Case Considerations:
- considerations

## Test Considerations:
- considerations

## Codebase Integration:
- follow established conventions, best practices, testing, and architecture patterns from the project's aspec.
