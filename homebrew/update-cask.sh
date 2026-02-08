#!/usr/bin/env bash
#
# Update the Homebrew cask with SHA256 values from a GitHub release.
#
# Usage:
#   ./homebrew/update-cask.sh v0.1.0
#
# Prerequisites:
#   - gh CLI installed and authenticated
#   - Release artifacts already uploaded to GitHub
#
set -euo pipefail

VERSION="${1:?Usage: $0 <version-tag>}"
REPO="${GITHUB_REPOSITORY:-anthropics/claude-session-monitor}"
CASK_FILE="homebrew/claude-session-monitor.rb"

echo "Downloading release artifacts for ${VERSION}..."

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# Download the .app.tar.gz files
gh release download "$VERSION" \
  --repo "$REPO" \
  --pattern "*.app.tar.gz" \
  --dir "$TMPDIR"

# Compute SHA256
AARCH64_SHA=$(shasum -a 256 "$TMPDIR"/*aarch64*.tar.gz | awk '{print $1}')
X86_64_SHA=$(shasum -a 256 "$TMPDIR"/*x86_64*.tar.gz | awk '{print $1}')
CLEAN_VERSION="${VERSION#v}"

echo "aarch64 SHA256: ${AARCH64_SHA}"
echo "x86_64  SHA256: ${X86_64_SHA}"

# Update the cask file
sed -i '' "s/version \".*\"/version \"${CLEAN_VERSION}\"/" "$CASK_FILE"
sed -i '' "s/sha256 \"REPLACE_WITH_AARCH64_SHA256\"/sha256 \"${AARCH64_SHA}\"/" "$CASK_FILE"
sed -i '' "s/sha256 \"REPLACE_WITH_X86_64_SHA256\"/sha256 \"${X86_64_SHA}\"/" "$CASK_FILE"

# Also update any previously set hashes
sed -i '' "/on_arm/,/end/{s/sha256 \"[a-f0-9]\{64\}\"/sha256 \"${AARCH64_SHA}\"/;}" "$CASK_FILE"
sed -i '' "/on_intel/,/end/{s/sha256 \"[a-f0-9]\{64\}\"/sha256 \"${X86_64_SHA}\"/;}" "$CASK_FILE"

echo ""
echo "Updated ${CASK_FILE}:"
echo "  version: ${CLEAN_VERSION}"
echo "  aarch64: ${AARCH64_SHA}"
echo "  x86_64:  ${X86_64_SHA}"
echo ""
echo "Next steps:"
echo "  1. Copy ${CASK_FILE} to your homebrew-tap repo as Casks/claude-session-monitor.rb"
echo "  2. Commit and push the tap repo"
