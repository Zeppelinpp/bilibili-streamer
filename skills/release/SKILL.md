---
description: Release a new version — bump version, commit, tag, push, and publish via gh
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

Ensure the new version is strictly greater than the current version. Reject downgrades or duplicates. If the user explicitly overrides (e.g. "remove previous releases, make this X"), proceed with their instruction.

## 4. Update all version files

Edit these three files to set the new version:
- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`

Then run `npm install --package-lock-only` to sync the version in `package-lock.json`.

## 5. Collect changes for release notes

Run `git log --pretty=format:"- %s" <last-tag>..HEAD` to get commits since the last version tag. If this is empty or ambiguous, fall back to `git log --oneline -20`.

Also run `git diff --stat HEAD~1` or review the latest commits to summarize the user-facing changes.

## 6. Stage and commit

Stage all the unstaged files:
```
git add .
```

Commit with a conventional commit message, Make the workspace git status clean:
```
chore(release): release v<version>

<brief summary of main changes>

<concise list of breaking changes>
```

## 7. Create and push tag

Create an annotated tag:
```
git tag -a v<version> -m "Release v<version>"
```

If the tag already exists, delete it locally first (`git tag -d v<version>`) and recreate it if the user requested a replacement.

Push both branch and tag:
```
git push origin <current-branch>
git push origin v<version>
```

If the remote tag already exists and you need to replace it, force-push the tag:
```
git push origin v<version> --force-with-lease
```

If branch push is rejected (non-fast-forward), merge remote first, then retry.

## 8. Generate and publish release notes

Draft release notes in markdown:

```markdown
## What's Changed

<summary of changes from git log>

**Full Changelog**: https://github.com/Zeppelinpp/bilibili-streamer/compare/v<previous-version>...v<version>
```

Publish the release using the GitHub CLI:
```
gh release create v<version> --title "Bilibili-Streamer v<version>" --notes-file - <<'EOF'
## What's Changed

<changes>

**Full Changelog**: https://github.com/Zeppelinpp/bilibili-streamer/compare/v<previous-version>...v<version>
EOF
```

If the release already exists on GitHub, update it instead:
```
gh release edit v<version> --title "Bilibili-Streamer v<version>" --notes-file - <<'EOF'
...
EOF
```

## 9. Confirm completion

Report:
- Version bumped from X to Y
- Commit hash
- Tag pushed
- GitHub release published / updated
- Remind the user that GitHub Actions will build the release artifacts automatically
