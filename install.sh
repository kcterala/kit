#!/bin/bash

set -e

echo "Installing kit..."

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin)
        if [ "$ARCH" = "arm64" ]; then
            BINARY_NAME="kit-macos-arm64"
        else
            BINARY_NAME="kit-macos-amd64"
        fi
        ;;
    *)
        echo "Error: Unsupported operating system: $OS"
        echo "Currently only macOS is supported."
        exit 1
        ;;
esac

# Get the latest release version
LATEST_VERSION=$(curl -s https://api.github.com/repos/kcterala/kit/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_VERSION" ]; then
    echo "Error: Could not determine latest version"
    exit 1
fi

echo "Downloading kit $LATEST_VERSION for $OS ($ARCH)..."

# Download the binary
DOWNLOAD_URL="https://github.com/kcterala/kit/releases/download/$LATEST_VERSION/$BINARY_NAME"
TMP_FILE="/tmp/kit"

curl -L -o "$TMP_FILE" "$DOWNLOAD_URL"

if [ $? -ne 0 ]; then
    echo "Error: Failed to download kit"
    exit 1
fi

# Determine installation directory - try multiple paths
INSTALL_DIRS=(
    "/usr/local/bin"
    "$HOME/.local/bin"
    "$HOME/bin"
    "$HOME/.bin"
)

INSTALL_DIR=""
for dir in "${INSTALL_DIRS[@]}"; do
    mkdir -p "$dir" 2>/dev/null || true
    if [ -d "$dir" ] && [ -w "$dir" ]; then
        INSTALL_DIR="$dir"
        break
    fi
done

if [ -z "$INSTALL_DIR" ]; then
    echo "Error: Could not find a writable installation directory"
    echo "Tried the following paths:"
    for dir in "${INSTALL_DIRS[@]}"; do
        echo "  - $dir"
    done
    exit 1
fi

# Install the binary
echo "Installing to $INSTALL_DIR..."
mv "$TMP_FILE" "$INSTALL_DIR/kit"
chmod +x "$INSTALL_DIR/kit"

# Check if install directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "Warning: $INSTALL_DIR is not in your PATH"
    echo "Add this line to your ~/.bashrc or ~/.zshrc:"
    echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
fi

echo ""
echo "kit installed successfully!"
echo ""
echo "Usage:"
echo "  kit clone <repo-url>    # Clone a repository"
echo "  kit fork <repo-url>     # Fork a repository (coming soon)"
echo ""
echo "On first use, you'll be prompted to authenticate with GitHub."
