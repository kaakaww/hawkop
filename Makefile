# HawkOp Makefile
# Build automation for the HawkOp CLI

.PHONY: help build release test lint fmt check-fmt pre-commit clean install run
.PHONY: build-all build-linux-x64 build-linux-arm64 build-macos-intel build-macos-arm build-windows-x64 build-windows-arm64
.PHONY: dist checksums changelog changelog-preview
.PHONY: functional-test functional-test-dry-run

# Default target
.DEFAULT_GOAL := help

# Variables
BINARY_NAME := hawkop
VERSION := $(shell grep '^version' Cargo.toml | head -n1 | cut -d'"' -f2)
DIST_DIR := dist
PREFIX ?= /usr/local

# Colors for output
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
NC := \033[0m

## help: Show this help message
help:
	@echo "$(CYAN)HawkOp Build System$(NC)"
	@echo ""
	@echo "$(GREEN)Available targets:$(NC)"
	@sed -n 's/^##//p' $(MAKEFILE_LIST) | column -t -s ':' | sed -e 's/^/ /'

## build: Build debug binary
build:
	@echo "$(CYAN)Building debug binary...$(NC)"
	cargo build

## release: Run interactive release wizard
release:
	@echo "$(YELLOW)Tip: For better Ctrl+C handling, run directly: ./scripts/release.sh$(NC)"
	@./scripts/release.sh

## test: Run all tests
test:
	@echo "$(CYAN)Running tests...$(NC)"
	cargo test

## lint: Run clippy lints
lint:
	@echo "$(CYAN)Running clippy...$(NC)"
	cargo clippy -- -D warnings

## fmt: Format code
fmt:
	@echo "$(CYAN)Formatting code...$(NC)"
	cargo fmt

## check-fmt: Check code formatting without modifying
check-fmt:
	@echo "$(CYAN)Checking code formatting...$(NC)"
	cargo fmt -- --check

## pre-commit: Run all checks before committing (format, lint, test)
pre-commit:
	@echo "$(CYAN)╔════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(CYAN)║           Running Pre-Commit Checks                    ║$(NC)"
	@echo "$(CYAN)╚════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@echo "$(YELLOW)[1/4] Formatting code...$(NC)"
	@cargo fmt
	@echo "$(GREEN)✓ Code formatted$(NC)"
	@echo ""
	@echo "$(YELLOW)[2/4] Checking format...$(NC)"
	@cargo fmt -- --check && echo "$(GREEN)✓ Format check passed$(NC)" || (echo "$(RED)✗ Format check failed$(NC)" && exit 1)
	@echo ""
	@echo "$(YELLOW)[3/4] Running clippy...$(NC)"
	@cargo clippy -- -D warnings && echo "$(GREEN)✓ Clippy passed$(NC)" || (echo "$(RED)✗ Clippy failed$(NC)" && exit 1)
	@echo ""
	@echo "$(YELLOW)[4/4] Running tests...$(NC)"
	@cargo test && echo "$(GREEN)✓ Tests passed$(NC)" || (echo "$(RED)✗ Tests failed$(NC)" && exit 1)
	@echo ""
	@echo "$(GREEN)╔════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(GREEN)║          All pre-commit checks passed! ✓               ║$(NC)"
	@echo "$(GREEN)╚════════════════════════════════════════════════════════╝$(NC)"

## clean: Remove build artifacts
clean:
	@echo "$(CYAN)Cleaning build artifacts...$(NC)"
	cargo clean
	rm -rf $(DIST_DIR)

## install: Install binary to system (use PREFIX to customize location)
install: build-release
	@echo "$(CYAN)Installing $(BINARY_NAME) to $(PREFIX)/bin...$(NC)"
	install -d $(PREFIX)/bin
	install -m 755 target/release/$(BINARY_NAME) $(PREFIX)/bin/

## run: Run the binary (debug mode)
run:
	cargo run --

## build-release: Build optimized release binary
build-release:
	@echo "$(CYAN)Building release binary...$(NC)"
	cargo build --release

## build-all: Build for all supported platforms
build-all: build-linux-x64 build-linux-arm64 build-macos-intel build-macos-arm build-windows-x64 build-windows-arm64

## build-linux-x64: Build for Linux x86_64
build-linux-x64:
	@echo "$(CYAN)Building for Linux x86_64...$(NC)"
	cargo build --release --target x86_64-unknown-linux-gnu

## build-linux-arm64: Build for Linux ARM64
build-linux-arm64:
	@echo "$(CYAN)Building for Linux ARM64...$(NC)"
	cargo build --release --target aarch64-unknown-linux-gnu

## build-macos-intel: Build for macOS Intel
build-macos-intel:
	@echo "$(CYAN)Building for macOS Intel...$(NC)"
	cargo build --release --target x86_64-apple-darwin

## build-macos-arm: Build for macOS Apple Silicon
build-macos-arm:
	@echo "$(CYAN)Building for macOS Apple Silicon...$(NC)"
	cargo build --release --target aarch64-apple-darwin

## build-windows-x64: Build for Windows x86_64
build-windows-x64:
	@echo "$(CYAN)Building for Windows x86_64...$(NC)"
	cargo build --release --target x86_64-pc-windows-msvc

## build-windows-arm64: Build for Windows ARM64
build-windows-arm64:
	@echo "$(CYAN)Building for Windows ARM64...$(NC)"
	cargo build --release --target aarch64-pc-windows-msvc

## dist: Create distribution archives for all platforms
dist: clean build-all
	@echo "$(CYAN)Creating distribution archives...$(NC)"
	@mkdir -p $(DIST_DIR)
	@# Linux x86_64
	@tar -czf $(DIST_DIR)/$(BINARY_NAME)-v$(VERSION)-x86_64-unknown-linux-gnu.tar.gz \
		-C target/x86_64-unknown-linux-gnu/release $(BINARY_NAME) \
		-C ../../../ LICENSE README.md
	@# Linux ARM64
	@tar -czf $(DIST_DIR)/$(BINARY_NAME)-v$(VERSION)-aarch64-unknown-linux-gnu.tar.gz \
		-C target/aarch64-unknown-linux-gnu/release $(BINARY_NAME) \
		-C ../../../ LICENSE README.md
	@# macOS Intel
	@tar -czf $(DIST_DIR)/$(BINARY_NAME)-v$(VERSION)-x86_64-apple-darwin.tar.gz \
		-C target/x86_64-apple-darwin/release $(BINARY_NAME) \
		-C ../../../ LICENSE README.md
	@# macOS Apple Silicon
	@tar -czf $(DIST_DIR)/$(BINARY_NAME)-v$(VERSION)-aarch64-apple-darwin.tar.gz \
		-C target/aarch64-apple-darwin/release $(BINARY_NAME) \
		-C ../../../ LICENSE README.md
	@# Windows x86_64
	@cd target/x86_64-pc-windows-msvc/release && \
		zip -q $(CURDIR)/$(DIST_DIR)/$(BINARY_NAME)-v$(VERSION)-x86_64-pc-windows-msvc.zip \
		$(BINARY_NAME).exe && \
		cd $(CURDIR) && \
		zip -qj $(DIST_DIR)/$(BINARY_NAME)-v$(VERSION)-x86_64-pc-windows-msvc.zip LICENSE README.md
	@# Windows ARM64
	@cd target/aarch64-pc-windows-msvc/release && \
		zip -q $(CURDIR)/$(DIST_DIR)/$(BINARY_NAME)-v$(VERSION)-aarch64-pc-windows-msvc.zip \
		$(BINARY_NAME).exe && \
		cd $(CURDIR) && \
		zip -qj $(DIST_DIR)/$(BINARY_NAME)-v$(VERSION)-aarch64-pc-windows-msvc.zip LICENSE README.md
	@$(MAKE) checksums
	@echo "$(GREEN)Distribution archives created in $(DIST_DIR)/$(NC)"

## checksums: Generate SHA256 checksums for distribution archives
checksums:
	@echo "$(CYAN)Generating SHA256 checksums...$(NC)"
	@cd $(DIST_DIR) && shasum -a 256 *.tar.gz *.zip > checksums.txt
	@echo "$(GREEN)Checksums generated: $(DIST_DIR)/checksums.txt$(NC)"

## changelog: Extract changelog for a version (usage: make changelog or make changelog V=0.2.0)
changelog:
	@V=$${V:-$(VERSION)}; \
	if [ "$$V" = "Unreleased" ]; then \
		awk '/^## \[Unreleased\]/{found=1; next} /^## \[/{if(found) exit} /^\[.*\]:/{next} found{print}' CHANGELOG.md; \
	else \
		awk -v ver="$$V" '/^## \[/{if(found) exit; if($$0 ~ "\\["ver"\\]") found=1; next} /^\[.*\]:/{next} found{print}' CHANGELOG.md; \
	fi

## changelog-preview: Show changelog for current version with header
changelog-preview:
	@echo "$(CYAN)╔════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(CYAN)║       Changelog for v$(VERSION)$(NC)"
	@echo "$(CYAN)╚════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@$(MAKE) -s changelog
	@echo ""

## functional-test: Run functional tests against real API (uses HAWKOP_PROFILE)
functional-test:
	@echo "$(CYAN)╔════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(CYAN)║         Running Functional Tests                       ║$(NC)"
	@echo "$(CYAN)╚════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@if [ -z "$$HAWKOP_PROFILE" ]; then \
		echo "$(RED)Error: HAWKOP_PROFILE must be set$(NC)"; \
		echo "$(YELLOW)Example: HAWKOP_PROFILE=test make functional-test$(NC)"; \
		exit 1; \
	fi
	@echo "$(YELLOW)Using profile: $$HAWKOP_PROFILE$(NC)"
	@echo ""
	cargo test --features functional-tests --test functional -- --test-threads=1 --nocapture

## functional-test-dry-run: Preview functional tests (list only, no execution)
functional-test-dry-run:
	@echo "$(CYAN)╔════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(CYAN)║      Functional Tests Preview (List Only)              ║$(NC)"
	@echo "$(CYAN)╚════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@echo "$(YELLOW)The following tests would run:$(NC)"
	@echo ""
	@cargo test --features functional-tests --test functional -- --list 2>/dev/null | grep ": test$$" | sed 's/: test$$//' | while read test; do \
		echo "  $(GREEN)✓$(NC) $$test"; \
	done
	@echo ""
	@echo "$(CYAN)Total:$(NC) $$(cargo test --features functional-tests --test functional -- --list 2>/dev/null | grep -c ': test$$') tests"
	@echo ""
	@echo "$(YELLOW)To run these tests:$(NC)"
	@echo "  HAWKOP_PROFILE=test make functional-test"
