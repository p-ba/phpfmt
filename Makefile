.PHONY: all clean install

# Get current OS and Arch for the install target.
OS := $(shell uname -s | tr '[[:upper:]]' '[[:lower:]]')
ARCH := $(shell uname -m)

# The default target is 'all'.
default: all

# Build binaries for OSX and Linux platforms. # Binaries are placed in the 'bin' directory.
all:
	@echo "Building for OSX and Linux into ./bin/"
	@mkdir -p bin
	@go tool dist list | grep -E '^(darwin|linux)/' | while read -r line; do \
		os=$$(echo $$line | cut -d'/' -f1); \
		arch=$$(echo $$line | cut -d'/' -f2); \
		echo "  > building for $$os/$$arch"; \
		GOOS=$$os GOARCH=$$arch go build -o "bin/frmt-$$os-$$arch" main.go; \
	done

clean:
	@echo "Cleaning up..."
	@if [ -f "$(HOME)/.local/bin/frmt" ]; then \
		echo "Deleting executable from $(HOME)/.local/bin/frmt"; \
		rm "$(HOME)/.local/bin/frmt"; \
	fi

install: clean
	@echo "Installing for $(OS)/$(ARCH)..."
	@if [ ! -f "bin/frmt-$(OS)-$(ARCH)" ]; then \
		echo "Error: Binary for $(OS)/$(ARCH) does not exist in bin directory"; \
		echo "Please run 'make all' first or build the specific binary"; \
		exit 1; \
	fi
	@mkdir -p "$(HOME)/.local/bin"
	@echo "bin/frmt-$(OS)-$(ARCH)"
	@cp "bin/frmt-$(OS)-$(ARCH)" "$(HOME)/.local/bin/frmt"
	@chmod +x "$(HOME)/.local/bin/frmt"
	@echo "frmt installed in $(HOME)/.local/bin"
