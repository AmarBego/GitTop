#!/bin/bash
# Linux Installation Script for GitTop
# Installs binary, icon, and desktop file to user's local directories.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BINARY_PATH="$PROJECT_ROOT/target/release/gittop"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}GitTop Linux Installer${NC}"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}Error: Binary not found at $BINARY_PATH${NC}"
    echo "Please run: cargo build --release"
    exit 1
fi

# define paths
INSTALL_BIN="$HOME/.local/bin"
INSTALL_APPS="$HOME/.local/share/applications"
INSTALL_ICONS="$HOME/.local/share/icons/hicolor"

# Create directories
mkdir -p "$INSTALL_BIN"
mkdir -p "$INSTALL_APPS"
mkdir -p "$INSTALL_ICONS/256x256/apps"
mkdir -p "$INSTALL_ICONS/512x512/apps"

echo "Installing binary..."
cp "$BINARY_PATH" "$INSTALL_BIN/gittop"
chmod +x "$INSTALL_BIN/gittop"

echo "Installing icons..."
cp "$PROJECT_ROOT/assets/images/GitTop-256x256.png" "$INSTALL_ICONS/256x256/apps/gittop.png"
cp "$PROJECT_ROOT/assets/images/GitTop-512x512.png" "$INSTALL_ICONS/512x512/apps/gittop.png"

echo "Installing desktop entry..."
cp "$PROJECT_ROOT/src/platform/resources/gittop.desktop" "$INSTALL_APPS/gittop.desktop"

# Update desktop file to ensure Exec name matches what we installed
# We assume 'gittop' is in PATH if installed to ~/.local/bin, but being explicit doesn't hurt if we want absolute path
# For now, keeping Exec=GitTop or Exec=gittop is fine if ~/.local/bin is in PATH.
# Standard sed to ensure Exec line is what we expect? 
# The source desktop file has "Exec=GitTop". Let's standardize on "gittop".
sed -i 's/^Exec=GitTop/Exec=gittop/' "$INSTALL_APPS/gittop.desktop"

echo "Updating caches..."
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$INSTALL_APPS" 2>/dev/null || true
fi

echo -e "${GREEN}Installation complete!${NC}"
echo ""
echo "Binary installed to: $INSTALL_BIN/gittop"
echo "Make sure $INSTALL_BIN is in your PATH."
echo ""
echo "You may need to log out and back in for the icon to appear in some environments."
