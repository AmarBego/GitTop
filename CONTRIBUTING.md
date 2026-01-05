# Contributing to GitTop

Thank you for your interest in contributing to GitTop! We welcome contributions from everyone.

## Core Resources

Before you dive in, please verify the architectural guidelines and branching strategies in our **[Developer Documentation](https://amarbego.github.io/GitTop/dev/)**.

## Environment Setup

### Prerequisites

*   **Rust**: Version 1.85+ (edition 2024)
*   **Git**: For version control
*   **Bacon**: Highly recommended for the development feedback loop.
    ```bash
    cargo install --locked bacon
    ```
*   **Prek**: Used for pre-commit checks.
    ```bash
    cargo binstall prek # Recommended
    # Or see https://github.com/j178/prek for other methods
    ```

**Linux Specifics:**
*   **Arch Linux**: `sudo pacman -S gcc-libs gtk3 libappindicator-gtk3`
*   **Desktop Integration**: Run `./scripts/install.sh` to install desktop icons and `.desktop` files.

### Installation

1.  **Clone and Enter**:
    ```bash
    git clone https://github.com/AmarBego/GitTop.git
    cd GitTop
    ```

## Development Workflow

We use `bacon` to manage development tasks. It runs in the background, monitoring your code and providing instant feedback on errors, warnings, and test results.

### Running the App

To start the app in development mode with live reloading:

```bash
bacon run
```

### Common Tasks

| Job | Command | Hotkey | Description |
| :--- | :--- | :--- | :--- |
| **Check** | `bacon` | `c` | Standard syntax and type checking (default). |
| **Run** | `bacon run` | `r` | Compiling and running the app. |
| **Test** | `bacon test` | `t` | Run unit tests. |
| **Clippy** | `bacon clippy`| `l` | Run linter checks for code style and strictness. |
| **Mock** | `bacon mock` | `m` | Run with 1000 mock notifications for UI testing. |

### Quality Assurance

Before submitting a Pull Request, please ensure all local checks pass. references our `prek` configuration:

```bash
prek
```
