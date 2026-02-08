#!/usr/bin/env bash
#
# c9watch installer
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/minchenlee/c9watch/main/install.sh | bash
#
# This script:
#   1. Detects your Mac's architecture (Apple Silicon or Intel)
#   2. Downloads the latest signed DMG from GitHub
#   3. Installs c9watch.app to /Applications
#
set -euo pipefail

REPO="minchenlee/c9watch"
APP_NAME="c9watch"
INSTALL_DIR="/Applications"

# --- Helpers ---

info()  { printf '\033[1;34m=>\033[0m %s\n' "$*"; }
error() { printf '\033[1;31mError:\033[0m %s\n' "$*" >&2; exit 1; }

# --- Detect architecture ---

ARCH=$(uname -m)
case "$ARCH" in
  arm64|aarch64) ARCH_LABEL="aarch64" ;;
  x86_64)        ARCH_LABEL="x86_64" ;;
  *)             error "Unsupported architecture: $ARCH" ;;
esac

OS=$(uname -s)
case "$OS" in
  MINGW*|MSYS*|CYGWIN*)
    info "Windows detected. Please use the PowerShell installer instead:"
    echo ""
    echo "    irm https://raw.githubusercontent.com/${REPO}/main/install.ps1 | iex"
    echo ""
    exit 0
    ;;
  Darwin)
    # macOS - continue installation
    ;;
  *)
    error "This installer supports macOS only. For Windows, use install.ps1. Detected OS: $OS"
    ;;
esac

info "Detected macOS ($ARCH_LABEL)"

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

# --- Download ---

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${APP_NAME}_${LATEST_TAG}_${ARCH_LABEL}.dmg"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

info "Downloading ${APP_NAME} for ${ARCH_LABEL}..."
curl -fSL --progress-bar "$DOWNLOAD_URL" -o "$TMPDIR/${APP_NAME}.dmg"

# --- Mount and install ---

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

# --- Done ---

echo ""
info "c9watch has been installed to ${INSTALL_DIR}/${APP_BASENAME}"
info "You can launch it from Spotlight or by running:"
echo ""
echo "    open '${INSTALL_DIR}/${APP_BASENAME}'"
echo ""
