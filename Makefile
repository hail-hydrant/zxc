mkscripts := $(CURDIR)/mkscripts
DIR := zxc
REQUIRED-BINS := getfattr openssl tmux vim
CARGO-VERSION := $(shell cargo --version 2>/dev/null)

all: check-binaries vim-features configuration ca build
push: check lint test fmt

help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

check-binaries: # Show help for each of the Makefile recipes.
	$(foreach bin,$(REQUIRED-BINS),\
		$(if $(shell command -v $(bin) 2> /dev/null),,$(error Please install `$(bin)`)))
	@echo "All binaries present"

vim-features: # Check for vim features
	@sh $(mkscripts)/check_vim_features.sh || { echo "Exiting due to missing vim feature"; exit 1; }

configuration: # Copy configuration
	@sh $(mkscripts)/zxc_config.sh
	@echo -n "Copy zxc's keymap configuration ? [y/N] " && read ans && \
		[ "$${ans:-N}" = "y" ] && { \
		sh $(mkscripts)/vim_config.sh; \
		sh $(mkscripts)/vim_ft_config.sh; \
		} || echo "Configuration not copied."

ca: # Generate ca certificate
	@sh $(mkscripts)/ca.sh

fmt: # Run cargo fmt
	@echo "Formatting....."
	@cargo +nightly fmt --all --check

lint: # Run cargo clippy
	@echo "Linting....."
	@cargo clippy --workspace --all-targets --all-features -- -Dwarnings

test: # Run cargo nextest
	@echo "Running tests....."
	@cargo nextest r

check: # Run cargo check
	@echo "Checking....."
	@cargo check

build: # Build binary
ifdef CARGO-VERSION
	@echo -n "Build Binaries ? [y/N] " && read ans && \
		[ "$${ans:-N}" = "y" ] && { \
		echo "Building....."; \
		cargo build --release; \
		} || echo "Binaries not built. Download binaries from github release"
else
	@echo "cargo not found. Cannot build binaries. Install cargo or download binaries from github release"
endif
