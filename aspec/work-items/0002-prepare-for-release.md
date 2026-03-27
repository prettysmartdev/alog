# Work Item: Task

Title: prepare for release
Issue: issuelink

## Summary:
- prepare alog for its first release


## Implementation Details:
- ensure Github CI workflows are set up for test (on every commit to every branch)
- ensure Github CI release workflows are set up for all vX.Y.Z tag that is pushed
	- binaries for mac, windows, linux should be built (amd64 and ARM) and published to GH releases. The binaries should all be just "alog" or "alog.exe", not "alog-macos-aarch64". They can be put in tarballs with the platform/arch names, but the binaries within should just be "alog".
	- use the prettysmartdev/amux repo for inspiration.
- copy the local workflow from the `prettysmartdev/amux` repo (the `make release` command and shell script).
- add the tests badge to the README just like amux repo has (below the logo image).
- make sure readme, docs, and getting started guides are up to date, including tool and Claude skill installs.


## Edge Case Considerations:
- pretty much follow what amux does in every way.

## Test Considerations:
- considerations

## Codebase Integration:
- follow established conventions, best practices, testing, and architecture patterns from the project's aspec.
