use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub const MAX_LOG_ENTRIES: usize = 1000;
pub const MAX_LOG_FILE_BYTES: u64 = 512 * 1024; // 512 KB
pub const LOG_FILE_NAME: &str = "auri.log";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Stream,
    Audio,
    Info,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Stream => "STREAM",
            LogLevel::Audio => "AUDIO",
            LogLevel::Info => "INFO",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub component: &'static str,
    pub message: String,
}

struct LogStore {
    entries: VecDeque<LogEntry>,
    log_file: Option<PathBuf>,
}

static STORE: OnceLock<Mutex<LogStore>> = OnceLock::new();

fn get_store() -> &'static Mutex<LogStore> {
    STORE.get_or_init(|| {
        Mutex::new(LogStore {
            entries: VecDeque::with_capacity(MAX_LOG_ENTRIES),
            log_file: None,
        })
    })
}

fn current_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = now.as_secs();
    let hours = (total_secs / 3600) % 24;
    let mins = (total_secs / 60) % 60;
    let secs = total_secs % 60;
    let millis = now.subsec_millis();
    format!("{hours:02}:{mins:02}:{secs:02}.{millis:03}")
}

pub fn init(folder: Option<PathBuf>) {
    let mut store = get_store().lock().unwrap();
    if let Some(folder) = folder {
        let _ = std::fs::create_dir_all(&folder);
        let log_path = folder.join(LOG_FILE_NAME);
        store.log_file = Some(log_path.clone());

        // Header for new session
        let ts = current_timestamp();
        let header = format!(
            "\n--- AURI Log Session Started at {} (UTC) ---\n",
            ts
        );
        let _ = append_to_file(&log_path, &header);
    }
}

fn append_to_file(path: &PathBuf, text: &str) -> std::io::Result<()> {
    let grown = std::fs::metadata(path).is_ok_and(|meta| meta.len() > MAX_LOG_FILE_BYTES);
    let mut file = OpenOptions::new()
        .create(true)
        .append(!grown)
        .write(true)
        .truncate(grown)
        .open(path)?;
    file.write_all(text.as_bytes())
}

pub fn log(level: LogLevel, component: &'static str, message: String) {
    let timestamp = current_timestamp();
    let formatted = format!("[{}] [{}] [{}] {}\n", timestamp, level.as_str(), component, message);

    // Also echo to stderr in dev/console mode
    #[cfg(debug_assertions)]
    eprint!("{formatted}");

    let mut store = get_store().lock().unwrap();
    if let Some(log_path) = &store.log_file {
        let _ = append_to_file(log_path, &formatted);
    }

    if store.entries.len() >= MAX_LOG_ENTRIES {
        store.entries.pop_front();
    }
    store.entries.push_back(LogEntry {
        timestamp,
        level,
        component,
        message,
    });
}

pub fn entries() -> Vec<LogEntry> {
    let store = get_store().lock().unwrap();
    store.entries.iter().cloned().collect()
}

pub fn clear() {
    let mut store = get_store().lock().unwrap();
    store.entries.clear();
}

pub fn log_file_path() -> Option<PathBuf> {
    let store = get_store().lock().unwrap();
    store.log_file.clone()
}

pub fn export_report(app_summary: &str) -> String {
    let store = get_store().lock().unwrap();
    let mut out = String::with_capacity(32 * 1024);
    out.push_str("=========================================\n");
    out.push_str("          AURI DIAGNOSTIC REPORT         \n");
    out.push_str("=========================================\n");
    out.push_str(app_summary);
    out.push_str("\n\n---------------- LOGS -------------------\n");
    for entry in &store.entries {
        out.push_str(&format!(
            "[{}] [{:6}] [{}] {}\n",
            entry.timestamp,
            entry.level.as_str(),
            entry.component,
            entry.message
        ));
    }
    out.push_str("=========================================\n");
    out
}

#[macro_export]
macro_rules! log_error {
    ($comp:expr, $($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Error, $comp, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($comp:expr, $($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Warn, $comp, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_stream {
    ($comp:expr, $($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Stream, $comp, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_audio {
    ($comp:expr, $($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Audio, $comp, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($comp:expr, $($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Info, $comp, format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_ring_buffer_and_levels() {
        let temp_dir = std::env::temp_dir().join("auri_test_logs");
        init(Some(temp_dir.clone()));
        clear();
        log(LogLevel::Info, "Test", "Hello Info".to_string());
        log(LogLevel::Error, "Test", "Hello Error".to_string());
        log(LogLevel::Stream, "Stream", "Hello Stream".to_string());
        log(LogLevel::Audio, "Audio", "Hello Audio".to_string());

        let list = entries();
        assert_eq!(list.len(), 4);
        assert_eq!(list[0].level, LogLevel::Info);
        assert_eq!(list[1].level, LogLevel::Error);
        assert_eq!(list[2].level, LogLevel::Stream);
        assert_eq!(list[3].level, LogLevel::Audio);

        let report = export_report("Test System Summary");
        assert!(report.contains("AURI DIAGNOSTIC REPORT"));
        assert!(report.contains("Test System Summary"));
        assert!(report.contains("Hello Error"));

        let log_file = temp_dir.join(LOG_FILE_NAME);
        assert!(log_file.exists());
        let content = std::fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("Hello Info"));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
