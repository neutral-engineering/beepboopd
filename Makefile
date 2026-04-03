BINARY = target/debug/beepboopd
RELEASE = target/release/beepboopd
INSTALL_DIR = $(HOME)/.local/bin
SYSTEMD_DIR = $(HOME)/.config/systemd/user

.PHONY: help build release install installcheck uninstall all check \
	beep clock chords scale zelda jazz classical

help: ## Show this help
	@grep -E '^[a-zA-Z0-9_-]+:.*##' $(MAKEFILE_LIST) | awk -F ':.*## ' '{printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}'

all: build ## Play all styles: beep, clock, chords, scale, zelda, jazz, help
	$(BINARY) beep success
	$(BINARY) beep failure
	$(BINARY) clock
	$(BINARY) chords
	$(BINARY) scale
	$(BINARY) zelda
	$(BINARY) jazz
	$(BINARY) --help

build: ## Build the debug binary
	cargo build

release: ## Build the release binary
	cargo build --release

check: ## Run clippy + format check
	cargo clippy
	cargo fmt --check

install: release ## Install binary + service
	mkdir -p $(INSTALL_DIR)
	cp $(RELEASE) $(INSTALL_DIR)/
	$(INSTALL_DIR)/beepboopd install

installcheck: ## Show service status + recent logs
	$(INSTALL_DIR)/beepboopd status

uninstall: ## Stop and remove binary + service
	$(INSTALL_DIR)/beepboopd uninstall
	rm -f $(INSTALL_DIR)/beepboopd

beep: build ## Play beep (success + failure)
	$(BINARY) beep success
	$(BINARY) beep failure

clock: build ## Play clock at current hour
	$(BINARY) clock

chords: build ## Play chords at current hour
	$(BINARY) chords

scale: build ## Play scale at current hour
	$(BINARY) scale

zelda: build ## Play zelda at current hour
	$(BINARY) zelda

jazz: build ## Play the lick at current hour
	$(BINARY) jazz

classical: build ## Play classical piece (rotates by hour)
	$(BINARY) classical
