# Branch and Merge Strategy

This document describes c9watch's branching model and merge strategy.

## Branch Strategy

### Main Branch
- **`main`** - Production-ready code
- Protected branch (all changes via PRs)
- Always deployable
- Never commit directly to main

### Feature Branches
- **Naming**: `feature/description`, `fix/description`, `docs/description`
- Short-lived (delete after merge)
- Created from and merged back into `main`

**Examples:**
```bash
feature/linux-support
fix/session-detection-bug
docs/update-readme
chore/update-dependencies
```

## Merge Strategy

### Squash and Merge (Enforced)
- **All PRs are squashed** into a single commit when merged
- This keeps the main branch history clean and linear
- Each commit in main represents one complete feature/fix

**Why squash?**
- ✅ Clean, readable history
- ✅ Easy to revert entire features
- ✅ No merge commit clutter
- ✅ Each commit is a complete, logical unit

### Commit Message Format
When squashing, the PR title becomes the commit message. Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add Linux support
fix: resolve session detection race condition
docs: update installation instructions
chore: bump dependencies
```

## Workflow Example

```bash
# 1. Create feature branch
git checkout -b feature/awesome-feature

# 2. Make changes and commit
git add .
git commit -m "feat: implement awesome feature"

# 3. Push and create PR
git push -u origin feature/awesome-feature
gh pr create --title "feat: add awesome feature" --body "..."

# 4. After review, maintainer squash-merges via GitHub
# (GitHub automatically deletes the remote branch)

# 5. Clean up local branch
git checkout main
git pull
git branch -d feature/awesome-feature
```

## Branch Protection

The `main` branch is protected with:
- ✅ Requires pull request before merging
- ✅ Requires status checks to pass (if CI configured)
- ✅ Enforced for administrators
- ✅ Stale PR approvals dismissed on new commits

## Repository Settings

**Merge options** (Settings → Pull Requests):
- ☐ Allow merge commits
- ☑️ Allow squash merging (ONLY this is enabled)
- ☐ Allow rebase merging
- ☑️ Automatically delete head branches

---

For more details, see [CONTRIBUTING.md](../CONTRIBUTING.md).
