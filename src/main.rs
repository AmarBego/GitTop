#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! GitTop - A beautiful native GitHub notification manager
//! No browser engine required. Pure Rust. Pure performance.

mod cache;
mod diagnostics;
mod github;
mod platform;
mod settings;
mod specs;
mod tray;
mod ui;
mod update_checker;

use single_instance::SingleInstance;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Mutex name for single instance detection
const SINGLE_INSTANCE_MUTEX: &str = "GitTop-SingleInstance-Mutex-7a8b9c0d";

/// Global mock notification count (set via CLI)
pub static MOCK_NOTIFICATION_COUNT: AtomicUsize = AtomicUsize::new(0);

static LOG_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

const LOG_FILE_PREFIX: &str = "gittop.log";
const LOG_RETENTION_FILES: usize = 7;

fn parse_cli_args() {
    let mut args = std::env::args().skip(1).peekable();

    while let Some(arg) = args.next() {
        if matches!(arg.as_str(), "--mock-notifications" | "-m")
            && let Some(Ok(count)) = args.next().map(|s| s.parse::<usize>())
        {
            MOCK_NOTIFICATION_COUNT.store(count, Ordering::Relaxed);
        }
    }
}

fn init_logging() {
    let crate_target = env!("CARGO_PKG_NAME");
    let crate_target_lc = crate_target.to_lowercase();
    let env_value = std::env::var("RUST_LOG").ok();
    let log_dir = diagnostics::log_directory();
    let mut log_dir_error: Option<(PathBuf, String)> = None;
    let mut file_logging_enabled = false;

    if log_dir.is_none() {
        log_dir_error = Some((PathBuf::from("<none>"), "No config directory".to_string()));
    }

    let mut filter = match env_value.as_deref().map(str::trim) {
        Some(value) if !value.is_empty() && !value.contains('=') && !value.contains(',') => {
            build_scoped_filter(crate_target, &crate_target_lc, value)
        }
        Some(value) if !value.is_empty() => value
            .parse()
            .unwrap_or_else(|_| build_default_filter(crate_target, &crate_target_lc)),
        _ => build_default_filter(crate_target, &crate_target_lc),
    };

    filter = add_dependency_filters(filter, env_value.as_deref());

    let file_layer = log_dir.as_ref().and_then(|dir| {
        if let Err(e) = std::fs::create_dir_all(dir) {
            log_dir_error = Some((dir.clone(), e.to_string()));
            return None;
        }

        prune_old_logs(dir, LOG_RETENTION_FILES);

        let file_appender = tracing_appender::rolling::daily(dir, LOG_FILE_PREFIX);
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        let _ = LOG_GUARD.set(guard);
        file_logging_enabled = true;

        Some(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(false),
        )
    });

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_ansi(cfg!(debug_assertions));

    if let Some(file_layer) = file_layer {
        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(stdout_layer)
            .with(file_layer)
            .try_init();
    } else {
        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(stdout_layer)
            .try_init();
    }

    if let Some((dir, error)) = log_dir_error {
        tracing::warn!(
            path = %dir.display(),
            error = %error,
            "File logging disabled"
        );
    } else if file_logging_enabled && let Some(dir) = log_dir {
        tracing::info!(path = %dir.display(), "File logging enabled");
    }
}

fn add_dependency_filters(
    mut filter: tracing_subscriber::EnvFilter,
    env_value: Option<&str>,
) -> tracing_subscriber::EnvFilter {
    const NOISY_TARGETS: [&str; 6] = [
        "wgpu",
        "wgpu_core",
        "wgpu_hal",
        "winit",
        "iced_wgpu",
        "iced_winit",
    ];

    for target in NOISY_TARGETS {
        if !env_mentions_target(env_value, target)
            && let Ok(directive) = format!("{target}=warn").parse()
        {
            filter = filter.add_directive(directive);
        }
    }

    filter
}

