//! Windows-specific platform implementations.

use std::ffi::CString;

/// Find and focus an existing GitTop window.
pub fn focus_existing_window() {
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextA, IsIconic, IsWindowVisible, SW_RESTORE, SW_SHOW,
        SetForegroundWindow, ShowWindow,
    };

    unsafe extern "system" fn enum_callback(hwnd: HWND, _lparam: LPARAM) -> windows::core::BOOL {
        unsafe {
            // Skip invisible windows
            if !IsWindowVisible(hwnd).as_bool() {
                return windows::core::BOOL::from(true);
            }

            // Get window title
            let mut title = [0u8; 256];
            let len = GetWindowTextA(hwnd, &mut title);

            if len > 0 {
                let title_str = std::str::from_utf8(&title[..len as usize]).unwrap_or("");

                // Check if this is a GitTop window
                if title_str.contains("GitTop") {
                    // Restore if minimized
                    if IsIconic(hwnd).as_bool() {
                        let _ = ShowWindow(hwnd, SW_RESTORE);
                    } else {
                        let _ = ShowWindow(hwnd, SW_SHOW);
                    }

                    // Bring to foreground
                    let _ = SetForegroundWindow(hwnd);

                    // Stop enumeration
                    return windows::core::BOOL::from(false);
                }
            }

            windows::core::BOOL::from(true)
        }
    }

    unsafe {
        let _ = EnumWindows(Some(enum_callback), LPARAM(0));
    }
}

/// Enable dark mode for Windows context menus (system tray).
/// Uses undocumented Windows API SetPreferredAppMode from uxtheme.dll.
pub fn enable_dark_mode() {
    // SetPreferredAppMode ordinal 135 in uxtheme.dll
    // 0 = Default, 1 = AllowDark, 2 = ForceDark, 3 = ForceLight, 4 = Max
    const APPMODE_FORCEDARK: i32 = 2;

    type SetPreferredAppModeFn = unsafe extern "system" fn(i32) -> i32;

    unsafe {
        let lib_name = CString::new("uxtheme.dll").unwrap();
        let lib = windows::Win32::System::LibraryLoader::LoadLibraryA(
            windows::core::PCSTR::from_raw(lib_name.as_ptr() as *const u8),
        );

        if let Ok(handle) = lib {
            // GetProcAddress with ordinal 135
            let func = windows::Win32::System::LibraryLoader::GetProcAddress(
                handle,
                windows::core::PCSTR::from_raw(135 as *const u8),
            );

            if let Some(f) = func {
                let set_preferred_app_mode: SetPreferredAppModeFn = std::mem::transmute(f);
                set_preferred_app_mode(APPMODE_FORCEDARK);
            }
        }
    }
}

/// Initialize the tray subsystem.
/// Windows doesn't require special initialization.
pub fn init_tray() {
    // No-op on Windows - tray-icon works without GTK
}

/// Aggressively trim the process working set to reduce memory footprint.
/// This moves pages to the page file, making the process appear to use less memory.
pub fn trim_working_set() {
    use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
    use windows::Win32::System::Threading::GetCurrentProcess;

    unsafe {
        let _ = EmptyWorkingSet(GetCurrentProcess());
    }
}

/// Send a native Windows toast notification.
///
/// Uses WinRT toast notifications which:
/// - Don't require a background service
/// - Don't keep anything resident
/// - Fire and exit
///
/// If `url` is provided, clicking the notification opens that URL.
pub fn notify(
    title: &str,
    body: &str,
    url: Option<&str>,
) -> Result<(), tauri_winrt_notification::Error> {
    use tauri_winrt_notification::{Duration, Toast};

    let mut toast = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(body)
        .duration(Duration::Short);

    // If URL provided, open it when notification is clicked
    if let Some(url) = url {
        let url_owned = url.to_string();
        toast = toast.on_activated(move |_action| {
            let _ = open::that(&url_owned);
            Ok(())
        });
    }

    // Fire and forget - no handles kept, no memory retained
    toast.show()
}
