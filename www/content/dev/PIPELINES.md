+++
title = "Release Architecture"
description = "CI/CD pipelines and distribution workflows for GitTop releases"
weight = 3
+++

This document maps out how GitTop gets from a git tag to a published release.

## The Big Picture

Automated pipelines handle everything from building the binary to updating package managers.

1.  **You push a tag** (e.g., `v0.2.0`).
2.  **GitHub Actions builds the app** for Windows and Linux.
3.  **A Release is created** on GitHub with all the artifacts (installers, archives).
4.  **Distribution bots wake up** (for stable releases) and push the update to AUR, Chocolatey, etc.

## The Workflows

We use a "fan-out" architecture. The main release workflow does the heavy lifting, and then triggers downstream workflows for specific distribution channels.

### 1. The Builder: `release.yml`

This is the core workflow. It listens for any tag starting with `v*` (like `v0.1.0` or `v0.1.0-rc.1`).

**What it does:**
*   **Compiles** the code for Windows and Linux.
*   **Packages** the installers:
    *   `gittop-X.Y.Z-setup.exe` (Inno Setup)
    *   `gittop-windows-x86_64.zip` (Portable)
    *   `gittop-linux-x86_64.tar.gz`
*   **Creates the GitHub Release**: Uploads all artifacts and checksums.
*   **Exports Metadata**: Saves a `release-meta` artifact containing the version tag and whether it's a pre-release.

### 2. The Distributors

These workflows run *only* after `release.yml` completes successfully. They download the `release-meta` artifact to know exactly what version to publish.

> **Important**: All distributor workflows skip **pre-releases**. They only run for stable versions.

#### `aur.yml` (Arch Linux)
*   Downloads the new Linux tarball.
*   Updates the `PKGBUILD` with the new version and checksums.
*   Pushes to the [AUR](https://aur.archlinux.org/packages/gittop-bin).

#### `chocolatey.yml` (Windows)
*   Downloads the new `.exe` installer.
*   Calculates the SHA256 checksum.
*   Updates the Chocolatey package files (`.nuspec`, `install.ps1`).
*   Packs and pushes to [Chocolatey Community Repository](https://community.chocolatey.org/packages/gittop).

#### `scoop.yml` (Windows)
*   Updates `bucket/gittop.json` in our repository.
*   Commits the changes, making the update instantly available to Scoop users.

## Packaging Files

Where the magic files live:

*   **AUR**: `packaging/aur/stable-bin/` (`PKGBUILD`, `gittop.install`)
*   **Chocolatey**: `packaging/chocolatey/` (`gittop.nuspec`, scripts)
*   **Scoop**: `bucket/gittop.json`
*   **Inno Setup**: `packaging/innosetup/gittop.iss` (The Windows installer script)

## Adding a New Distributor

Want to add Homebrew or Snap?

1.  **Create the packaging files** in `packaging/<manager>/`.
2.  **Add a Workflow** in `.github/workflows/<manager>.yml`.
3.  **Listen for the signal**:
    ```yaml
    on:
      workflow_run:
        workflows: ["Release"]
        types: [completed]
    ```
4.  **Fetch Metadata**: usage the `release-meta` artifact to get the correct version tag.
5.  **Skip Pre-releases**: Check `is_prerelease` before publishing.

## Inactive: MSI Installer

> We are currently using Inno Setup (EXE) instead of MSI because we don't have code signing yet. The following details are preserved for future use.

### WiX MSI Installer (`packaging/wix/`)

*   `main.wxs`: The WiX definition.
*   `License.rtf`: The license shown during install.

### MSI Versioning Rules

MSI requires specific numeric versions (`Major.Minor.Build.Revision`). To support SemVer strings like `alpha` or `rc`, we map them to the `Build` number using a formula.

**Formula**: `Build = (Patch * 10000) + StageOffset + N`

| Stage | Offset | Example | MSI Version |
|-|--||-|
| alpha | 1000 | `0.1.0-alpha.5` | `0.1.1005.0` |
| stable | 4000 | `0.1.0` | `0.1.4000.0` |

This ensures that Windows correctly identifies `alpha.2` as an upgrade to `alpha.1`, but older than the final release.
