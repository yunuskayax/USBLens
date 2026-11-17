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

use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const MANIFEST_URL: &str = "https://usblens.com/latest.json";
pub const GITHUB_LATEST_URL: &str = "https://api.github.com/repos/yunuskayax/USBLens/releases/latest";
const RELEASES_PAGE: &str = "https://github.com/yunuskayax/USBLens/releases/latest";

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
    /// "sha256:<hex>" — provided by the GitHub API for release assets.
    digest: Option<String>,
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

    fn installer(&self) -> Option<&GithubAsset> {
        let pick = |pred: &dyn Fn(&str) -> bool| self.assets.iter().find(|a| pred(&a.name.to_ascii_lowercase()));
        pick(&|n| n == "usblens-setup.exe").or_else(|| pick(&|n| n.ends_with("-setup.exe")))
    }

    fn installer_url(&self) -> Option<String> {
        self.installer().map(|a| a.browser_download_url.clone()).or_else(|| {
            self.assets
                .iter()
                .find(|a| a.name.to_ascii_lowercase().ends_with(".msi"))
                .map(|a| a.browser_download_url.clone())
        })
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
    let mut out: String = body.chars().take(4000).collect();
    if body.chars().count() > 4000 {
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
    /// Page / file to open in the browser (fallback).
    pub url: Option<String>,
    pub notes: Option<String>,
    pub date: Option<String>,
    /// Direct link to the NSIS installer for the in-app updater.
    pub installer: Option<String>,
    pub sha256: Option<String>,
    pub size: Option<u64>,
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
            || u.starts_with("https://github.com/")
    })
}

/// Use the operating system's certificate store (works behind corporate / school TLS proxies).
fn tls() -> ureq::tls::TlsConfig {
    ureq::tls::TlsConfig::builder()
        .root_certs(ureq::tls::RootCerts::PlatformVerifier)
        .build()
}

fn agent(current: &str) -> ureq::Agent {
    ureq::Agent::config_builder()
        .tls_config(tls())
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
    let asset = release.installer();
    let installer = asset.map(|a| a.browser_download_url.clone()).filter(|u| is_installer_url(u));
    let sha256 = asset
        .and_then(|a| a.digest.as_deref())
        .and_then(|d| d.strip_prefix("sha256:"))
        .map(str::to_ascii_lowercase);
    let size = asset.map(|a| a.size).filter(|s| *s > 0);
    Ok(UpdateInfo {
        current: current.to_string(),
        available: is_newer(&latest, current),
        installer,
        sha256,
        size,
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
    let installer = manifest.url.clone().filter(|u| is_installer_url(u));
    Ok(UpdateInfo {
        current: current.to_string(),
        available: is_newer(&manifest.version, current),
        latest: manifest.version,
        installer,
        sha256: None,
        size: None,
        url: safe_url(manifest.url).or_else(|| safe_url(manifest.page)),
        notes: manifest.notes,
        date: manifest.date,
    })
}

// ---------------------------------------------------------------- in-app updater

/// Only installers attached to this repository's releases can be downloaded and run.
pub fn is_installer_url(u: &str) -> bool {
    let l = u.to_ascii_lowercase();
    l.starts_with("https://github.com/yunuskayax/usblens/releases/") && l.ends_with(".exe")
}

const UPDATE_FILE_PREFIX: &str = "UsbLens-update-";

/// Download the installer to %TEMP%, reporting progress, and verify it (SHA-256, PE header).
///
/// Some networks break one way of connecting but not another (antivirus HTTPS scanning,
/// certificate revocation checks, system proxies). So three methods are tried in order:
/// 1. HTTPS with the Windows certificate store, 2. HTTPS with the built-in Mozilla roots,
/// 3. Windows' own downloader via PowerShell (uses the system proxy settings).
pub fn download<F: FnMut(u64, Option<u64>)>(
    current: &str,
    url: &str,
    sha256: Option<&str>,
    mut progress: F,
) -> Result<PathBuf, String> {
    if !is_installer_url(url) {
        return Err("UPDATE_URL_REJECTED".into());
    }
    let path = std::env::temp_dir().join(format!("{UPDATE_FILE_PREFIX}{}.exe", std::process::id()));
    let mut errors: Vec<String> = Vec::new();

    let mut ok = false;
    for (label, tls_config) in [("windows-tls", Some(tls())), ("builtin-tls", None)] {
        match fetch_with_ureq(current, url, &path, tls_config, &mut progress) {
            Ok(()) => {
                ok = true;
                break;
            }
            Err(e) => errors.push(format!("{label}: {e}")),
        }
    }
    if !ok {
        progress(0, None); // indeterminate while Windows downloads
        match fetch_with_windows(url, &path) {
            Ok(()) => ok = true,
            Err(e) => errors.push(format!("windows: {e}")),
        }
    }
    if !ok {
        let _ = std::fs::remove_file(&path);
        return Err(format!("UPDATE_FAILED:{}", errors.join(" | ")));
    }
    verify_download(&path, sha256)?;
    if let Ok(m) = std::fs::metadata(&path) {
        progress(m.len(), Some(m.len()));
    }
    Ok(path)
}

fn fetch_with_ureq<F: FnMut(u64, Option<u64>)>(
    current: &str,
    url: &str,
    path: &PathBuf,
    tls_config: Option<ureq::tls::TlsConfig>,
    progress: &mut F,
) -> Result<(), String> {
    let mut cfg = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(15)))
        .timeout_per_read(Some(Duration::from_secs(30)))
        .user_agent(format!("UsbLens/{current}"));
    if let Some(t) = tls_config {
        cfg = cfg.tls_config(t);
    }
    let agent: ureq::Agent = cfg.build().into();
    let mut resp = agent.get(url).call().map_err(|e| e.to_string())?;
    let total = resp.body().content_length();
    let mut reader = resp.body_mut().with_config().limit(300 * 1024 * 1024).reader();

    let mut file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; 64 * 1024];
    let mut done: u64 = 0;
    let mut last_report = 0u64;
    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        done += n as u64;
        if done - last_report >= 256 * 1024 {
            last_report = done;
            progress(done, total);
        }
    }
    file.flush().map_err(|e| e.to_string())?;
    if let Some(t) = total {
        if done != t {
            return Err(format!("incomplete download ({done}/{t} bytes)"));
        }
    }
    Ok(())
}

