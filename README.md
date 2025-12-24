<p align="center">
  <img
    width="400"
    src="assets/images/text.png"
    alt="GitTop"
  />
</p>

<h1></h1>

<img
  src="assets/images/GitTop-256x256.png"
  alt="GitTop Logo"
  width="30%"
  align="right"
/>

**A lightweight desktop client for GitHub notifications. Why spin up a browser just to check your GitHub notifications?**

- **Super lean:** ~5-15MB RAM whilst in use (1-2MB in tray)
- **Multi-account:** Seamless support for multiple GitHub accounts
- **Smart Rules:** Powerful engine for priorities and hiding noisy notification types
- **Cross platform:** Native experience on Windows and Linux
- **Dual Mode:** Minimalist by default. Enable **Power Mode** for in-app notification viewing, rule engine and more
- **Stay focused:** Be on top of your notifications

<img
  src="assets/images/showcase.png"
  alt="GitTop Logo"
  width="100%"
  align="center"
/>
<a name="🚀-installation"></a>
## Installation

**[Download pre-built binaries from GitHub Releases](https://github.com/AmarBego/GitTop/releases)**

### Windows

**Scoop:**
```pwsh
Coming soon
```

**Chocolatey:**
```pwsh
choco install gittop
```

**Manual:** Download `gittop-windows-x86_64.zip` from releases, extract, run `gittop.exe`.

### Linux

**Arch Linux (AUR):**
```bash
yay -S gittop-bin
```

**Manual:** Download `gittop-linux-x86_64.tar.gz` from releases:
```bash
tar xzf gittop-linux-x86_64.tar.gz
./gittop-linux-x86_64/gittop
```

## Building from source

Requirements:
- Rust 1.85+ (edition 2024)
- Platform-specific dependencies (see below)

```bash
git clone https://github.com/AmarBego/GitTop.git
cd GitTop
cargo build --release

# Linux: Install desktop integration (icons, .desktop file)
./scripts/install.sh
```

Binary will be at `target/release/gittop` (Linux) or `target\release\gittop.exe` (Windows).

### Platform dependencies

**Linux:**
```bash
# Debian/Ubuntu
sudo apt install pkg-config libglib2.0-dev libgtk-3-dev libxdo-dev

# Fedora
sudo dnf install pkg-config glib2-devel gtk3-devel

# Arch
sudo pacman -S pkg-config glib2 gtk3

# If building using clang (mold) instead of gcc additionally install
sudo apt install clang xdotool
sudo dnf install clang xdotool
sudo pacman -S clang xdotool

```

**Windows:** No additional dependencies.

## Development

```bash
# Run in development mode
cargo run

# Run with bacon (recommended for dev)
bacon run

# Format + lint + test (pre-commit)
prek run
```

See [RELEASING.md](RELEASING.md) for version tagging and release process.

## License

AGPL-3.0-only. See [LICENSE.md](LICENSE.md).
