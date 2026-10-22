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

CURSOR_BRIDGE_JS := bridges/cursor-sdk-bridge/dist/bridge.js
MODAL_BRIDGE_JS := bridges/modal-bridge/dist/bridge.js

deps:
	@echo "Build deps (Ubuntu): sudo apt-get install gcc-10 g++-10 libcap-ng-dev"
	@echo "cursor: models need Node >= 22.13 and npm at run time (not at build time)."
	@echo "malvin installs @cursor/sdk into ~/.malvinconf/sdk-bridges/ on first use."
	@echo "Manual: npm ci && npm run build in bridges/cursor-sdk-bridge/"

bridges: $(CURSOR_BRIDGE_JS) $(MODAL_BRIDGE_JS)

$(CURSOR_BRIDGE_JS): bridges/cursor-sdk-bridge/package.json bridges/cursor-sdk-bridge/package-lock.json \
		bridges/cursor-sdk-bridge/tsconfig.json $(wildcard bridges/cursor-sdk-bridge/src/*.ts)
	cd bridges/cursor-sdk-bridge && npm ci && npm run build

$(MODAL_BRIDGE_JS): bridges/modal-bridge/package.json bridges/modal-bridge/package-lock.json \
		bridges/modal-bridge/tsconfig.json $(wildcard bridges/modal-bridge/src/*.ts)
	cd bridges/modal-bridge && npm ci && npm run build

# Job count and the rustc wrapper (admin/rustc_sccache.sh) come from .cargo/config.toml.
# release-local is the release profile plus incremental compilation (Cargo.toml).
all: bridges
	cargo build --profile release-local

install: bridges
	cargo install --path . --force --locked --config 'build.rustflags=[]'

test: bridges
	pytest tests && cargo nextest run

# bridges/*/dist is tracked so that building needs no Node; clean must keep it.
clean:
	cargo clean
	rm -rf bridges/cursor-sdk-bridge/node_modules bridges/modal-bridge/node_modules
