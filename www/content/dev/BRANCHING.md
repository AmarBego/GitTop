+++
title = "Branching Strategy"
description = "How we manage branches and code flow in GitTop"
weight = 2
+++

This is how we manage code in GitTop. We use a **Release Branch** model, which is common for desktop software that needs support for older versions.

## The Gist

*   Develop on `main`.
*   Cut a `release/X.Y` branch when it's time to ship.
*   Fix bugs on `release/X.Y` and backport to `main`.
*   Feature branches (`feature/*`) for everything effectively.

## Our Branches

| Branch | What's in it? |
|--------|---------------|
| `main` | The bleeding edge. This is effectively the "next" version of the app. |
| `release/X.Y` | Stable lines. Receives **bugfixes only** (vX.Y.Z). New features go to X.Y+1. |
| `feature/*` | New stuff. Merge into `main` via PR. |
| `fix/*` | Bug fixes for future releases. Merge into `main`. |
| `hotfix/*` | Emergency fixes for *shipped* releases. Merge into `release/X.Y`. |

## How it Works

### 1. Normal Development

We act like most projects here. You branch off `main`, do your work, and PR it back into `main`.

```
main (e.g. 0.3.0-dev)
├── feature/cool-new-ui
└── fix/typo
```

### 2. Pre-releases (Alphas/Betas)

We enable testing by tagging directly on `main`.

```bash
# Tag it, push it, let CI handle the rest
git tag v0.3.0-alpha.1
git push origin v0.3.0-alpha.1
```


*   **GitHub Releases**: Users can download the `alpha`/`beta` installers.
*   **Package Managers**: They ignore these (by design).

### 3. Going Stable

When `main` is polished and ready, we "cut" the release branch.

```bash
# Create the release line
git checkout -b release/0.3 main

# Tag the stable version
git tag v0.3.0
git push origin release/0.3 --tags
```

`release/X.Y` branches allow us to ship fixes without destabilizing ongoing development on `main`.

This triggers the "Big Release":
*   Stable binaries are published.
*   Scoop, Chocolatey, and AUR get updated automatically.

### 4. Handling Hotfixes

Oops, verifying a bug in `v0.3.0`. Here is how we fix it without waiting for `v0.4.0`.

1.  **Branch off the release**:
    ```bash
    git checkout release/0.3
    git checkout -b hotfix/crash-fix
    ```

2.  **Fix and Release**:
    Merge it back into `release/0.3` and tag the patch version.
    ```bash
    git tag v0.3.1
    git push origin release/0.3 --tags
    ```

3.  **Don't forget Main**:
    If the bug exists in `main` (it usually does), cherry-pick the fix.
    ```bash
    git checkout main
    git cherry-pick <commit-hash>
    ```
