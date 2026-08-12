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

deps:
	@echo "Build deps (Ubuntu): sudo apt-get install gcc-10 g++-10 libcap-ng-dev"
	@echo "SDK bridges need Node >= 22.13 (cursor)."
	@echo "cargo build / cargo install run build.rs (npm ci into ~/.malvin_home/sdk-bridges/ when needed)."
	@echo "Manual: npm ci && npm run build in cursor-sdk-bridge/"

bridges: $(CURSOR_BRIDGE_JS)

$(CURSOR_BRIDGE_JS): cursor-sdk-bridge/package.json cursor-sdk-bridge/package-lock.json \
		cursor-sdk-bridge/tsconfig.json $(wildcard cursor-sdk-bridge/src/*.ts)
	cd cursor-sdk-bridge && npm ci && npm run build

all: bridges
	cargo build --release

install: bridges
	cargo install --path . --force --locked

test: bridges
	pytest tests && cargo nextest run

clean:
	cargo clean
	rm -rf cursor-sdk-bridge/dist cursor-sdk-bridge/node_modules