fn build_default_filter(
    crate_target: &str,
    crate_target_lc: &str,
) -> tracing_subscriber::EnvFilter {
    build_scoped_filter(crate_target, crate_target_lc, "info")
}

fn build_scoped_filter(
    crate_target: &str,
    crate_target_lc: &str,
    level: &str,
) -> tracing_subscriber::EnvFilter {
    let mut directives = vec![format!("{crate_target}={level}")];
    if crate_target_lc != crate_target {
        directives.push(format!("{crate_target_lc}={level}"));
    }
    tracing_subscriber::EnvFilter::new(directives.join(","))
}

fn env_mentions_target(env_value: Option<&str>, target: &str) -> bool {
    let Some(value) = env_value else {
        return false;
    };

    value.split(',').any(|part| {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix(target) {
            rest.starts_with('=') || rest.starts_with(':')
        } else {
            false
        }
    })
}

fn prune_old_logs(dir: &Path, max_files: usize) {
    if max_files == 0 {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    let mut log_files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.starts_with(LOG_FILE_PREFIX) {
            continue;
        }
        let modified = entry.metadata().and_then(|m| m.modified()).ok();
        log_files.push((modified, path));
    }

    log_files.sort_by_key(|(modified, _)| *modified);

    if log_files.len() <= max_files {
        return;
    }

    let remove_count = log_files.len() - max_files;
    for (_, path) in log_files.into_iter().take(remove_count) {
        let _ = std::fs::remove_file(path);
    }
}

fn log_startup_diagnostics() {
    let settings = settings::AppSettings::load();
    let rules = ui::screens::settings::rule_engine::rules::NotificationRuleSet::load();

    tracing::info!(
        app = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        "GitTop started"
    );

    if let Some(config_dir) = dirs::config_dir() {
        let base = config_dir.join("GitTop");
        tracing::info!(path = %base.display(), "Config directory");
        tracing::debug!(
            settings_path = %base.join("settings.json").display(),
            rules_path = %base.join("rules.json").display(),
            "Config paths"
        );
    }

    tracing::info!(
        theme = %settings.theme,
        icon_theme = ?settings.icon_theme,
        minimize_to_tray = settings.minimize_to_tray,
        power_mode = settings.power_mode,
        show_details_panel = settings.show_details_panel,
        proxy_enabled = settings.proxy.enabled,
        proxy_has_credentials = settings.proxy.has_credentials,
        "Settings snapshot"
    );

    tracing::info!(
        rules_enabled = rules.enabled,
        rule_set = %rules.name,
        account_rules = rules.account_rules.len(),
        org_rules = rules.org_rules.len(),
        type_rules = rules.type_rules.len(),
        active_rules = rules.active_rule_count(),
        high_impact_rules = rules.get_high_impact_rules().len(),
        "Rules snapshot"
    );
}

fn main() -> iced::Result {
    // Force OpenGL backend for wgpu to minimize memory footprint
    // OpenGL uses ~42MB vs Vulkan's ~164MB or DX12's ~133MB
    // Safety: This is called at program start before any threads are spawned
    unsafe { std::env::set_var("WGPU_BACKEND", "gl") };

    init_logging();
    diagnostics::install_panic_hook();
    log_startup_diagnostics();

    // Parse CLI arguments (e.g., --mock-notifications 1000)
    parse_cli_args();

    let instance =
        SingleInstance::new(SINGLE_INSTANCE_MUTEX).expect("Failed to create single-instance mutex");

    if !instance.is_single() {
        platform::focus_existing_window();
        return Ok(());
    }

    platform::enable_dark_mode();

    let _tray = match tray::TrayManager::new() {
        Ok(t) => Some(t),
        Err(e) => {
            tracing::warn!(error = %e, "Tray unavailable");
            None
        }
    };

    let result = platform::run_app();
    if let Err(e) = result.as_ref() {
        diagnostics::write_fatal_error(e);
    }
    result
}
