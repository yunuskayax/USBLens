//! UsbLens' own plug/unplug history (JSON Lines in the app data dir).
//! Windows only remembers the *last* arrival/removal per device; this log keeps every one.

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::model::{DeviceKind, RiskLevel, UsbEvent};

/// Keep the file bounded: when it grows past MAX, the oldest entries are dropped down to KEEP.
const MAX_EVENTS: usize = 10_000;
const KEEP_EVENTS: usize = 8_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct LogEvent {
    /// RFC 3339, UTC
    pub at: String,
    /// "arrival" | "removal"
    pub action: String,
    pub instance_id: String,
    pub name: String,
    pub kind: Option<DeviceKind>,
    pub vendor_id: Option<String>,
    pub product_id: Option<String>,
    pub serial_number: Option<String>,
    pub drive_letters: Vec<String>,
    /// Windows user signed in when it happened
    pub user: String,
    pub alerts: Vec<String>,
    pub level: RiskLevel,
    pub blocked: bool,
    pub block_error: Option<String>,
}

impl Default for LogEvent {
    fn default() -> Self {
        Self {
            at: String::new(),
            action: String::new(),
            instance_id: String::new(),
            name: String::new(),
            kind: None,
            vendor_id: None,
            product_id: None,
            serial_number: None,
            drive_letters: Vec::new(),
            user: String::new(),
            alerts: Vec::new(),
            level: RiskLevel::None,
            blocked: false,
            block_error: None,
        }
    }
}

impl LogEvent {
    pub fn from_event(ev: &UsbEvent, at: String, user: String) -> Self {
        let d = ev.device.as_ref();
        Self {
            at,
            action: ev.action.clone(),
            instance_id: ev
                .instance_id
                .clone()
                .or_else(|| d.map(|d| d.instance_id.clone()))
                .unwrap_or_default(),
            name: d.map(|d| d.name.clone()).or_else(|| ev.instance_id.clone()).unwrap_or_default(),
            kind: d.map(|d| d.kind),
            vendor_id: d.and_then(|d| d.vendor_id.clone()),
            product_id: d.and_then(|d| d.product_id.clone()),
            serial_number: d.and_then(|d| d.serial_number.clone()),
            drive_letters: d.map(|d| d.drive_letters.clone()).unwrap_or_default(),
            user,
            alerts: ev.alerts.clone(),
            level: ev.level,
            blocked: ev.blocked,
            block_error: ev.block_error.clone(),
        }
    }
}

pub struct EventLog {
    path: PathBuf,
    lock: Mutex<()>,
}

impl EventLog {
    pub fn new(path: PathBuf) -> Self {
        Self { path, lock: Mutex::new(()) }
    }

    fn read_all(&self) -> Vec<LogEvent> {
        let Ok(f) = std::fs::File::open(&self.path) else { return Vec::new() };
        BufReader::new(f)
            .lines()
            .map_while(Result::ok)
            .filter_map(|l| serde_json::from_str::<LogEvent>(&l).ok())
            .collect()
    }

    fn write_all(&self, events: &[LogEvent]) -> Result<(), String> {
        let tmp = self.path.with_extension("jsonl.tmp");
        {
            let mut f = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
            for e in events {
                let line = serde_json::to_string(e).map_err(|e| e.to_string())?;
                writeln!(f, "{line}").map_err(|e| e.to_string())?;
            }
        }
        std::fs::rename(&tmp, &self.path).map_err(|e| e.to_string())
    }

    pub fn append(&self, ev: &LogEvent) -> Result<(), String> {
        let _g = self.lock.lock().map_err(|e| e.to_string())?;
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let line = serde_json::to_string(ev).map_err(|e| e.to_string())?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| e.to_string())?;
        writeln!(f, "{line}").map_err(|e| e.to_string())?;
        drop(f);

