# Release Process

This document describes the complete process for releasing a new version of c9watch.

## Overview

c9watch uses Tauri's built-in auto-updater mechanism. The release workflow:

1. **Version bump** → Update version strings in config files
2. **Contributor recognition** → Credit contributors for their work
3. **Git tag** → Push a tag matching `v*` pattern (e.g., `v0.2.0`)
4. **CI/CD automation** → GitHub Actions builds artifacts and creates a draft release
5. **Testing** → Test the draft release builds
6. **Publishing** → Publish the release to trigger auto-updates

The updater endpoint (`https://github.com/minchenlee/c9watch/releases/latest/download/latest.json`) automatically serves the latest **published** release. Draft releases and pre-release tags don't affect the updater.

## Version Numbering

c9watch follows [Semantic Versioning](https://semver.org/):

- **Major** (`x.0.0`): Breaking changes, incompatible API changes, or significant rewrites
- **Minor** (`0.x.0`): New features, backward-compatible functionality additions
- **Patch** (`0.0.x`): Bug fixes, small improvements, backward-compatible changes

Examples:
- Adding WebSocket mobile client support: `0.1.0` → `0.2.0` (new feature)
- Fixing a crash in session listing: `0.1.0` → `0.1.1` (bug fix)
- Changing the session file format incompatibly: `0.1.0` → `1.0.0` (breaking change)

## Pre-Release Checklist

Before starting the release process, ensure:

- [ ] All intended PRs are merged to `main`
- [ ] All tests pass locally (`npm run check`, Rust tests if any)
- [ ] CI is green on `main`
- [ ] No known critical bugs in the current `main` branch
- [ ] You have decided on the version number (following semver)

## Release Steps

### 1. Create Release Preparation Branch

Create a dedicated branch for the release preparation:

```bash
git checkout main
git pull origin main
git checkout -b release/vX.Y.Z-prep
```

This keeps the release preparation organized and allows for review before merging.

### 2. Update Version Numbers

Update the version in **three** files. The version string must be **identical** in all three:

#### a) `package.json` (line 3)

```json
{
  "name": "c9watch",
  "version": "X.Y.Z",  // Update this line
  "description": "...",
  ...
}
```

#### b) `src-tauri/tauri.conf.json` (line 4)

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "c9watch",
  "version": "X.Y.Z",  // Update this line
  "identifier": "com.minchenlee.c9watch",
  ...
}
```

#### c) `src-tauri/Cargo.toml` (line 3)

```toml
[package]
name = "c9watch"
version = "X.Y.Z"  # Update this line
description = "Monitor and control all your Claude Code sessions from one place"
...
```

**Important:** After editing `Cargo.toml`, run `cargo check` in the `src-tauri` directory to update `Cargo.lock`:

```bash
cd src-tauri
cargo check
cd ..
git add src-tauri/Cargo.lock
```

### 3. Update Contributor Recognition

If this release includes contributions from external contributors (merged PRs), update contributor recognition in **three** places:

#### a) `CONTRIBUTORS.md`

Add contributors under the appropriate category:

```markdown
## Features
- **Contributor Name** ([@username](https://github.com/username)) - Description of contribution ([#PR](https://github.com/minchenlee/c9watch/pull/PR))
```

Categories include:
- Platform Support
- Features
- Documentation
- Bug Fixes

#### b) `.all-contributorsrc`

Add a new contributor object to the `contributors` array:

```json
{
  "login": "username",
  "name": "Full Name",
  "avatar_url": "https://github.com/username.png",
  "profile": "https://github.com/username",
  "contributions": [
    "code"  // or "doc", "platform", "design", "infra", "bug", etc.
  ]
}
```

Common contribution types:
- `code`: Code contributions
- `doc`: Documentation
- `bug`: Bug reports
- `platform`: Platform/build support
- `design`: Design/UX
- `infra`: Infrastructure/DevOps

#### c) `README.md`

Add a new table cell in the "Contributors" section (usually near the bottom):

```html
<td align="center" valign="top" width="14.28%">
  <a href="https://github.com/username">
    <img src="https://github.com/username.png?s=100" width="100px;" alt="Full Name"/>
    <br />
    <sub><b>Full Name</b></sub>
  </a>
  <br />
  <a href="#code-username" title="Code">💻</a>  <!-- Adjust emoji based on contribution type -->
</td>
```

**Tip:** You can use the `all-contributors` CLI tool to automate this:

```bash
npx all-contributors add username code
```

### 4. Commit and Push Changes

Commit the version bump and contributor updates:

```bash
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
git add CONTRIBUTORS.md .all-contributorsrc README.md  # If updating contributors
git commit -m "chore: prepare release vX.Y.Z

- Bump version to X.Y.Z
- Add contributor recognition for [names/PRs if applicable]

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

Push the branch and create a PR:

```bash
git push -u origin release/vX.Y.Z-prep
gh pr create --title "chore: prepare release vX.Y.Z" --body "Preparation for vX.Y.Z release

## Changes
- Version bumped to X.Y.Z
- Contributor recognition updated (if applicable)

## Pre-merge checklist
- [ ] All version strings match across files
- [ ] Cargo.lock updated
- [ ] Contributors properly credited
- [ ] CI passes
"
```

### 5. Merge the Release PR

After approval, merge the PR to `main`:

```bash
gh pr merge --squash  # or merge via GitHub UI
```

### 6. Create and Push the Git Tag

The CI/CD workflow triggers on tags matching `v*`. The tag determines the release version, **not** the version strings in the config files.

```bash
git checkout main
git pull origin main
git tag vX.Y.Z
git push origin vX.Y.Z
```

**Critical:** Ensure the tag exactly matches `vX.Y.Z` format (e.g., `v0.2.0`, not `0.2.0` or `v0.2.0-beta`).

### 7. Monitor the CI/CD Build

After pushing the tag:

1. Go to the [Actions tab](https://github.com/minchenlee/c9watch/actions)
2. Find the "Release" workflow triggered by your tag
3. Wait for the build to complete (~10-15 minutes)
4. The workflow will:
   - Build macOS binaries for both `aarch64` (Apple Silicon) and `x86_64` (Intel)
   - Create DMG installers
   - Generate `.app.tar.gz` archives for the updater
   - Create an updater manifest (`latest.json`)
   - Create a **draft** GitHub release with all artifacts

### 8. Test the Draft Release

Before publishing, test the draft release:

1. Go to the [Releases page](https://github.com/minchenlee/c9watch/releases)
2. Find your draft release
3. Download the DMG for your architecture
4. Test installation and core functionality
5. Verify no regressions or critical issues

**Important:** Draft releases are **not** visible to the updater endpoint. Users won't receive update notifications for draft releases.

### 9. Publish the Release

Once testing is complete:

1. Edit the draft release on GitHub
2. Review/edit the auto-generated release notes
3. Add any additional context or breaking change warnings
4. **Uncheck "Set as a pre-release"** (if checked)
5. Click **"Publish release"**

Publishing the release:
- Makes it the latest release
- Updates the `/latest/download/latest.json` endpoint
- Triggers auto-update notifications for existing users

## Post-Release

After publishing:

1. **Verify the updater endpoint:**
   ```bash
   curl -s https://github.com/minchenlee/c9watch/releases/latest/download/latest.json | jq
   ```
   Confirm it shows the new version.

2. **Test auto-update:** Run an older version of c9watch and verify it detects and offers the update.

3. **Update Homebrew cask** (if applicable): The `.app.tar.gz` artifacts can be used for Homebrew distribution.

4. **Announce the release:** Consider announcing on relevant channels (project README, social media, etc.).

## Testing Pre-Release Builds

To test a release candidate without affecting production users:

### Option 1: Use Draft Releases (Recommended)

Draft releases don't appear at the `/latest/` endpoint, so they won't trigger auto-updates.

1. Push a tag (e.g., `v0.2.0-rc1`)
2. Let CI build the artifacts
3. Test the draft release
4. Delete the tag and draft if issues found
5. Fix issues and repeat

### Option 2: Use Pre-Release Tags

Tags with pre-release suffixes (e.g., `v0.2.0-beta1`, `v0.2.0-rc1`) can be marked as pre-releases in GitHub:

1. Push the tag
2. Let CI build
3. Edit the draft release and check "Set as a pre-release"
4. Publish as a pre-release

Pre-releases:
- Don't trigger auto-updates
- Don't appear at `/latest/`
- Are visible on the releases page for manual testing

## Troubleshooting

### Build Fails on CI

- Check the Actions logs for errors
- Common issues:
  - Rust compilation errors (check `Cargo.toml` dependencies)
  - Node/npm issues (check `package.json`)
  - Tauri configuration errors (check `tauri.conf.json`)
- Fix the issue, delete the tag, and re-tag after merging the fix

### Version Mismatch Warnings

If the version in config files doesn't match the tag:

- The release will use the **tag version** (not config versions)
- This creates inconsistency between the built app and the release
- Always ensure config versions match the tag before pushing

### Updater Not Working

If users aren't seeing the update:

- Verify the release is **published** (not draft, not pre-release)
- Check `/latest/download/latest.json` serves the correct version
- Ensure the `latest.json` manifest has correct signatures and URLs
- Verify the `pubkey` in `tauri.conf.json` matches the signing key used in CI

### Need to Hotfix a Release

If a critical bug is found after release:

1. Create a hotfix branch from the release tag
2. Fix the issue
3. Bump to the next patch version (e.g., `v0.2.0` → `v0.2.1`)
4. Follow the normal release process
5. The new patch will automatically replace the broken version as "latest"

## CI/CD Workflow Details

The release workflow (`.github/workflows/release.yml`):

- **Trigger:** Pushing a tag matching `v*`
- **Builds:** macOS binaries for `aarch64` and `x86_64`
- **Artifacts:**
  - DMG installers: `c9watch_vX.Y.Z_<arch>.dmg`
  - App bundles: `c9watch_vX.Y.Z_<arch>.app.tar.gz`
  - Updater manifest: `latest.json`
- **Signing:** Uses `TAURI_SIGNING_PRIVATE_KEY` secret for code signing
- **Notarization:** Uses Apple credentials if secrets are configured
- **Release:** Creates a draft GitHub release with all artifacts

The workflow **does not** automatically publish releases—this is intentional to allow testing before making the release public.

## Additional Resources

- [Tauri Updater Documentation](https://v2.tauri.app/plugin/updater/)
- [Semantic Versioning Spec](https://semver.org/)
- [All-Contributors Specification](https://allcontributors.org/)
- [Conventional Commits](https://www.conventionalcommits.org/)
