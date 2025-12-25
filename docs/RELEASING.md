# Release Handbook

## Version Format

GitTop follows [Semantic Versioning](https://semver.org/):

```
vMAJOR.MINOR.PATCH[-PRERELEASE]
```

| Component | When to increment |
|-----------|-------------------|
| **MAJOR** | Breaking changes (API, config format, behavior) |
| **MINOR** | New features, backward compatible |
| **PATCH** | Bug fixes only |

## Pre-release Tags

| Tag | Purpose | Downstream effect |
|-----|---------|-------------------|
| `v0.1.0-alpha.1` | Early testing, unstable | GitHub pre-release only |
| `v0.1.0-beta.1` | Feature complete, needs testing | GitHub pre-release only |
| `v0.1.0-rc.1` | Release candidate, final testing | GitHub pre-release only |
| `v0.1.0` | **Stable release** | Updates Scoop/Chocolatey/AUR |

Pre-releases create GitHub releases marked as "pre-release" but don't trigger package manager updates.

---

## How to Release

### 1. Update version in Cargo.toml
```toml
version = "0.1.0"
```

### 2. Commit the version bump
```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.1.0"
git push
```

### 3. Create and push tag
```bash
# For release candidate (test first!)
git tag v0.1.0-rc.1
git push origin v0.1.0-rc.1

# For stable release
git tag v0.1.0
git push origin v0.1.0
```

### 4. What happens automatically

1. **release.yml** triggers on the tag
2. Builds Windows + Linux binaries
3. Creates GitHub Release with:
   - `gittop-windows-x86_64.zip`
   - `gittop-linux-x86_64.tar.gz`
   - `SHA256SUMS.txt`

### 5. Update package managers (manual for now)

After release, update checksums in:
- `packaging/scoop/gittop.json`
- `packaging/chocolatey/tools/chocolateyInstall.ps1`
- `packaging/aur/PKGBUILD`

---

## Version Matching Rule

The tag version (without pre-release suffix) **must match** `Cargo.toml`:

| Cargo.toml | Valid tags |
|------------|------------|
| `0.1.0` | `v0.1.0`, `v0.1.0-rc.1`, `v0.1.0-beta.2` |
| `0.2.0` | `v0.2.0`, `v0.2.0-alpha.1` |

The release workflow **fails** if they don't match.

---

## Quick Reference

```bash
# Test the release pipeline
git tag v0.1.0-rc.1 && git push origin v0.1.0-rc.1

# Ship stable release
git tag v0.1.0 && git push origin v0.1.0

# Delete a bad tag (if needed)
git tag -d v0.1.0-rc.1
git push origin :refs/tags/v0.1.0-rc.1
```
