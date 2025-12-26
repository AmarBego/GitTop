+++
title = "Release Process"
description = "How to version and release GitTop"
weight = 4
+++

This guide covers how we version and release GitTop.

## Versioning Strategy

We follow a standard `vMAJOR.MINOR.PATCH` format with optional pre-release suffixes.

| Component | Description |
|-----------|-------------|
| **MAJOR** | Major milestones that redefine the maturity or direction of the product. |
| **MINOR** | New features, improvements, and functionality updates. |
| **PATCH** | Critical bug fixes and security patches only. No new features. |

This versioning scheme prioritizes clarity for users over strict semantic versioning rules.

### Pre-releases

We use suffixes to mark pre-release builds.

- **`alpha`**: Internal builds. Things might be broken.
- **`beta`**: Public testing builds. Feature complete but needs polish.
- **`rc`**: Release Candidate. We think this is ready for stable unless we find a critical bug.

**Examples:**
- `1.3.0` (Stable)
- `1.3.1-alpha.1` (Internal test)
- `1.3.1-rc.1` (Release candidate)

> Pre-releases may exist for any version (including patch releases) to validate cross-platform builds and installers. Cargo.toml does NOT need to be updated for pre-releases, only the base version must match.

## How to Release

Follow these steps to ship a new version.

### 1. Bump the Version

Update the version number in your `Cargo.toml`:

```toml
version = "0.1.0"
```

Commit this change:

```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.1.0"
git push
```

> Note: The version in Cargo.toml must match the base version of the tag (e.g. 0.1.0 > v0.1.0 OR v0.1.0-rc.1). If they don’t match, the release workflow will fail.

### 2. Create the Tag

Create and push a git tag for the version. The tag triggers the release pipeline.

For a release candidate:
```bash
git tag v0.1.0-rc.1
git push origin v0.1.0-rc.1
```

For a stable release:
```bash
git tag v0.1.0
git push origin v0.1.0
```

### 3. CI/CD Process

Once the tag is pushed, the `release.yml` workflow kicks in:

1.  **Builds** binaries and installers for Windows and Linux.
2.  **Creates a GitHub Release** with the artifacts:
    - `gittop-windows-x86_64.zip`
    - `gittop-X.Y.Z-setup.exe`
    - `gittop-linux-x86_64.tar.gz`
    - `SHA256SUMS.txt`
3.  **Updates Package Managers** (Stable releases only):
    - **Scoop**: Updates the manifest in our bucket.
    - **Chocolatey**: Pushes the new package.
    - **AUR**: Updates the PKGBUILD.

> **Note**: Pre-releases do not trigger package manager updates.

## Important Rules

**Tag Matching**: The git tag must match the version in `Cargo.toml`.
- `Cargo.toml`: `0.1.0` -> Tag: `v0.1.0` or `v0.1.0-rc.1`
- If they don't match, the release workflow will fail.

## Cheat Sheet

Common commands for releasing:

```bash
# Release a candidate
git tag v0.1.0-rc.1 && git push origin v0.1.0-rc.1

# Release stable
git tag v0.1.0 && git push origin v0.1.0

# Delete a mistake
git tag -d v0.1.0-rc.1
git push origin :refs/tags/v0.1.0-rc.1
```

## Inactive: MSI Installer Details

> These features are currently disabled. We need code signing before we can fully support MSI installers.

### MSI Version Limits

MSI has strict versioning constraints that limit how many updates we can release per minor version:

| Constraint | Limit | Solution |
|------------|-------|----------|
| Patches per Minor | ~6 | Bump the Minor version |
| Pre-release Builds | 999 | Bump the Patch or Stage |

Practically, this means if you hit `0.1.7` or `alpha.999`, it's time to bump the minor version anyway.

### How Versions Map to MSI

Windows Installer only understands `Major.Minor.Build`. To support SemVer (like `alpha` or `rc`), we map the suffixes to the Build number.

**Formula**: `Build = (Patch * 10000) + StageOffset + N`

| Stage | Offset | Example | MSI Version |
|-------|--------|---------|-------------|
| alpha | 1000 | `0.1.0-alpha.5` | `0.1.1005.0` |
| stable | 4000 | `0.1.0` | `0.1.4000.0` |
| stable | 4000 | `0.1.1` | `0.1.14000.0` |

This logic ensures that Windows sees `alpha.1` as "newer" than the base version, but "older" than `beta` or stable, allowing upgrades to work correctly.
