# Release Pipeline Architecture

This document explains how GitTop's automated release and distribution pipeline works.

## Overview

```
1. Developer pushes tag: v0.2.0
   │
2. release.yml extracts version: 0.2.0
   │
3. Builds create:
   │  • gittop-windows-x86_64.zip
   │  • gittop-X.Y.Z-setup.exe (Inno Setup)
   │  • gittop-linux-x86_64.tar.gz
   │
4. GitHub Release created with prerelease=false
   │
5. release-meta artifact saved:
   │  • tag: v0.2.0
   │  • is_prerelease: false
   │
6. Downstream workflows trigger:
   │
   ├─▶ aur.yml
   │   • Downloads new tarball
   │   • Updates pkgver=0.2.0
   │   • Recalculates sha256sums
   │   • Pushes to AUR
   │
   ├─▶ chocolatey.yml
   │   • Downloads EXE installer
   │   • Calculates checksum
   │   • Updates nuspec + install script
   │   • Pushes gittop.0.2.0.nupkg
   │
   └─▶ scoop.yml
       • Updates bucket/gittop.json
       • Commits to repository
```

## Workflows

### 1. CI (`ci.yml`)

**Trigger:** Push to `main`, PRs to `main`

**Purpose:** Validate that the code builds on Windows.

| Job | Description |
|-----|-------------|
| `build-windows` | Builds release binary on Windows |

---

### 2. Release (`release.yml`)

**Trigger:** Push tag matching `v*.*.*` (including `-rc`, `-alpha`, `-beta` suffixes)

**Purpose:** Build release artifacts and create GitHub Release.

| Job | Description |
|-----|-------------|
| `build-windows` | Builds Windows `.zip` and `.exe` installer |
| `build-linux` | Builds Linux `.tar.gz` with desktop integration files |
| `release` | Creates GitHub Release, uploads artifacts, saves metadata for downstream |

**Windows Installer:**
- **EXE** — Built with Inno Setup (silent install: `/VERYSILENT /SUPPRESSMSGBOXES`)

**Key Output:** `release-meta` artifact containing:
- `tag` — The release tag (e.g., `v0.2.0`)
- `is_prerelease` — Boolean flag (`true` or `false`)

This artifact is consumed by downstream workflows to ensure they operate on the **exact** release that triggered them.

---

### 3. AUR Distribution (`aur.yml`)

**Trigger:** `workflow_run` after `Release` completes successfully

**Purpose:** Publish stable releases to [Arch User Repository](https://aur.archlinux.org/packages/gittop-bin).

| Step | Description |
|------|-------------|
| Download metadata | Gets `release-meta` artifact from triggering workflow |
| Skip prereleases | Exits early if `is_prerelease == true` |
| Publish | Updates PKGBUILD version/checksums, pushes to AUR |

---

### 4. Chocolatey Distribution (`chocolatey.yml`)

**Trigger:** `workflow_run` after `Release` completes successfully

**Purpose:** Publish stable releases to [Chocolatey Community Repository](https://community.chocolatey.org/packages/gittop).

| Step | Description |
|------|-------------|
| Download metadata | Gets `release-meta` artifact from triggering workflow |
| Skip prereleases | Exits early if `is_prerelease == true` |
| Download EXE installer | Fetches from GitHub Release |
| Calculate checksum | SHA256 of the EXE file |
| Update package files | Replaces `{{VERSION}}` and `{{CHECKSUM}}` placeholders |
| Pack & Push | Runs `choco pack` and `choco push` |

---

### 5. Scoop Distribution (`scoop.yml`)

**Trigger:** `workflow_run` after `Release` completes successfully

**Purpose:** Publish stable releases to the [self-hosted Scoop bucket](https://github.com/AmarBego/GitTop).

| Step | Description |
|------|-------------|
| Download metadata | Gets `release-meta` artifact from triggering workflow |
| Skip prereleases | Exits early if `is_prerelease == true` |
| Update manifest | Updates `bucket/gittop.json` with version/checksums |
| Commit & push | Pushes changes to repository |

---

## Packaging Files

### AUR (`packaging/aur/stable-bin/`)

| File | Purpose |
|------|---------|
| `PKGBUILD` | Build instructions for Arch Linux |
| `gittop.install` | Post-install hooks (icon cache, desktop database) |

The PKGBUILD uses `${pkgver}` variable in the source URL, so version updates are automatic.

### Chocolatey (`packaging/chocolatey/`)

| File | Purpose |
|------|---------|
| `gittop.nuspec` | Package metadata (uses `{{VERSION}}` placeholder) |
| `tools/chocolateyInstall.ps1` | Install script (uses `{{VERSION}}`, `{{CHECKSUM}}`) |
| `tools/chocolateyUninstall.ps1` | Uninstall script |

### Scoop (`bucket/`)

| File | Purpose |
|------|---------|
| `gittop.json` | Scoop manifest (auto-updated by `scoop.yml`) |

> Self-hosted bucket at `https://github.com/AmarBego/GitTop`. Future migration to Scoop Extras planned.

### Inno Setup EXE Installer (`packaging/innosetup/`)

| File | Purpose |
|------|---------|
| `gittop.iss` | Inno Setup script |

---

## Adding New Package Managers

To add a new distribution target:

1. Create `packaging/<manager>/` with required files
2. Create `.github/workflows/<manager>.yml` with:
   ```yaml
   on:
     workflow_run:
       workflows: ["Release"]
       types: [completed]
   ```
3. Download `release-meta` artifact using `run-id: ${{ github.event.workflow_run.id }}`
4. Skip prereleases based on `is_prerelease` value
5. Add required secrets to GitHub repository settings

---

## OBSOLETE FOR NOW

> The following sections document MSI installer functionality that is currently disabled pending code signing setup.

### WiX MSI Installer (`packaging/wix/`)

| File | Purpose |
|------|---------|
| `main.wxs` | WiX configuration (uses `{{VERSION}}` placeholder) |
| `License.rtf` | AGPL-3.0 license dialog |

### MSI Version Mapping

MSI requires numeric `Major.Minor.Build.Revision` format (each 0–65535). SemVer is mapped to ensure correct upgrade ordering across patches and prereleases:

**Formula:** `Build = (patch × 10000) + base_offset + N`

| Stage | Base Offset | Example SemVer | Build Calculation | MSI Version |
|-------|-------------|----------------|-------------------|-------------|
| alpha | 1000 | `0.1.0-alpha.5` | 0×10000 + 1000 + 5 | `0.1.1005.0` |
| beta | 2000 | `0.1.0-beta.3` | 0×10000 + 2000 + 3 | `0.1.2003.0` |
| rc | 3000 | `0.1.0-rc.1` | 0×10000 + 3000 + 1 | `0.1.3001.0` |
| stable | 4000 | `0.1.0` | 0×10000 + 4000 + 0 | `0.1.4000.0` |
| stable | 4000 | `0.1.1` | 1×10000 + 4000 + 0 | `0.1.14000.0` |
| alpha | 1000 | `0.1.2-alpha.1` | 2×10000 + 1000 + 1 | `0.1.21001.0` |

### MSI Sanity Rules

1. **Only Build is used for upgrade ordering** — Revision is always `0`
2. **Build numbers must increase monotonically** — Never decrement
3. **Patch contributes 10000 per increment** — Ensures patch releases upgrade correctly
4. **Max 6 patches per minor** — Build > 65535 fails the pipeline
5. **ARPDisplayVersion uses full SemVer** — Users see `0.1.0-alpha.10`, not `0.1.1010.0`

This ensures: **0.1.0 < 0.1.1-alpha.1 < 0.1.1** — always.
