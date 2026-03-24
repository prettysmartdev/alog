.PHONY: all build install test clean

all: build

build:
	cargo build --release

install: build
	cp target/release/alog /usr/local/bin/alog

test:
	cargo test

clean:
	cargo clean
