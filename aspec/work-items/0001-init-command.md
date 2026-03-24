# Work Item: Feature

Title: init command
Issue: issuelink

## Summary:
- init command to prepare a project's use of alog.

## User Stories

### User Story 1:
As a: user

I want to:
run `alog init`

So I can:
initialize my project's use of `alog`.


## Implementation Details:
- add a new `alog init` subcommand. the init subcommand should create a `.alog.json` configuration file (with no consent needed). If an `aspec` directory exists within the current git root, add the file to `GITROOT/aspec/.alog.json`. If there is no aspec folder, add the file to `GITROOT/.aspec.json. The file should contain one field: `defaultSimilarityThreshold`, and the value should be set to 25.
- whenever `alog search` is run, check if there is a `.alog.json` in the current GITROOT or GITROOT/aspec folder. If so, adopt the `defaultSimilarityThreshold` setting and only return search results with at least that similarity (25 == 25%). The `--threshold` flag should override the default setting.
- The `init` subcommand should also offer to download `.claude/skills/alog.md` from the alog github repository (latest main branch commit) and install it in the current project's .claude/skills directory. ask the user explicit permission before doing so.


## Edge Case Considerations:
- considerations

## Test Considerations:
- considerations

## Codebase Integration:
- follow established conventions, best practices, testing, and architecture patterns from the project's aspec.
