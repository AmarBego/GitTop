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

*   **`app.rs`**: The main entry point and global state orchestrator.
*   **`context.rs`**: Shared read-only state (Settings, SessionManager) passed to all screens.
*   **`effects.rs`**: The "Effect Pattern" definitions. Screens return `AppEffect` (Navigation, Session) instead of mutating state.
*   **`routing.rs`**: Screen enum definitions and navigation targets.
*   **`theme.rs`**: Color palette and styling constants.
*   **`window_state.rs`**: Window visibility/focus state tracking.
*   **`handlers/`**: **Logic Implementation**. `app.rs` delegates actual work here.
    *   `navigation.rs`: Handles screen transitions and state rebuilding.
    *   `platform.rs`: OS-specific events (tick, tray, window management).
*   **`features/`**: **Business Logic & Feature UI**. Independent modules implementing specific capabilities.
    *   `account_management`: Account list and auth state.
    *   `account_rules`: Scheduling and availability rules.
    *   `bulk_actions`: Multi-select operations (read, done, archive).
    *   `general_settings`: Theme and app preferences.
    *   `network_proxy`: Proxy configuration.
    *   `notification_details`: Loading and viewing thread details.
    *   `org_rules`: Organization-specific rule management.
    *   `power_mode`: Keyboard-centric power tools.
    *   `rule_overview`: Rule engine dashboard and health metrics.
    *   `thread_actions`: Single-thread operations (snooze, mute, unsubscribe).
    *   `type_rules`: Notification type classification rules.
*   **`screens/`**: **Routing Shells**. Top-level views that compose features.
    *   **`login/`**: OAuth login flow.
    *   **`notifications/`**: The Inbox.
        *   `engine.rs`: **The Executor**. Applies rules to incoming notifications.
        *   `screen.rs`: Main screen shell.
        *   `view/`: Layout composition (sidebar, header, content).
    *   **`settings/`**: Configuration screens.
        *   `screen.rs`: Settings shell (routes to General, Account, Proxy, Rules).
        *   `rule_engine/`: **The Rule Editor**.
            *   `screen.rs`: Editor shell (routes to Overview, Org, Type, Account rules).
            *   `rules.rs`: Core rule types (serialization/evaluation).
            *   `inspector.rs`: Split-view tool to explain rule application.

#### How We Write UI Code

Every screen follows the same shape. Once you've seen one, you've seen them all.

**The core idea: Screens are routing shells. Features own behavior.**

> [!IMPORTANT]
> **Architecture Standard**: "Screens are Shells". Screens hold state and route messages but delegate almost all logic and rendering to separate modules.
>
> 1.  **Behavior Logic** -> `ui/features/<feature>/`
> 2.  **View Logic** -> `ui/features/<feature>/view.rs`
> 3.  **Composed Layout** -> `screen.rs` (using `container().style(theme::app_container)`)
>
> Do not write complex logic or long view functions directly in `screen.rs`.

**The `screen.rs` file always has three things:**

1.  **State Aggregation**: Holds instances of Feature State structs (e.g., `AccountRulesState`).
2.  **Update Routing**: Delegates specific message variants to feature update functions.
3.  **View Composition**: Calls feature functions (e.g., `org_rules::view()`) and wraps them in a consistent layout.

**Feature-Based Architecture (`ui/features/`):**

When a screen accumulates too much behavior, we extract independent behaviors into feature modules:

```
src/ui/features/
├── mod.rs
├── org_rules/          # Example Feature
│   ├── state.rs        # Struct holding data (e.g. OrgRulesState)
│   ├── message.rs      # Enum of actions (e.g. OrgMessage)
│   ├── update.rs       # fn update(state, msg, ...) -> Task
│   ├── view.rs         # fn view(state, ...) -> Element
│   └── widgets/        # (optional) Feature-specific components
│       ├── mod.rs
│       └── rule_card.rs
└── ...
```

**Feature Isolation Rule:**

A feature may not directly mutate state it does not own. Screen-owned collections (like the global `NotificationRuleSet`) may only be modified by passing them as mutable references to the feature's `update` function.

**Message Routing Pattern:**

Screen-level messages are routing wrappers that delegate to feature messages. This keeps the top-level match block clean.

```rust
enum RuleEngineMessage {
    // Navigation / Lifecycle
    Back,
    SelectTab(RuleTab),

    // Feature Delegation
    Org(OrgMessage),
    Account(AccountRuleMessage),
}
```

The `update()` function explicitly delegates:

```rust
fn update(&mut self, msg: RuleEngineMessage) -> Task<RuleEngineMessage> {
    match msg {
        RuleEngineMessage::Org(inner_msg) => {
            org_rules::update(&mut self.org_rules, inner_msg, &mut self.rules)
                .map(RuleEngineMessage::Org)
        }
        // ...
    }
}
```

**UI Components & Theming:**

1.  **Generics**: specialized components (like list items) should be generic over the `Message` type where possible to allow reuse across features.
2.  **Theming**: Always use `ui/theme.rs` and `ui/icons.rs`. Never hardcode colors.
3.  **Containers**: The main view of every screen MUST be wrapped in `container().style(theme::app_container)` to ensure the correct background color (e.g., dark grey instead of default blue).

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
