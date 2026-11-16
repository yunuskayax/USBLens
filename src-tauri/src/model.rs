#![cfg_attr(not(any(windows, usblens_check)), allow(dead_code))]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DeviceKind {
    Storage,
    Hid,
    Audio,
    Video,
    Phone,
    Printer,
    Network,
    Bluetooth,
    Serial,
    Hub,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbDevice {
    /// PnP instance id, e.g. `USB\VID_0781&PID_5567\4C530001234567`
    pub instance_id: String,
    /// Best human readable name (bus reported name > friendly name > description)
    pub name: String,
    pub description: String,
    pub kind: DeviceKind,
    pub class_name: String,
    pub manufacturer: String,
    pub service: String,
    pub vendor_id: Option<String>,
    pub product_id: Option<String>,
    pub serial_number: Option<String>,
    /// Physically plugged in right now
    pub connected: bool,
    /// Disabled by the user (problem code 22)
    pub disabled: bool,
    pub problem_code: Option<u32>,
    pub drive_letters: Vec<String>,
    pub location: Option<String>,
    pub parent_id: Option<String>,
    pub driver_version: Option<String>,
    /// RFC 3339 timestamps (UTC)
    pub first_install: Option<String>,
    pub last_arrival: Option<String>,
    pub last_removal: Option<String>,
    /// Filled by `security::annotate`.
    #[serde(default)]
    pub security: Security,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "camelCase")]
