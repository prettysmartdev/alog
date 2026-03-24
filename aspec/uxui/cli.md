# CLI Design

Binary name: alog
Install path: /usr/local/bin/
Storage location: $HOME/.alog/

## Design principles:

### Command structure
Top level command groups:
- `alog write <category> <log entry>`: save a new log entry for the given category
	- flags:
		- --project=<name>: set the project associated with the log item
		- --replace=<id>: new log item should be added and the indicated log item should be deleted
- `alog recall <category|all> <search term>`: fuzzy search for the given search term, return results ranked by relevance
	- flags:
		- --project=<name>: restrict the search to the specific project
		- --count=<n>: restrict the number of results
		- --threshold=<n>: the minimum percentage similarity a log item must reach to be considered

### Flag structure
Flag guidance:
- follow above

### Inputs and outputs
I/O Guidance:
- stdin
- stdout
- stderr

### Configuration
Global config:
- store configuration for a specific Git repo within a JSON file: GITROOT/.alog.json
- store global config within `$HOME/.alog/config.json
- store log JSON files within `$HOME/.alog/logbook/{projectname|global}/{categoryname}.json`