.DEFAULT_GOAL := all

# aws-lc-sys (via microsandbox → rustls) rejects GCC 9's memcmp bug; prefer GCC 10+ when installed.
ifneq (,$(wildcard /usr/bin/gcc-10))
export CC := gcc-10
export CXX := g++-10
endif

# libcap-ng: runtime lib is .so.0; linker needs libcap-ng.so from libcap-ng-dev
ifneq (,$(wildcard /usr/lib/x86_64-linux-gnu/libcap-ng.so))
else ifneq (,$(wildcard /lib/x86_64-linux-gnu/libcap-ng.so.0))
MALVIN_LINK_DIR := $(CURDIR)/target/.link-stubs
$(shell mkdir -p $(MALVIN_LINK_DIR) && ln -sf /lib/x86_64-linux-gnu/libcap-ng.so.0 $(MALVIN_LINK_DIR)/libcap-ng.so)
export LIBRARY_PATH := $(MALVIN_LINK_DIR)$(if $(LIBRARY_PATH),:$(LIBRARY_PATH))
endif

.PHONY: all install test deps bridges clean

CURSOR_BRIDGE_JS := cursor-sdk-bridge/dist/bridge.js
PRIME_BRIDGE_JS := prime-sdk-bridge/dist/bridge.js

deps:
	@echo "Build deps (Ubuntu): sudo apt-get install gcc-10 g++-10 libcap-ng-dev"
	@echo "SDK bridges also need Node >= 22.13 (cursor) / >= 22.8 (prime): npm ci && npm run build in each *-sdk-bridge/"

bridges: $(CURSOR_BRIDGE_JS) $(PRIME_BRIDGE_JS)

$(CURSOR_BRIDGE_JS): cursor-sdk-bridge/package.json cursor-sdk-bridge/package-lock.json \
		cursor-sdk-bridge/tsconfig.json $(wildcard cursor-sdk-bridge/src/*.ts)
	cd cursor-sdk-bridge && npm ci && npm run build

$(PRIME_BRIDGE_JS): prime-sdk-bridge/package.json prime-sdk-bridge/package-lock.json \
		prime-sdk-bridge/tsconfig.json $(wildcard prime-sdk-bridge/src/*.ts)
	cd prime-sdk-bridge && npm ci && npm run build

all: bridges
	cargo build --release

install: bridges
	cargo install --path . --force --locked

test: bridges
	pytest tests && cargo nextest run

clean:
	cargo clean
	rm -rf cursor-sdk-bridge/dist cursor-sdk-bridge/node_modules \
		prime-sdk-bridge/dist prime-sdk-bridge/node_modules
