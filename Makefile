.DEFAULT_GOAL := all

.PHONY: all install test

all:
	cargo build --release

install:
	cargo install --path . --force

test:
	cargo nextest run
