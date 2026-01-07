//! Diagnostics and crash reporting.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::Utc;

use crate::github::redaction::redact_secrets;

#[derive(Debug, Clone)]
pub struct CrashNotice {
    pub report_path: PathBuf,
    pub log_dir: Option<PathBuf>,
}

pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let payload = panic_payload(info);
        let location = info
            .location()
            .map(|loc| format!("{}:{}", loc.file(), loc.line()))
            .unwrap_or_else(|| "unknown".to_string());
        let thread = std::thread::current()
            .name()
            .unwrap_or("unnamed")
            .to_string();
        let backtrace = std::backtrace::Backtrace::force_capture();

        let report = format!(
            "GitTop crash report\n\
Timestamp (UTC): {}\n\
Thread: {}\n\
Location: {}\n\
Panic: {}\n\
\n\
Backtrace:\n\
{}\n",
            Utc::now().to_rfc3339(),
            thread,
            location,
            payload,
            backtrace
        );

        let report = redact_secrets(&report);
        if let Err(e) = write_crash_report(&report) {
            tracing::error!(error = %e, "Failed to write crash report");
        }

        tracing::error!(
            panic_message = %redact_secrets(&payload),
            location = %location,
            "Unexpected panic"
        );
    }));
}

pub fn write_fatal_error(error: &dyn std::error::Error) {
    let backtrace = std::backtrace::Backtrace::force_capture();
    let report = format!(
        "GitTop fatal error\n\
Timestamp (UTC): {}\n\
Error: {}\n\
\n\
Backtrace:\n\
{}\n",
        Utc::now().to_rfc3339(),
        error,
        backtrace
    );

    let report = redact_secrets(&report);
    if let Err(e) = write_crash_report(&report) {
        tracing::error!(error = %e, "Failed to write crash report");
    }
}

pub fn load_crash_notice() -> Option<CrashNotice> {
    let report_path = crash_report_path()?;
    if !report_path.exists() {
        return None;
    }

    Some(CrashNotice {
        report_path,
        log_dir: log_directory(),
    })
}

pub fn clear_crash_notice() {
    if let Some(path) = crash_report_path() {
        let _ = fs::remove_file(path);
    }
}

pub fn log_directory() -> Option<PathBuf> {
    config_dir_base().map(|p| p.join("logs"))
}

fn crash_report_path() -> Option<PathBuf> {
    config_dir_base().map(|p| p.join("crash-report.txt"))
}

fn config_dir_base() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("GitTop"))
}

fn write_crash_report(contents: &str) -> Result<(), std::io::Error> {
    let Some(path) = crash_report_path() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No config directory",
        ));
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, contents)?;
    Ok(())
}

fn panic_payload(info: &std::panic::PanicHookInfo<'_>) -> String {
    if let Some(payload) = info.payload().downcast_ref::<&str>() {
        (*payload).to_string()
    } else if let Some(payload) = info.payload().downcast_ref::<String>() {
        payload.clone()
    } else {
        "Unknown panic payload".to_string()
    }
}

#[allow(dead_code)]
fn report_age_seconds(path: &Path) -> Option<u64> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let age = SystemTime::now().duration_since(modified).ok()?;
    Some(age.as_secs())
}
