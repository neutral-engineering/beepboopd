BINARY = target/debug/beepboopd
RELEASE = target/release/beepboopd
INSTALL_DIR = $(HOME)/.local/bin
SYSTEMD_DIR = $(HOME)/.config/systemd/user

.PHONY: help build release install installcheck uninstall all check \
	beep clock chords scale zelda

help: ## Show this help
	@grep -E '^[a-zA-Z0-9_-]+:.*##' $(MAKEFILE_LIST) | awk -F ':.*## ' '{printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}'

all: build ## Play all styles: beep, clock, chords, scale, zelda, help
	$(BINARY) beep success
	$(BINARY) beep failure
	$(BINARY) clock
	$(BINARY) chords
	$(BINARY) scale
	$(BINARY) zelda
	$(BINARY) --help

build: ## Build the debug binary
	cargo build

release: ## Build the release binary
	cargo build --release

check: ## Run clippy + format check
	cargo clippy
	cargo fmt --check

install: release ## Install binary + systemd service
	-systemctl --user stop beepboopd.service 2>/dev/null
	mkdir -p $(INSTALL_DIR) $(SYSTEMD_DIR)
	cp $(RELEASE) $(INSTALL_DIR)/
	cp beepboopd.service $(SYSTEMD_DIR)/
	systemctl --user daemon-reload
	systemctl --user enable --now beepboopd.service

installcheck: ## Show service status + recent logs
	systemctl --user status beepboopd.service || true
	@echo ""
	journalctl --user -u beepboopd.service -n 10 --no-pager

uninstall: ## Stop and remove binary + systemd service
	systemctl --user disable --now beepboopd.service
	rm -f $(INSTALL_DIR)/beepboopd
	rm -f $(SYSTEMD_DIR)/beepboopd.service
	systemctl --user daemon-reload

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
