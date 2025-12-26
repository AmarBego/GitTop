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
2. Builds Windows + Linux binaries and installers
3. Creates GitHub Release with:
   - `gittop-windows-x86_64.zip` — Portable archive
   - `gittop-X.Y.Z-setup.exe` — EXE installer (Inno Setup)
   - `gittop-linux-x86_64.tar.gz` — Linux archive
   - `SHA256SUMS.txt`
4. Downstream workflows update package managers (stable releases only)

### 5. Package manager updates (automated)

For **stable releases only**, downstream workflows automatically update:
- **Scoop** — Updates `bucket/gittop.json` with new version/checksum
- **Chocolatey** — Builds and pushes `.nupkg` to Chocolatey.org
- **AUR** — Updates `PKGBUILD` and pushes to AUR

Prereleases (`-alpha`, `-beta`, `-rc`) skip package manager updates.

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

---

## OBSOLETE FOR NOW

> The following sections document MSI installer functionality that is currently disabled pending code signing setup.

### Tagging Limits (MSI-specific)

Due to MSI version constraints, there are practical limits on prerelease counts:

| Limit | Max Value | If exceeded... |
|-------|-----------|----------------|
| Patches per minor | 6 | Bump the minor version |
| Prereleases per stage | 999 | Bump the patch or stage |

**In practice:** If you hit `alpha.999` or `0.1.7`, bump the minor version. These limits far exceed normal usage.

### MSI Version Mapping

Windows Installer (MSI) requires numeric `Major.Minor.Build.Revision` format. SemVer is mapped using:

**Formula:** `Build = (patch × 10000) + base_offset + N`

| Stage | Base | Example SemVer | MSI ProductVersion |
|-------|------|----------------|---------------------|
| alpha | 1000 | `0.1.0-alpha.5` | `0.1.1005.0` |
| stable | 4000 | `0.1.0` | `0.1.4000.0` |
| stable | 4000 | `0.1.1` | `0.1.14000.0` |

This ensures upgrades work correctly: **0.1.0 → 0.1.1-alpha.1 → 0.1.1**.

> Max 6 patches per minor (build > 65535 fails). Filename uses full SemVer for humans.
