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
	@echo "cursor: models need Node >= 22.13 and npm at run time (not at build time)."
	@echo "malvin installs @cursor/sdk into ~/.malvinconf/sdk-bridges/ on first use or via 'malvin admin setup-cursor'."
	@echo "Manual: npm ci && npm run build in cursor-sdk-bridge/"

bridges: $(CURSOR_BRIDGE_JS)

$(CURSOR_BRIDGE_JS): cursor-sdk-bridge/package.json cursor-sdk-bridge/package-lock.json \
		cursor-sdk-bridge/tsconfig.json $(wildcard cursor-sdk-bridge/src/*.ts)
	cd cursor-sdk-bridge && npm ci && npm run build

# One release rustc was about 8 GiB RSS. The default job count is one per CPU
# (24 here), and two rustc processes together exhausted 15 GiB. Keep `make` at one job.
all: bridges
	cargo build --release --jobs 8

install: bridges
	cargo install --path . --force --locked --config 'build.rustflags=[]'

test: bridges
	pytest tests && cargo nextest run

clean:
	cargo clean
	rm -rf cursor-sdk-bridge/dist cursor-sdk-bridge/node_modules
