//! Simple update check: download a small JSON manifest and compare versions.
//!
//! Manifest (hosted at `MANIFEST_URL`):
//! ```json
//! { "version": "0.2.2", "url": "https://github.com/yunuskayax/USBLens/releases/latest/download/UsbLens-setup.exe",
//!   "page": "https://usblens.com", "notes": "…", "date": "2026-11-16" }
//! ```

use std::time::Duration;

use serde::{Deserialize, Serialize};

pub const MANIFEST_URL: &str = "https://usblens.com/latest.json";

#[derive(Debug, Deserialize)]
struct Manifest {
    version: String,
    url: Option<String>,
    page: Option<String>,
    notes: Option<String>,
    date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current: String,
    pub latest: String,
    pub available: bool,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub date: Option<String>,
}

/// "v1.2.3-beta" -> [1, 2, 3]
fn parse_version(v: &str) -> Vec<u64> {
    v.trim()
        .trim_start_matches(['v', 'V'])
        .split(['-', '+'])
        .next()
        .unwrap_or("")
        .split('.')
        .map(|p| p.parse().unwrap_or(0))
        .collect()
}

pub fn is_newer(latest: &str, current: &str) -> bool {
    let (mut a, mut b) = (parse_version(latest), parse_version(current));
    let n = a.len().max(b.len());
    a.resize(n, 0);
    b.resize(n, 0);
    a > b
}

/// Only allow https links on our own site or GitHub to be opened from the UI.
fn safe_url(u: Option<String>) -> Option<String> {
    u.filter(|u| {
        u.starts_with("https://usblens.com/")
            || u.starts_with("https://www.usblens.com/")
            || u.starts_with("https://yunuskaya.com.tr/")
            || u.starts_with("https://www.yunuskaya.com.tr/")
            || u.starts_with("https://github.com/")
    })
}

pub fn check(current: &str) -> Result<UpdateInfo, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .user_agent(format!("UsbLens/{current}"))
        .build()
        .into();
    let manifest: Manifest = agent
        .get(MANIFEST_URL)
        .call()
        .map_err(|e| format!("UPDATE_FAILED:{e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("UPDATE_FAILED:{e}"))?;

    Ok(UpdateInfo {
        current: current.to_string(),
        available: is_newer(&manifest.version, current),
        latest: manifest.version,
        url: safe_url(manifest.url).or_else(|| safe_url(manifest.page)),
        notes: manifest.notes,
        date: manifest.date,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("v1.0", "0.9.9"));
        assert!(is_newer("0.1.10", "0.1.9"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.1.0-beta", "0.1.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
    }

    #[test]
    fn filters_urls() {
        assert!(safe_url(Some("https://yunuskaya.com.tr/x.exe".into())).is_some());
        assert!(safe_url(Some("https://usblens.com/".into())).is_some());
        assert!(safe_url(Some("http://yunuskaya.com.tr/x.exe".into())).is_none());
        assert!(safe_url(Some("https://evil.example/x.exe".into())).is_none());
    }
}
