.PHONY: all build install test clean release

all: build

build:
	cargo build --release

install: build
	cp target/release/alog /usr/local/bin/alog

test:
	cargo test

clean:
	cargo clean

release:
	@if [ -z "$(VERSION)" ]; then \
		echo "Usage: make release VERSION=vX.Y.Z"; \
		exit 1; \
	fi
	@if [ -n "$$(git status --porcelain)" ]; then \
		echo "Error: working tree is not clean. Commit or stash changes first."; \
		exit 1; \
	fi
	@NOTES_FILE="docs/releases/$(VERSION).md"; \
	if [ ! -f "$$NOTES_FILE" ]; then \
		echo "Error: release notes not found at $$NOTES_FILE"; \
		echo "Create the file with release notes before tagging."; \
		exit 1; \
	fi
	make test
	git tag "$(VERSION)"
	git push origin main
	git push origin "$(VERSION)"