        // Cheap size check first; only re-read when the file is big.
        let big = std::fs::metadata(&self.path).map(|m| m.len() > (MAX_EVENTS as u64) * 200).unwrap_or(false);
        if big {
            let all = self.read_all();
            if all.len() > MAX_EVENTS {
                self.write_all(&all[all.len() - KEEP_EVENTS..])?;
            }
        }
        Ok(())
    }

    /// Newest first.
    pub fn list(&self, limit: usize) -> Vec<LogEvent> {
        let _g = self.lock.lock();
        let mut all = self.read_all();
        all.reverse();
        all.truncate(limit);
        all
    }

    pub fn clear(&self) -> Result<(), String> {
        let _g = self.lock.lock().map_err(|e| e.to_string())?;
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

fn csv_cell(s: &str) -> String {
    let s = if s.starts_with(['=', '+', '-', '@']) { format!("'{s}") } else { s.to_string() };
    if s.contains([',', '"', '\n', '\r', ';']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s
    }
}

pub fn to_csv(events: &[LogEvent], lang: &str) -> String {
    let tr = lang == "tr";
    let head = if tr {
        ["Zaman (UTC)", "Olay", "Cihaz", "VID", "PID", "Seri No", "Sürücü", "Kullanıcı", "Uyarılar", "Engellendi", "Instance ID"]
    } else {
        ["Time (UTC)", "Event", "Device", "VID", "PID", "Serial", "Drive", "User", "Alerts", "Blocked", "Instance ID"]
    };
    let mut out = String::from("\u{feff}");
    out.push_str(&head.iter().map(|h| csv_cell(h)).collect::<Vec<_>>().join(","));
    out.push_str("\r\n");
    for e in events {
        let action = match (e.action.as_str(), tr) {
            ("arrival", true) => "Takıldı",
            ("removal", true) => "Çıkarıldı",
            ("arrival", false) => "Plugged in",
            ("removal", false) => "Unplugged",
            (a, _) => a,
        };
        let row = [
            e.at.as_str(),
            action,
            e.name.as_str(),
            e.vendor_id.as_deref().unwrap_or(""),
            e.product_id.as_deref().unwrap_or(""),
            e.serial_number.as_deref().unwrap_or(""),
            &e.drive_letters.join(" "),
            e.user.as_str(),
            &e.alerts.join(" | "),
            if e.blocked { if tr { "Evet" } else { "Yes" } } else { "" },
            e.instance_id.as_str(),
        ];
        out.push_str(&row.iter().map(|c| csv_cell(c)).collect::<Vec<_>>().join(","));
        out.push_str("\r\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(i: usize) -> LogEvent {
        LogEvent { at: format!("2026-11-17T00:00:{:02}Z", i % 60), action: "arrival".into(), name: format!("dev{i}"), ..Default::default() }
    }

    #[test]
    fn appends_lists_newest_first_and_clears() {
        let dir = std::env::temp_dir().join(format!("usblens-events-{}", std::process::id()));
        let log = EventLog::new(dir.join("events.jsonl"));
        for i in 0..5 {
            log.append(&ev(i)).unwrap();
        }
        let l = log.list(3);
        assert_eq!(l.len(), 3);
        assert_eq!(l[0].name, "dev4");
        log.clear().unwrap();
        assert!(log.list(10).is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn trims_when_too_big() {
        let dir = std::env::temp_dir().join(format!("usblens-trim-{}", std::process::id()));
        let log = EventLog::new(dir.join("events.jsonl"));
        let big: Vec<LogEvent> = (0..MAX_EVENTS + 5).map(ev).collect();
        std::fs::create_dir_all(&dir).unwrap();
        log.write_all(&big).unwrap();
        // Pad the file so the size check fires.
        log.append(&LogEvent { name: "x".repeat(MAX_EVENTS * 200), ..ev(0) }).unwrap();
        assert_eq!(log.list(usize::MAX).len(), KEEP_EVENTS);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn csv_has_header_and_rows() {
        let csv = to_csv(&[ev(1)], "tr");
        assert!(csv.contains("Takıldı"));
        assert_eq!(csv.lines().count(), 2);
    }
}
