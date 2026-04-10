.DEFAULT_GOAL := all

.PHONY: all install

all:
	cargo build --release

install:
	cargo install --path . --force
