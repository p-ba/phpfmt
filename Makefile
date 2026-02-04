.PHONY: all clean install rust rust-release

# The default target is 'all'.
default: rust-release

# Build Rust binary in debug mode
rust:
	cargo build

# Build Rust binary in release mode
rust-release:
	cargo build --release

# Install Rust binary to ~/.local/bin
install: rust-release
	@mkdir -p "$(HOME)/.local/bin"
	@cp target/release/frmt "$(HOME)/.local/bin/frmt"
	@chmod +x "$(HOME)/.local/bin/frmt"
	@echo "frmt installed in $(HOME)/.local/bin"

# Clean build artifacts
clean:
	cargo clean
	@if [ -f "$(HOME)/.local/bin/frmt" ]; then \
		echo "Removing $(HOME)/.local/bin/frmt"; \
		rm "$(HOME)/.local/bin/frmt"; \
	fi

# Run clippy linter
clippy:
	cargo clippy --all-targets --all-features

# Format code
fmt:
	cargo fmt

# Run tests
test:
	cargo test
