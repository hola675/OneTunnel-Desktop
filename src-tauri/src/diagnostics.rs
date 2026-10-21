//! Structured local diagnostics: codes only, no credentials or untrusted text.
use serde::{Deserialize, Serialize};
use std::{fs::{self, OpenOptions}, io::Write, path::PathBuf, sync::Mutex, time::{SystemTime, UNIX_EPOCH}};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event { pub timestamp_ms: u128, pub level: String, pub component: String, pub code: String }

pub struct Diagnostics { path: PathBuf, lock: Mutex<()> }
impl Diagnostics {
    pub fn new(path: PathBuf) -> Self { Self { path, lock: Mutex::new(()) } }
    pub fn record(&self, level: &'static str, component: &'static str, code: &'static str) {
        if !["INFO", "WARN", "ERROR"].contains(&level) { return; }
        if ![component, code].iter().all(|s| s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')) { return; }
        let Ok(_guard) = self.lock.lock() else { return; };
        if let Some(parent) = self.path.parent() { if fs::create_dir_all(parent).is_err() { return; } }
        if fs::metadata(&self.path).is_ok_and(|m| m.len() >= 1_048_576) {
            let previous = self.path.with_extension("previous.jsonl");
            let _ = fs::remove_file(&previous);
            if fs::rename(&self.path, previous).is_err() { return; }
        }
        let entry = Event { timestamp_ms: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(), level: level.into(), component: component.into(), code: code.into() };
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.path) {
            if let Ok(line) = serde_json::to_string(&entry) { let _ = writeln!(file, "{line}"); }
        }
    }
    pub fn recent(&self) -> Result<Vec<Event>, String> {
        let _guard = self.lock.lock().map_err(|_| "Diagnostics unavailable")?;
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(_) => return Err("Cannot read diagnostics".into()),
        };
        Ok(text.lines().rev().take(200).filter_map(|line| serde_json::from_str(line).ok()).collect())
    }
}
