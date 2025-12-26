# Platform Architecture

## Philosophy

GitTop uses platform-specific implementations where behavior differs. The `src/platform/` module abstracts these differences behind a unified API.

## Application Runner (`run_app`)

| Platform | Mode | Reason |
|----------|------|--------|
| **Windows** | `iced::application` | `window::Mode::Hidden` works correctly |
| **macOS** | `iced::application` | Hidden mode supported |
| **Linux/FreeBSD** | `iced::daemon` | Hidden mode broken on Wayland; daemon keeps process alive with zero windows |

### Linux Daemon Decision

Wayland compositors don't support hiding windows via `winit` (the window library iced uses). The workaround is:
1. Use `iced::daemon` so the process stays alive without windows
2. On "minimize to tray": `window::close()` destroys the window
3. On "show from tray": `window::open()` creates a new window

This is the officially recommended Wayland approach.

## Tray Icon

Built with `tray-icon` crate:
- **Windows**: Native Win32 system tray
- **Linux/FreeBSD**: GTK via AppIndicator/libayatana (requires `gtk::init()` before use)
- **macOS**: Native NSStatusItem

## Notifications

| Platform | Implementation |
|----------|----------------|
| Windows | WinRT Toast (`tauri-winrt-notification`) |
| Linux/FreeBSD | DBus (`notify-rust`) |
| macOS | NSUserNotificationCenter |

## Memory Management

- **Windows**: `EmptyWorkingSet()` trims working set
- **Linux (glibc)**: `malloc_trim()` releases memory to OS
- Called when minimizing to tray

## Dark Mode

- **Windows**: Undocumented `SetPreferredAppMode()` from uxtheme.dll
- **Linux/macOS**: Follows system GTK/Qt theme automatically
