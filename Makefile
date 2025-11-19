# Makefile for wcurl
# Cross-platform build system

# Variables
CARGO := cargo
VERSION := 2025.11.9
PREFIX ?= /usr/local
INSTALL_DIR := $(PREFIX)/bin

# Detect OS
UNAME_S := $(shell uname -s 2>/dev/null || echo Windows)
UNAME_M := $(shell uname -m 2>/dev/null || echo x86_64)

ifeq ($(UNAME_S),Linux)
    ifeq ($(UNAME_M),aarch64)
        CURRENT_TARGET := aarch64-unknown-linux-gnu
    else
        CURRENT_TARGET := x86_64-unknown-linux-gnu
    endif
    BINARY_NAME := wcurl
endif
ifeq ($(UNAME_S),Darwin)
    ifeq ($(UNAME_M),arm64)
        CURRENT_TARGET := aarch64-apple-darwin
    else
        CURRENT_TARGET := x86_64-apple-darwin
    endif
    BINARY_NAME := wcurl
endif
ifeq ($(UNAME_S),FreeBSD)
    ifeq ($(UNAME_M),aarch64)
        CURRENT_TARGET := aarch64-unknown-freebsd
    else
        CURRENT_TARGET := x86_64-unknown-freebsd
    endif
    BINARY_NAME := wcurl
endif
ifeq ($(UNAME_S),OpenBSD)
    ifeq ($(UNAME_M),aarch64)
        CURRENT_TARGET := aarch64-unknown-openbsd
    else
        CURRENT_TARGET := x86_64-unknown-openbsd
    endif
    BINARY_NAME := wcurl
endif
ifeq ($(UNAME_S),Windows)
    CURRENT_TARGET := x86_64-pc-windows-gnu
    BINARY_NAME := wcurl.exe
endif

# Build targets - all platforms and architectures
TARGETS := \
    x86_64-pc-windows-gnu \
    aarch64-pc-windows-msvc \
    x86_64-unknown-linux-gnu \
    aarch64-unknown-linux-gnu \
    x86_64-unknown-freebsd \
    aarch64-unknown-freebsd \
    x86_64-unknown-openbsd \
    aarch64-unknown-openbsd \
    x86_64-apple-darwin \
    aarch64-apple-darwin

.PHONY: all build release clean install uninstall test check help install-targets dist

# Default target
all: dist

# Build for current platform
build:
	$(CARGO) build --release

# Alias for build
release: build

# Install Rust targets for cross-compilation
install-targets:
	@echo "Installing Rust targets..."
	rustup target add x86_64-pc-windows-gnu
	rustup target add x86_64-unknown-linux-gnu
	rustup target add aarch64-unknown-linux-gnu
	rustup target add x86_64-unknown-freebsd
	@echo "✓ Targets installed"

# Build for all platforms
dist: install-targets
	@echo "Building for all platforms..."
	@mkdir -p dist
	
	@echo "Building Windows amd64..."
	@$(CARGO) build --release --target x86_64-pc-windows-gnu
	@cp target/x86_64-pc-windows-gnu/release/wcurl.exe dist/wcurl-$(VERSION)-windows-amd64.exe 2>/dev/null || true
	
	@echo "Building Linux amd64..."
	@$(CARGO) build --release --target x86_64-unknown-linux-gnu
	@cp target/x86_64-unknown-linux-gnu/release/wcurl dist/wcurl-$(VERSION)-linux-amd64 2>/dev/null || true
	@chmod +x dist/wcurl-$(VERSION)-linux-amd64 2>/dev/null || true
	
	@echo "Building Linux arm64..."
	@$(CARGO) build --release --target aarch64-unknown-linux-gnu
	@cp target/aarch64-unknown-linux-gnu/release/wcurl dist/wcurl-$(VERSION)-linux-arm64 2>/dev/null || true
	@chmod +x dist/wcurl-$(VERSION)-linux-arm64 2>/dev/null || true
	
	@echo "Building FreeBSD amd64..."
	@$(CARGO) build --release --target x86_64-unknown-freebsd
	@cp target/x86_64-unknown-freebsd/release/wcurl dist/wcurl-$(VERSION)-freebsd-amd64 2>/dev/null || true
	@chmod +x dist/wcurl-$(VERSION)-freebsd-amd64 2>/dev/null || true
	
	@echo ""
	@echo "✓ Build complete! Binaries in dist/:"
	@ls -lh dist/ 2>/dev/null || dir dist

# Clean build artifacts
clean:
	$(CARGO) clean
	rm -rf dist

# Install to system (Unix-like systems only)
install: build
	@echo "Installing wcurl to $(INSTALL_DIR)..."
	@mkdir -p $(INSTALL_DIR)
	@cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/wcurl
	@chmod +x $(INSTALL_DIR)/wcurl
	@echo "✓ Installed to $(INSTALL_DIR)/wcurl"

# Uninstall from system
uninstall:
	@echo "Removing wcurl from $(INSTALL_DIR)..."
	@rm -f $(INSTALL_DIR)/wcurl
	@echo "✓ Uninstalled"

# Run tests
test:
	$(CARGO) test

# Check code without building
check:
	$(CARGO) check
	$(CARGO) clippy -- -D warnings

# Format code
fmt:
	$(CARGO) fmt

# Run the program (for testing)
run:
	$(CARGO) run -- --help

# Create release archives
package: dist
	@echo "Creating release archives..."
	@cd dist && \
	tar czf wcurl-$(VERSION)-windows-amd64.tar.gz wcurl-$(VERSION)-windows-amd64.exe 2>/dev/null || true && \
	tar czf wcurl-$(VERSION)-linux-amd64.tar.gz wcurl-$(VERSION)-linux-amd64 2>/dev/null || true && \
	tar czf wcurl-$(VERSION)-linux-arm64.tar.gz wcurl-$(VERSION)-linux-arm64 2>/dev/null || true && \
	tar czf wcurl-$(VERSION)-freebsd-amd64.tar.gz wcurl-$(VERSION)-freebsd-amd64 2>/dev/null || true
	@echo "✓ Archives created in dist/"

# Show help
help:
	@echo "wcurl Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  make build          - Build for current platform"
	@echo "  make all / dist     - Build for all supported platforms"
	@echo "  make install-targets- Install Rust cross-compilation targets"
	@echo "  make clean          - Remove build artifacts"
	@echo "  make install        - Install to $(INSTALL_DIR)"
	@echo "  make uninstall      - Remove from $(INSTALL_DIR)"
	@echo "  make test           - Run tests"
	@echo "  make check          - Check code quality"
	@echo "  make fmt            - Format code"
	@echo "  make package        - Create release archives"
	@echo "  make help           - Show this help"
	@echo ""
	@echo "Variables:"
	@echo "  PREFIX              - Installation prefix (default: /usr/local)"
	@echo ""
	@echo "Examples:"
	@echo "  make                - Build all platforms"
	@echo "  make build          - Quick build for testing"
	@echo "  sudo make install   - Install system-wide"
