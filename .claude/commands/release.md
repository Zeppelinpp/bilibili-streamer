---
description: Release a new version — bump version, commit, tag, push, and generate release notes
---

Release a new version of Bilibili-Streamer. Follow this workflow exactly:

## 1. Determine the new version

If `$ARGUMENTS` is provided (e.g. `/release 3.0.4`), use that version. Otherwise, ask the user what version to release.

## 2. Check current versions

Read the current version from these files and report them:
- `package.json` — `version` field
- `src-tauri/tauri.conf.json` — `version` field
- `src-tauri/Cargo.toml` — `version` field

If they are not identical, warn the user and stop.

## 3. Validate version bump

Ensure the new version is strictly greater than the current version. Reject downgrades or duplicates.

## 4. Update all version files

Edit these three files to set the new version:
- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`

## 5. Collect changes for release notes

Run `git log --pretty=format:"- %s" <last-tag>..HEAD` to get commits since the last version tag. If this is empty or ambiguous, fall back to `git log --oneline -20`.

Also run `git diff --stat HEAD~1` or review the latest commits to summarize the user-facing changes.

## 6. Stage and commit

Stage the version-bump files:
```
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
```

Commit with a conventional commit message:
```
chore(release): bump version to <version>

<brief summary of main changes>
```

## 7. Create and push tag

Create an annotated tag:
```
git tag -a v<version> -m "Release v<version>"
```

Push both branch and tag:
```
git push origin <current-branch>
git push origin v<version>
```

If push is rejected (non-fast-forward), merge remote first, then retry.

## 8. Generate release notes

After push succeeds, output a markdown release notes block the user can paste into GitHub Release (if GitHub Actions doesn't auto-generate it):

```markdown
## What's Changed

<summary of changes from git log>

**Full Changelog**: https://github.com/Zeppelinpp/bilibili-streamer/compare/v<previous-version>...v<version>
```

## 9. Confirm completion

Report:
- Version bumped from X to Y
- Commit hash
- Tag pushed
- Remind the user that GitHub Actions will build the release artifacts automatically
