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

<p align="left">
  <a href="https://amarbego.github.io/GitTop/">
    <img src="https://img.shields.io/badge/Read_the_Docs-FF5A47?style=for-the-badge&logo=googledocs&logoColor=white" alt="Read the Docs">
  </a>
</p>

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

**Installer (recommended):**
- [Download EXE installer](https://github.com/AmarBego/GitTop/releases/latest) Wizard-based setup with optional startup integration

**Winget**
```pwsh
winget install AmarBego.GitTop
```

**Chocolatey:**
```pwsh
choco install gittop
```

**Scoop:**
```pwsh
scoop bucket add gittop https://github.com/AmarBego/GitTop
scoop install gittop
```
> *Once GitTop is added to the [Scoop Extras](https://github.com/ScoopInstaller/Extras) bucket, you'll be able to install directly with `scoop install gittop`.*

**Manual:** Download `gittop-windows-x86_64.zip` from releases, extract, run `gittop.exe`.

### Linux

**Flatpak:**
```bash
# From Flathub (when published)
# flatpak install flathub io.github.AmarBego.GitTop

# Or install from bundled .flatpak file
flatpak install gittop-VERSION.flatpak
flatpak run io.github.AmarBego.GitTop
```

**Arch Linux (AUR):**
```bash
# If using yay
yay -S gittop-bin

# If using paru
paru -S gittop-bin
```

**Fedora (COPR):**
```bash
sudo dnf copr enable amarbego/gittop
sudo dnf install gittop
```

**Ubuntu (PPA):**
```bash
sudo add-apt-repository ppa:amarbego/gittop
sudo apt update
sudo apt install gittop
```

**Manual:** Download `gittop-linux-x86_64.tar.gz` from releases:
```bash
tar xzf gittop-linux-x86_64.tar.gz
./gittop-linux-x86_64/gittop
```

## Building from Source

For those who prefer to compile from source or are interested in contributing, please check out our [Contribution Guide](CONTRIBUTING.md).

It covers everything from:
- Building and running locally with `bacon`
- Platform-specific dependencies
- Access to our full [Developer Documentation](https://amarbego.github.io/GitTop/dev/)

## License

AGPL-3.0-only. See [LICENSE.md](LICENSE.md).
