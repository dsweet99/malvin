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

.PHONY: all install test deps

deps:
	@echo "Build deps (Ubuntu): sudo apt-get install gcc-10 g++-10 libcap-ng-dev"

all:
	cargo build --release

install:
	cargo install --path . --force

test:
	cargo nextest run
