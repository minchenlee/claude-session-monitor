#!/usr/bin/env bash
#
# c9watch installer
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/minchenlee/c9watch/main/install.sh | bash
#
# This script:
#   1. Detects your OS and architecture
#   2. Downloads the latest release from GitHub
#   3. Installs c9watch (macOS: to /Applications, Linux: via package manager)
#
set -euo pipefail

REPO="minchenlee/c9watch"
APP_NAME="c9watch"
INSTALL_DIR="/Applications"

# --- Helpers ---

info()  { printf '\033[1;34m=>\033[0m %s\n' "$*"; }
error() { printf '\033[1;31mError:\033[0m %s\n' "$*" >&2; exit 1; }

# --- Detect OS and architecture ---

OS=$(uname -s)
ARCH=$(uname -m)

case "$ARCH" in
  arm64|aarch64) ARCH_LABEL="aarch64" ;;
  x86_64)        ARCH_LABEL="x86_64" ;;
  *)             error "Unsupported architecture: $ARCH" ;;
esac

case "$OS" in
  Darwin)
    PLATFORM="macos"
    info "Detected macOS ($ARCH_LABEL)"
    ;;
  Linux)
    PLATFORM="linux"
    info "Detected Linux ($ARCH_LABEL)"
    ;;
  *)
    error "Unsupported operating system: $OS"
    ;;
esac

# --- Find latest release ---

info "Fetching latest release..."

LATEST_TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name":' \
  | head -1 \
  | cut -d'"' -f4)

if [ -z "$LATEST_TAG" ]; then
  error "Could not determine the latest release. Check https://github.com/${REPO}/releases"
fi

info "Latest version: ${LATEST_TAG}"

# --- Platform-specific installation ---

if [ "$PLATFORM" = "macos" ]; then
  # --- macOS: Download and install DMG ---

  DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${APP_NAME}_${LATEST_TAG}_${ARCH_LABEL}.dmg"
  TMPDIR=$(mktemp -d)
  trap 'rm -rf "$TMPDIR"' EXIT

  info "Downloading ${APP_NAME} for ${ARCH_LABEL}..."
  curl -fSL --progress-bar "$DOWNLOAD_URL" -o "$TMPDIR/${APP_NAME}.dmg"

  info "Mounting DMG..."
  MOUNT_POINT=$(hdiutil attach "$TMPDIR/${APP_NAME}.dmg" -nobrowse -noverify | grep "/Volumes/" | tail -1 | awk '{print $3}')

  if [ -z "$MOUNT_POINT" ]; then
    error "Failed to mount DMG"
  fi

  # Ensure we unmount on exit
  trap 'hdiutil detach "$MOUNT_POINT" -quiet 2>/dev/null || true; rm -rf "$TMPDIR"' EXIT

  # Find the .app bundle
  APP_BUNDLE=$(find "$MOUNT_POINT" -maxdepth 1 -name "*.app" -type d | head -1)
  if [ -z "$APP_BUNDLE" ]; then
    error "No .app bundle found in DMG"
  fi

  APP_BASENAME=$(basename "$APP_BUNDLE")

  # Remove old version if it exists
  if [ -d "${INSTALL_DIR}/${APP_BASENAME}" ]; then
    info "Removing previous installation..."
    rm -rf "${INSTALL_DIR}/${APP_BASENAME}"
  fi

  info "Installing to ${INSTALL_DIR}/${APP_BASENAME}..."
  cp -R "$APP_BUNDLE" "${INSTALL_DIR}/"

  echo ""
  info "c9watch has been installed to ${INSTALL_DIR}/${APP_BASENAME}"
  info "You can launch it from Spotlight or by running:"
  echo ""
  echo "    open '${INSTALL_DIR}/${APP_BASENAME}'"
  echo ""

elif [ "$PLATFORM" = "linux" ]; then
  # --- Linux: Download and install package ---

  # Detect package format preference
  if command -v dpkg &> /dev/null; then
    PKG_FORMAT="deb"
    PKG_EXT="deb"
  else
    PKG_FORMAT="appimage"
    PKG_EXT="AppImage"
  fi

  # Construct download URL
  if [ "$PKG_FORMAT" = "deb" ]; then
    # DEB packages are usually named: c9watch_<version>_<arch>.deb
    # But Tauri might generate different naming, so we'll try to fetch it
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${APP_NAME}_${LATEST_TAG#v}_amd64.deb"
  else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${APP_NAME}_${LATEST_TAG#v}_amd64.${PKG_EXT}"
  fi

  TMPDIR=$(mktemp -d)
  trap 'rm -rf "$TMPDIR"' EXIT

  info "Downloading ${APP_NAME} (${PKG_FORMAT})..."
  PKG_FILE="$TMPDIR/${APP_NAME}.${PKG_EXT}"

  if ! curl -fSL --progress-bar "$DOWNLOAD_URL" -o "$PKG_FILE"; then
    error "Failed to download ${PKG_FORMAT} package. URL: ${DOWNLOAD_URL}"
  fi

  # Install based on package format
  if [ "$PKG_FORMAT" = "deb" ]; then
    info "Installing DEB package (requires sudo)..."
    sudo dpkg -i "$PKG_FILE" || {
      info "Fixing dependencies..."
      sudo apt-get install -f -y
    }
    echo ""
    info "c9watch has been installed successfully!"
    info "Launch it with: c9watch"
    echo ""
  else
    # AppImage installation
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"

    INSTALL_PATH="$INSTALL_DIR/${APP_NAME}"

    info "Installing AppImage to ${INSTALL_PATH}..."
    cp "$PKG_FILE" "$INSTALL_PATH"
    chmod +x "$INSTALL_PATH"

    echo ""
    info "c9watch has been installed to ${INSTALL_PATH}"
    info "Make sure ${INSTALL_DIR} is in your PATH, then launch with: c9watch"
    echo ""

    # Check if in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
      echo "To add ${INSTALL_DIR} to your PATH, add this to your shell rc file (~/.bashrc or ~/.zshrc):"
      echo ""
      echo "    export PATH=\"\$PATH:${INSTALL_DIR}\""
      echo ""
    fi
  fi
fi