pub enum RiskLevel {
    #[default]
    None,
    Info,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Security {
    /// Instance id of the physical device (composite parent) this entry belongs to.
    pub root_id: String,
    /// The physical device exposes a keyboard (HID keyboard interface).
    pub keyboard: bool,
    /// The physical device exposes mass storage.
    pub storage: bool,
    pub level: RiskLevel,
    /// Machine-readable reasons, e.g. "KEYBOARD_AND_STORAGE", "PROGRAMMABLE_BOARD:Arduino".
    pub reasons: Vec<String>,
    /// None = allowlist off / device out of scope.
    pub allowed: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub is_windows: bool,
    pub elevated: bool,
    pub computer_name: String,
    pub user_name: String,
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbEvent {
    /// "arrival" | "removal"
    pub action: String,
    pub instance_id: Option<String>,
    pub device: Option<UsbDevice>,
    /// Security reasons raised for this event (see `security`).
    #[serde(default)]
    pub alerts: Vec<String>,
    #[serde(default)]
    pub level: RiskLevel,
    /// The device was disabled by the allowlist policy.
    #[serde(default)]
    pub blocked: bool,
    pub block_error: Option<String>,
}

/// Convert a Windows FILETIME (100 ns ticks since 1601-01-01 UTC) to RFC 3339.
pub fn filetime_to_rfc3339(ticks: u64) -> Option<String> {
    const EPOCH_DIFF: u64 = 116_444_736_000_000_000;
    if ticks <= EPOCH_DIFF {
        return None;
    }
    let since_unix = ticks - EPOCH_DIFF;
    let secs = (since_unix / 10_000_000) as i64;
    let nanos = ((since_unix % 10_000_000) * 100) as u32;
    chrono::DateTime::<chrono::Utc>::from_timestamp(secs, nanos)
        .map(|d| d.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}

/// `\\?\USB#VID_0781&PID_5567#4C53#{a5dcbf10-...}` -> `USB\VID_0781&PID_5567\4C53`
pub fn symlink_to_instance_id(link: &str) -> Option<String> {
    let s = link
        .strip_prefix("\\\\?\\")
        .or_else(|| link.strip_prefix("\\??\\"))
        .unwrap_or(link);
    let (body, _guid) = s.rsplit_once("#{")?;
    Some(body.replace('#', "\\"))
}

/// Parse VID / PID / serial from a PnP instance id.
pub fn parse_instance_id(id: &str) -> (Option<String>, Option<String>, Option<String>) {
    let parts: Vec<&str> = id.split('\\').collect();
    let upper = id.to_ascii_uppercase();
    let mut vid = None;
    let mut pid = None;

    if let Some(hw) = parts.get(1) {
        for token in hw.split('&') {
            let t = token.to_ascii_uppercase();
            if let Some(v) = t.strip_prefix("VID_") {
                vid = Some(v.chars().take(4).collect());
            } else if let Some(p) = t.strip_prefix("PID_") {
                pid = Some(p.chars().take(4).collect());
            }
        }
    }

    let serial = parts.get(2).and_then(|s| {
        if upper.starts_with("USBSTOR\\") || upper.starts_with("SCSI\\") {
            // USBSTOR\Disk&Ven_X&Prod_Y\SERIAL&0 -> SERIAL (when it is not generated)
            let core = s.rsplit_once('&').map(|(a, _)| a).unwrap_or(s);
            if core.contains('&') || core.is_empty() {
                None
            } else {
                Some(core.to_string())
            }
        } else if s.contains('&') {
            // Windows generated an id because the device reports no serial
            None
        } else {
            Some(s.to_string())
        }
    });

    (vid, pid, serial)
}

/// Decide what kind of device this is from its class/service/name.
pub fn classify(class_name: &str, service: &str, name: &str, instance_id: &str) -> DeviceKind {
    let c = class_name.to_ascii_lowercase();
    let s = service.to_ascii_lowercase();
    let n = name.to_ascii_lowercase();
    let id = instance_id.to_ascii_uppercase();

    if id.starts_with("USBSTOR\\")
        || s == "usbstor"
        || s == "uaspstor"
        || c == "diskdrive"
        || c == "cdrom"
    {
        return DeviceKind::Storage;
    }
    if c == "wpd" || s == "wudfwpdmtp" || (s == "winusb" && n.contains("phone")) || n.contains("mtp") {
        return DeviceKind::Phone;
    }
    if c == "hidclass" || c == "keyboard" || c == "mouse" || s == "hidusb" {
        return DeviceKind::Hid;
    }
    if c == "media" || c == "audioendpoint" || s == "usbaudio" || s == "usbaudio2" {
        return DeviceKind::Audio;
    }
    if c == "camera" || c == "image" || s == "usbvideo" {
        return DeviceKind::Video;
    }
    if c == "printer" || c == "printqueue" || s == "usbprint" {
        return DeviceKind::Printer;
    }
    if c == "net" {
        return DeviceKind::Network;
    }
    if c == "bluetooth" || s == "bthusb" {
        return DeviceKind::Bluetooth;
    }
    if c == "ports" || s == "usbser" {
        return DeviceKind::Serial;
    }
    if n.contains("hub") || s.starts_with("usbhub") || id.starts_with("USB\\ROOT_HUB") {
        return DeviceKind::Hub;
    }
    DeviceKind::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_usb_ids() {
        let (v, p, s) = parse_instance_id("USB\\VID_0781&PID_5567\\4C530001234567");
        assert_eq!(v.as_deref(), Some("0781"));
        assert_eq!(p.as_deref(), Some("5567"));
        assert_eq!(s.as_deref(), Some("4C530001234567"));

        let (_, _, s) = parse_instance_id("USB\\VID_046D&PID_C52B\\6&1A2B3C4D&0&2");
        assert_eq!(s, None);

        let (v, _, s) =
            parse_instance_id("USBSTOR\\Disk&Ven_SanDisk&Prod_Cruzer&Rev_1.00\\4C530001234567&0");
        assert_eq!(v, None);
        assert_eq!(s.as_deref(), Some("4C530001234567"));
    }

    #[test]
    fn converts_filetime_and_symlinks() {
        // 2024-01-01T00:00:00Z
        assert_eq!(
            filetime_to_rfc3339(133_485_408_000_000_000).as_deref(),
            Some("2024-01-01T00:00:00Z")
        );
        assert_eq!(filetime_to_rfc3339(0), None);
        assert_eq!(
            symlink_to_instance_id(
                "\\\\?\\USB#VID_0781&PID_5567#4C530001234567#{a5dcbf10-6530-11d2-901f-00c04fb951ed}"
            )
            .as_deref(),
            Some("USB\\VID_0781&PID_5567\\4C530001234567")
        );
    }

    #[test]
    fn classifies() {
        assert_eq!(classify("USB", "USBSTOR", "Mass Storage", "USB\\VID_1"), DeviceKind::Storage);
        assert_eq!(classify("HIDClass", "HidUsb", "Keyboard", "USB\\VID_1"), DeviceKind::Hid);
        assert_eq!(classify("USB", "USBHUB3", "Generic Hub", "USB\\VID_1"), DeviceKind::Hub);
        assert_eq!(classify("WPD", "WUDFWpdMtp", "Galaxy", "USB\\VID_1"), DeviceKind::Phone);
    }
}