/// Last resort: Windows' own HTTP stack (Schannel, system proxy, enterprise certificates).
fn fetch_with_windows(url: &str, path: &std::path::Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        // The URL was validated by is_installer_url; still refuse anything that could break quoting.
        let target = path.to_string_lossy();
        if url.contains(['\'', '"', '`']) || target.contains(['\'', '"', '`']) {
            return Err("unsafe characters".into());
        }
        let script = format!(
            "$ErrorActionPreference='Stop'; $ProgressPreference='SilentlyContinue'; \
             [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
             Invoke-WebRequest -UseBasicParsing -Uri '{url}' -OutFile '{target}'"
        );
        let out = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(())
        } else {
            let msg = String::from_utf8_lossy(&out.stderr);
            Err(msg.lines().find(|l| !l.trim().is_empty()).unwrap_or("powershell failed").trim().to_string())
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (url, path);
        Err("not available on this OS".into())
    }
}

fn verify_download(path: &PathBuf, sha256: Option<&str>) -> Result<(), String> {
    let fail = |msg: &str| {
        let _ = std::fs::remove_file(path);
        Err(msg.to_string())
    };
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() < 1024 || &bytes[..2] != b"MZ" {
        return fail("UPDATE_BAD_FILE");
    }
    if let Some(expected) = sha256 {
        let got: String = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect();
        if !got.eq_ignore_ascii_case(expected) {
            return fail("UPDATE_HASH_MISMATCH");
        }
    }
    Ok(())
}

/// Start the downloaded installer in passive mode: progress bar only, closes the running app,
/// keeps settings and shortcuts (/UPDATE) and starts UsbLens again when done (/R).
pub fn launch_installer(path: &str) -> Result<(), String> {
    let p = PathBuf::from(path);
    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    let in_temp = p.parent().map(|d| d == std::env::temp_dir().as_path()).unwrap_or(false);
    if !in_temp || !name.starts_with(UPDATE_FILE_PREFIX) || !p.is_file() {
        return Err("UPDATE_BAD_FILE".into());
    }
    #[cfg(windows)]
    {
        std::process::Command::new(&p)
            .args(["/P", "/R", "/UPDATE"])
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Err("WINDOWS_ONLY".into())
    }
}

pub fn check(current: &str) -> Result<UpdateInfo, String> {
    let platform = agent(current);
    let builtin: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .user_agent(format!("UsbLens/{current}"))
        .build()
        .into();
    let mut errors = Vec::new();
    for (label, a) in [("github", &platform), ("github/builtin-tls", &builtin)] {
        match from_github(a, current) {
            Ok(info) => return Ok(info),
            Err(e) => errors.push(format!("{label}: {e}")),
        }
    }
    for (label, a) in [("site", &platform), ("site/builtin-tls", &builtin)] {
        match from_manifest(a, current) {
            Ok(info) => return Ok(info),
            Err(e) => errors.push(format!("{label}: {e}")),
        }
    }
    Err(format!("UPDATE_FAILED:{}", errors.join(" | ")))
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
    fn only_repo_installers_are_downloadable() {
        assert!(is_installer_url("https://github.com/yunuskayax/USBLens/releases/download/v0.2.5/UsbLens-setup.exe"));
        assert!(is_installer_url("https://github.com/yunuskayax/USBLens/releases/latest/download/UsbLens-setup.exe"));
        assert!(!is_installer_url("https://github.com/someone/else/releases/download/v1/x.exe"));
        assert!(!is_installer_url("http://github.com/yunuskayax/USBLens/releases/download/v1/x.exe"));
        assert!(!is_installer_url("https://github.com/yunuskayax/USBLens/releases/download/v1/x.msi"));
        assert!(launch_installer("/etc/passwd").is_err());
    }

    #[test]
    fn filters_urls() {
        assert!(safe_url(Some("https://usblens.com/x.exe".into())).is_some());
        assert!(safe_url(Some("https://usblens.com/".into())).is_some());
        assert!(safe_url(Some("http://usblens.com/x.exe".into())).is_none());
        assert!(safe_url(Some("https://yunuskaya.com.tr/x.exe".into())).is_none());
        assert!(safe_url(Some("https://evil.example/x.exe".into())).is_none());
    }
}

#[cfg(test)]
mod net_tests {
    /// `USBLENS_NET_URL=... USBLENS_NET_SHA=... cargo test net_download -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn net_download() {
        let url = std::env::var("USBLENS_NET_URL").unwrap();
        let sha = std::env::var("USBLENS_NET_SHA").ok();
        let mut calls = 0;
        let p = super::download("0.0.0", &url, sha.as_deref(), |d, t| { calls += 1; if calls % 4 == 0 { println!("{d}/{t:?}"); } }).unwrap();
        println!("ok {} {} bytes", p.display(), std::fs::metadata(&p).unwrap().len());
        assert!(super::download("0.0.0", &url, Some("00"), |_, _| {}).unwrap_err().contains("HASH"));
    }

    /// `cargo test net_check -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn net_check() {
        let info = super::check("0.2.2").unwrap();
        println!("{info:#?}");
        assert!(info.available);
        assert!(info.installer.is_some() && info.sha256.is_some());
    }
}
