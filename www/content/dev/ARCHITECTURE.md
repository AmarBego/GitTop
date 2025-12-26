+++
title = "Architecture & Codebase"
description = "High-level codebase structure and platform-specific implementation details"
weight = 1
+++

This document explains both the high-level codebase structure and the low-level platform specifics.

## Codebase Map

A high-level overview of where things live.

| Directory | Core Purpose |
|-----------|--------------|
| `src/ui` | **Presentation**. The Iced frontend. |
| `src/github` | **Integration**. API, Auth, and Keyring. |
| `src/platform` | **Abstraction**. OS-level glue code. |
| `src/cache` | **Persistence**. Disk caching logic. |
| `src/specs` | **Mocking**. Test data generators. |
| `src/settings.rs` | **Configuration**. Global app structs. |
| `src/tray.rs` | **Orchestration**. System tray logic. |

### UI Architecture (`src/ui`)

The UI is built with [Iced](https://github.com/iced-rs/iced). Everything here is about rendering frames and handling user messages.

*   **`app.rs`**: The main entry point and global state manager.
*   **`icons.rs`**: Icon primitives and glyph helpers.
*   **`theme.rs`**: Color palette and styling constants.
*   **`window_state.rs`**: Window visibility/focus state tracking.
*   **`screens/`**: User-facing views.
    *   **`login/`**: OAuth login flow.
    *   **`notifications/`**: The Inbox. This is where users spend 90% of their time.
        *   `engine.rs`: **The Executor**. Applies rules to incoming notifications.
        *   `helper.rs`: Utility functions for notification operations.
        *   `messages.rs`: All message types for the notifications screen.
        *   `screen.rs`: Main screen state and update logic.
        *   `view/`: Rendering modules.
            *   `sidebar`/`sidebar_state`: Navigation and filters.
            *   `content`: The main notification list.
            *   `header`: Content header with title, sync status, filters.
            *   `bulk`: Bulk action bar (Power Mode).
            *   `group`: Collapsible headers.
            *   `states`: Loading, error, and empty state views.
    *   **`settings/`**: Configuration screens.
        *   `tabs/`: `accounts`, `general`, `power_mode`.
        *   `rule_engine/`: **The Rule Editor**. Logic for the visual Rule Editor.
            *   `rules.rs`: Core rule types and evaluation engine.
            *   `tabs/`: `overview`, `type_rules`, `account_rules`, `org`.
            *   `inspector.rs`: Tool that explains *why* a notification was treated a certain way.
            *   `explain_decision.rs`: Test lab for simulating rule outcomes.
            *   `components.rs`: Shared UI components (rule cards, empty states).
*   **`widgets/`**: Reusable components.
    *   `notification_item.rs`: The card view for a single notification.
    *   **`power/`**: "Power Mode" widgets.
        *   `top_bar.rs`: Command center header bar.
        *   `details_panel.rs`: Notification detail view.
        *   `status_bar.rs`: Bottom status indicators.

#### How We Write UI Code

Every screen follows the same shape. Once you've seen one, you've seen them all.

**The core idea: Screens own state. Views just render it.**

A "screen" is a top-level page (Notifications, Settings, Login). Each screen lives in its own folder under `screens/` and has a predictable structure:

| File | What It Does |
|------|--------------|
| `screen.rs` | Owns the state, handles messages, orchestrates the layout |
| `messages.rs` | Defines all the things a user can do on this screen |
| `view/` | Pure rendering functions no state mutation, just turning data into pixels |

**The `screen.rs` file always has three things:**

1.  A struct holding all the screen's state
2.  An `update()` method that handles incoming messages
3.  A `view()` method that renders the UI

The `view()` method doesn't do much itself it calls into smaller view helpers to build the layout. Think of it as the conductor.

**Two ways to write view code:**

When the view needs access to everything on the screen, we use an `impl` block that extends the screen struct:

```rust
// view/content.rs
impl NotificationsScreen {
    pub fn view_content(&self) -> Element<'_, NotificationMessage> {
        // Can access all of self.* here
    }
}
```

When the view only needs a few pieces of data, we use a free function with a small state struct:

```rust
// view/sidebar.rs
pub fn view_sidebar(state: SidebarState) -> Element<'_, NotificationMessage> {
    // Only sees what's in SidebarState
}
```

The second approach makes dependencies explicit and keeps views focused.

**Messages are grouped, not flat:**

Instead of one giant enum with 50 variants, we nest related messages:

```rust
enum NotificationMessage {
    Filter(FilterMessage),   // Type/repo selection
    Thread(ThreadMessage),   // Open, mark read, mark done
    Bulk(BulkMessage),       // Multi-select actions
    View(ViewMessage),       // Scroll, expand, select
    Navigation(NavigationMessage), // Logout, settings, switch account
}
```

The `update()` function matches on the outer enum and delegates to private handlers. This keeps each handler small and focused.

**Tabs get their own folder:**

Screens with tabs (like Settings or Rule Engine) put each tab in a `tabs/` subdirectory. Each tab is a free function returning an `Element`. The main `screen.rs` calls the right one based on which tab is selected.

### Backend Architecture (`src/github`)

Handles all interaction with the GitHub API.

*   `client.rs`: The HTTP client wrapping REST/GraphQL calls.
*   `auth.rs`: OAuth flow and token validation.
*   `keyring.rs`: Secure storage for API tokens (using OS keychain).
*   `session.rs`: Manages active accounts and switch state.

### Data Layer (`src/cache` & `src/specs`)

*   **`cache/`** (WIP):
    *   `disk`: Persistent storage using `sled`.
*   **`specs/`**:
    *   `mock_notifications`: Generates fake data for testing.

## Platform Specifics (`src/platform`)

GitTop tries to feel native everywhere. To do that, we sometimes have to do things differently on each OS. All that messy logic is hidden here.

### How the App Runs (`run_app`)

The core application loop changes depending on where you are.

| OS | Runner Mode | Why? |
|----|-------------|------|
| **Windows / macOS** | `iced::application` | Standard desktop app. We can just hide the window when minimizing to tray. |
| **Linux** | `iced::daemon` | Wayland makes "hiding" windows complicated. |

### The Linux Wayland Situation

On Linux (especially Wayland), you can't reliably "hide" a window. If you try, it might just minimize or stay visible.

So, on Linux we use `iced::daemon`. This lets the process run without *any* window open.
*   **Minimize to Tray**: We actually `close()` (destroy) the window.
*   **Open from Tray**: We `open()` (create) a fresh window.

It sounds drastic, but it's the correct way to handle tray-only apps on modern Linux compositors.

### Desktop Integration

#### System Tray

We use the `tray-icon` crate to sit in your status bar.

*   **Windows**: Standard Win32 tray icon.
*   **Linux**: Uses `libayatana` or AppIndicator. We have to initialize GTK first.
*   **macOS**: Native status item.

#### Notifications

We use the native notification systems so GitTop feels integrated.

*   **Windows**: WinRT Toasts (via `tauri-winrt-notification`).
*   **Linux**: DBus notifications (via `notify-rust`).
*   **macOS**: Native Notification Center.

#### Memory Optimization

We are aggressive about memory usage. When you minimize the app to the tray, we ask the OS to reclaim unused memory.

*   **Windows**: Calls `EmptyWorkingSet()` to trim the working set.
*   **Linux (glibc)**: Calls `malloc_trim()` to release heap memory back to the OS.

#### Dark Mode

*   **Windows**: We call the undocumented `SetPreferredAppMode` API in `uxtheme.dll` to force the window frame to match your system theme.
*   **Linux / macOS**: Usually "Just Works" by following the system GTK/Qt theme.
