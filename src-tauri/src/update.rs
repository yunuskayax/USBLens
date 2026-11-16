//! Update check.
//!
//! 1. Primary: the latest published GitHub Release (`GITHUB_LATEST_URL`). Tag `v0.2.3` → version
//!    0.2.3, release notes → notes, `UsbLens-setup.exe` asset → download link. Nothing to maintain
//!    by hand: publishing a release is enough.
//! 2. Fallback (GitHub unreachable / rate-limited): a small manifest at `MANIFEST_URL`:
//! ```json
//! { "version": "0.2.2", "url": "https://github.com/yunuskayax/USBLens/releases/latest/download/UsbLens-setup.exe",
//!   "page": "https://usblens.com", "notes": "…", "date": "2026-11-16" }
//! ```

use std::time::Duration;

use serde::{Deserialize, Serialize};

pub const MANIFEST_URL: &str = "https://usblens.com/latest.json";
pub const GITHUB_LATEST_URL: &str = "https://api.github.com/repos/yunuskayax/USBLens/releases/latest";
const RELEASES_PAGE: &str = "https://github.com/yunuskayax/USBLens/releases/latest";

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    published_at: Option<String>,
    #[serde(default)]
    assets: Vec<GithubAsset>,
}

impl GithubRelease {
    /// "v0.2.3" in the tag, or failing that in the title ("UsbLens 0.2.3").
    fn version(&self) -> Option<String> {
        [Some(self.tag_name.as_str()), self.name.as_deref()]
            .into_iter()
            .flatten()
            .find_map(extract_version)
    }

    fn installer_url(&self) -> Option<String> {
        let pick = |pred: &dyn Fn(&str) -> bool| {
            self.assets
                .iter()
                .find(|a| pred(&a.name.to_ascii_lowercase()))
                .map(|a| a.browser_download_url.clone())
        };
        pick(&|n| n == "usblens-setup.exe")
            .or_else(|| pick(&|n| n.ends_with("-setup.exe")))
            .or_else(|| pick(&|n| n.ends_with(".msi")))
    }
}

/// Find the first "1.2" / "1.2.3" looking token in a string.
fn extract_version(s: &str) -> Option<String> {
    s.split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .map(|t| t.trim_matches('.'))
        .find(|t| t.contains('.') && t.split('.').all(|p| !p.is_empty()))
        .map(str::to_string)
}

/// Trim long release notes for the settings card.
fn short_notes(body: Option<String>) -> Option<String> {
    let body = body?.trim().to_string();
    if body.is_empty() {
        return None;
    }
    let mut out: String = body.chars().take(600).collect();
    if body.chars().count() > 600 {
        out.push('…');
    }
    Some(out)
}

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

fn agent(current: &str) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .user_agent(format!("UsbLens/{current}"))
        .build()
        .into()
}

fn from_github(agent: &ureq::Agent, current: &str) -> Result<UpdateInfo, String> {
    let release: GithubRelease = agent
        .get(GITHUB_LATEST_URL)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_json()
        .map_err(|e| e.to_string())?;
    let latest = release.version().ok_or("latest release has no version number")?;
    Ok(UpdateInfo {
        current: current.to_string(),
        available: is_newer(&latest, current),
        url: safe_url(release.installer_url()).or_else(|| Some(RELEASES_PAGE.to_string())),
        notes: short_notes(release.body),
        date: release.published_at.map(|d| d.chars().take(10).collect()),
        latest,
    })
}

fn from_manifest(agent: &ureq::Agent, current: &str) -> Result<UpdateInfo, String> {
    let manifest: Manifest = agent
        .get(MANIFEST_URL)
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_json()
        .map_err(|e| e.to_string())?;
    Ok(UpdateInfo {
        current: current.to_string(),
        available: is_newer(&manifest.version, current),
        latest: manifest.version,
        url: safe_url(manifest.url).or_else(|| safe_url(manifest.page)),
        notes: manifest.notes,
        date: manifest.date,
    })
}

pub fn check(current: &str) -> Result<UpdateInfo, String> {
    let agent = agent(current);
    match from_github(&agent, current) {
        Ok(info) => Ok(info),
        Err(github_err) => from_manifest(&agent, current)
            .map_err(|site_err| format!("UPDATE_FAILED:github: {github_err}; site: {site_err}")),
    }
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
    fn reads_github_releases() {
        let rel: GithubRelease = serde_json::from_str(r#"{
            "tag_name": "v0.2.3", "name": "UsbLens v0.2.3", "body": "notes", "published_at": "2026-11-20T10:00:00Z",
            "assets": [
              {"name": "UsbLens_0.2.3_x64_tr-TR.msi", "browser_download_url": "https://github.com/a/b/x.msi"},
              {"name": "UsbLens-setup.exe", "browser_download_url": "https://github.com/a/b/UsbLens-setup.exe"}
            ]}"#).unwrap();
        assert_eq!(rel.version().as_deref(), Some("0.2.3"));
        assert!(rel.installer_url().unwrap().ends_with("UsbLens-setup.exe"));

        // A tag that is not a version ("İndir") falls back to the title, or yields None.
        let rel: GithubRelease = serde_json::from_str(r#"{"tag_name": "İndir", "name": "İndir"}"#).unwrap();
        assert_eq!(rel.version(), None);
        assert_eq!(extract_version("UsbLens 1.4"), Some("1.4".into()));
    }

    #[test]
    fn filters_urls() {
        assert!(safe_url(Some("https://yunuskaya.com.tr/x.exe".into())).is_some());
        assert!(safe_url(Some("https://usblens.com/".into())).is_some());
        assert!(safe_url(Some("http://yunuskaya.com.tr/x.exe".into())).is_none());
        assert!(safe_url(Some("https://evil.example/x.exe".into())).is_none());
    }
}
