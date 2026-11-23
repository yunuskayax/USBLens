//! USB security analysis: BadUSB-style heuristics and the allowlist policy.
//!
//! Everything here works on the device list, so it is platform independent and testable.
//! Reason codes are translated in the UI (and in `reason_text` for OS notifications).

use std::collections::{HashMap, HashSet};

use crate::model::{DeviceKind, RiskLevel, Security, UsbDevice};
use crate::settings::{AllowEntry, Settings};

/// Vendor IDs of programmable / hobbyist boards that are often used to build
/// keystroke-injection ("BadUSB") devices. Being on this list is not malicious by itself.
pub const PROGRAMMABLE_VIDS: &[(&str, &str)] = &[
    ("2341", "Arduino"),
    ("2A03", "Arduino"),
    ("1B4F", "SparkFun"),
    ("16C0", "Teensy / V-USB"),
    ("16D0", "Digispark / MCS"),
    ("2E8A", "Raspberry Pi Pico"),
    ("239A", "Adafruit"),
    ("1209", "pid.codes (open hardware)"),
];

pub fn programmable_board(vid: Option<&str>) -> Option<&'static str> {
    let vid = vid?.to_ascii_uppercase();
    PROGRAMMABLE_VIDS.iter().find(|(v, _)| *v == vid).map(|(_, n)| *n)
}

fn upper(s: &str) -> String {
    s.to_ascii_uppercase()
}

/// Interfaces (`...&MI_00\...`) and storage children (`USBSTOR\...`, `SCSI\...`) belong to
/// their parent USB device; the root is the physical device.
fn root_of(id: &str, parents: &HashMap<String, String>) -> String {
    let mut cur = upper(id);
    for _ in 0..4 {
        let child_like = cur.contains("&MI_") || cur.starts_with("USBSTOR\\") || cur.starts_with("SCSI\\");
        if !child_like {
            break;
        }
        match parents.get(&cur) {
            Some(p) if p.starts_with("USB\\") => cur = p.clone(),
            _ => break,
        }
    }
    cur
}

pub fn allow_matches(e: &AllowEntry, d: &UsbDevice) -> bool {
    let (Some(v), Some(p)) = (&d.vendor_id, &d.product_id) else { return false };
    if !e.vendor_id.eq_ignore_ascii_case(v) || !e.product_id.eq_ignore_ascii_case(p) {
        return false;
    }
    match (&e.serial_number, &d.serial_number) {
        (None, _) => true,
        (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
        (Some(_), None) => false,
    }
}

/// Seconds between an RFC 3339 timestamp and now (None if unparseable).
fn age_secs(ts: Option<&str>, now: chrono::DateTime<chrono::Utc>) -> Option<i64> {
    let t = chrono::DateTime::parse_from_rfc3339(ts?).ok()?;
    Some((now - t.with_timezone(&chrono::Utc)).num_seconds())
}

/// A keyboard first installed this recently counts as "new".
pub const NEW_KEYBOARD_WINDOW_SECS: i64 = 24 * 3600;

/// Fill `device.security` for every device.
/// `keyboard_parents` are the parent ids of HID keyboards (`platform::keyboard_parent_ids`).
pub fn annotate(
    devices: &mut [UsbDevice],
    keyboard_parents: &HashSet<String>,
    settings: &Settings,
    now: chrono::DateTime<chrono::Utc>,
) {
    let parents: HashMap<String, String> = devices
        .iter()
        .filter_map(|d| d.parent_id.as_ref().map(|p| (upper(&d.instance_id), upper(p))))
        .collect();
    let roots: Vec<String> = devices.iter().map(|d| root_of(&d.instance_id, &parents)).collect();

    // Aggregate capabilities per physical device.
    let mut keyboard_roots = HashSet::new();
    let mut storage_roots = HashSet::new();
    for (d, root) in devices.iter().zip(&roots) {
        let id = upper(&d.instance_id);
        if keyboard_parents.contains(&id) {
            keyboard_roots.insert(root.clone());
        }
        if d.kind == DeviceKind::Storage {
            storage_roots.insert(root.clone());
        }
    }
    let by_id: HashMap<String, usize> = devices.iter().enumerate().map(|(i, d)| (upper(&d.instance_id), i)).collect();

    // Evaluate each physical device once, using its root entry's identity.
    let mut root_security: HashMap<String, Security> = HashMap::new();
    for root in roots.iter().collect::<HashSet<_>>() {
        let Some(&ri) = by_id.get(root) else { continue };
        let d = &devices[ri];
        let keyboard = keyboard_roots.contains(root);
        let storage = storage_roots.contains(root);
        let mut reasons = Vec::new();
        let mut level = RiskLevel::None;
        let mut raise = |l: RiskLevel, r: String, reasons: &mut Vec<String>| {
            level = level.max(l);
            reasons.push(r);
        };

        if settings.security_alerts {
            if keyboard && storage {
                raise(RiskLevel::High, "KEYBOARD_AND_STORAGE".into(), &mut reasons);
            }
            if let Some(board) = programmable_board(d.vendor_id.as_deref()) {
                let l = if keyboard { RiskLevel::High } else { RiskLevel::Info };
                raise(l, format!("PROGRAMMABLE_BOARD:{board}"), &mut reasons);
            }
        }
        if settings.new_keyboard_alert && keyboard {
            if let Some(age) = age_secs(d.first_install.as_deref(), now) {
                if (0..=NEW_KEYBOARD_WINDOW_SECS).contains(&age) {
                    raise(RiskLevel::Medium, "NEW_KEYBOARD".into(), &mut reasons);
                }
            }
        }

        let in_scope = d.kind != DeviceKind::Hub
            && root.starts_with("USB\\")
            && !root.starts_with("USB\\ROOT_HUB")
            && (settings.allowlist_scope == "all" || storage);
        let allowed = if settings.allowlist_enabled && in_scope {
            let ok = settings.allowlist.iter().any(|e| allow_matches(e, d));
            if !ok {
                raise(RiskLevel::High, "NOT_ALLOWED".into(), &mut reasons);
            }
            Some(ok)
        } else {
            None
        };

        root_security.insert(root.clone(), Security { root_id: d.instance_id.clone(), keyboard, storage, level, reasons, allowed });
    }

    for (d, root) in devices.iter_mut().zip(&roots) {
        d.security = root_security.get(root).cloned().unwrap_or_else(|| Security {
            root_id: d.instance_id.clone(),
            ..Default::default()
        });
    }
}

/// Short human text for OS notifications (the UI has its own translations).
pub fn reason_text(code: &str, lang: &str) -> String {
    let tr = lang == "tr";
    let (head, arg) = code.split_once(':').unwrap_or((code, ""));
    match head {
        "KEYBOARD_AND_STORAGE" if tr => "Hem klavye hem depolama gibi davranıyor (BadUSB olabilir)".into(),
        "KEYBOARD_AND_STORAGE" => "Acts as both a keyboard and storage (possible BadUSB)".into(),
        "PROGRAMMABLE_BOARD" if tr => format!("Programlanabilir kart: {arg}"),
        "PROGRAMMABLE_BOARD" => format!("Programmable board: {arg}"),
        "NEW_KEYBOARD" if tr => "İlk kez takılan bir klavye".into(),
        "NEW_KEYBOARD" => "A keyboard connected for the first time".into(),
        "NOT_ALLOWED" if tr => "Beyaz listede olmayan cihaz".into(),
        "NOT_ALLOWED" => "Device is not on the allowlist".into(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::parse_instance_id;

    fn dev(id: &str, kind: DeviceKind, parent: Option<&str>, first: Option<&str>) -> UsbDevice {
        let (vendor_id, product_id, serial_number) = parse_instance_id(id);
        UsbDevice {
            instance_id: id.into(),
            name: id.into(),
            description: String::new(),
            kind,
            class_name: String::new(),
            manufacturer: String::new(),
            service: String::new(),
            vendor_id,
            product_id,
            serial_number,
            connected: true,
            disabled: false,
            problem_code: None,
            drive_letters: vec![],
            capacity_bytes: None,
            removable: None,
            vendor_name: None,
            product_name: None,
            location: None,
            parent_id: parent.map(Into::into),
            driver_version: None,
            first_install: first.map(Into::into),
            last_arrival: None,
            last_removal: None,
            security: Default::default(),
            children: vec![],
        }
    }

    fn now() -> chrono::DateTime<chrono::Utc> {
        "2026-11-17T12:00:00Z".parse().unwrap()
    }

    #[test]
    fn composite_keyboard_plus_storage_is_high_risk() {
        // Composite parent with a HID keyboard interface and a mass-storage interface.
        let mut list = vec![
            dev("USB\\VID_1234&PID_5678\\ABC", DeviceKind::Other, None, Some("2025-01-01T00:00:00Z")),
            dev("USB\\VID_1234&PID_5678&MI_00\\7&1&0&0", DeviceKind::Hid, Some("USB\\VID_1234&PID_5678\\ABC"), None),
            dev("USB\\VID_1234&PID_5678&MI_01\\7&1&0&1", DeviceKind::Storage, Some("USB\\VID_1234&PID_5678\\ABC"), None),
            dev("USBSTOR\\DISK&VEN_X&PROD_Y\\7&1&0&1&0", DeviceKind::Storage, Some("USB\\VID_1234&PID_5678&MI_01\\7&1&0&1"), None),
        ];
        let kb: HashSet<String> = ["USB\\VID_1234&PID_5678&MI_00\\7&1&0&0".to_string()].into();
        annotate(&mut list, &kb, &Settings::default(), now());
        for d in &list {
            assert_eq!(d.security.level, RiskLevel::High, "{}", d.instance_id);
            assert!(d.security.reasons.contains(&"KEYBOARD_AND_STORAGE".to_string()));
            assert_eq!(d.security.root_id, "USB\\VID_1234&PID_5678\\ABC");
        }
    }

    #[test]
    fn programmable_keyboard_is_high_and_plain_board_is_info() {
        let mut list = vec![
            dev("USB\\VID_16D0&PID_0753\\5&1", DeviceKind::Hid, None, Some("2020-01-01T00:00:00Z")),
            dev("USB\\VID_2341&PID_0043\\95", DeviceKind::Serial, None, Some("2020-01-01T00:00:00Z")),
        ];
        let kb: HashSet<String> = ["USB\\VID_16D0&PID_0753\\5&1".to_string()].into();
        annotate(&mut list, &kb, &Settings::default(), now());
        assert_eq!(list[0].security.level, RiskLevel::High);
        assert_eq!(list[1].security.level, RiskLevel::Info);
        assert!(list[1].security.reasons[0].starts_with("PROGRAMMABLE_BOARD:Arduino"));
    }

    #[test]
    fn new_keyboard_is_medium_only_when_recent() {
        let mut list = vec![
            dev("USB\\VID_046D&PID_C31C\\A", DeviceKind::Hid, None, Some("2026-11-17T11:59:00Z")),
            dev("USB\\VID_046D&PID_C31D\\B", DeviceKind::Hid, None, Some("2026-10-01T00:00:00Z")),
        ];
        let kb: HashSet<String> = list.iter().map(|d| d.instance_id.to_ascii_uppercase()).collect();
        annotate(&mut list, &kb, &Settings::default(), now());
        assert_eq!(list[0].security.level, RiskLevel::Medium);
        assert_eq!(list[1].security.level, RiskLevel::None);

        let mut off = Settings::default();
        off.new_keyboard_alert = false;
        annotate(&mut list, &kb, &off, now());
        assert_eq!(list[0].security.level, RiskLevel::None);
    }

    #[test]
    fn allowlist_scope_and_matching() {
        let mut list = vec![
            dev("USB\\VID_0781&PID_5567\\SERIAL1", DeviceKind::Storage, None, None),
            dev("USB\\VID_0951&PID_1666\\OTHER", DeviceKind::Storage, None, None),
            dev("USB\\VID_046D&PID_C52B\\6&1A&0&2", DeviceKind::Hid, None, None),
        ];
        let mut s = Settings { allowlist_enabled: true, ..Default::default() };
        s.allowlist.push(AllowEntry {
            vendor_id: "0781".into(),
            product_id: "5567".into(),
            serial_number: Some("serial1".into()),
            ..Default::default()
        });
        annotate(&mut list, &HashSet::new(), &s, now());
        assert_eq!(list[0].security.allowed, Some(true));
        assert_eq!(list[1].security.allowed, Some(false));
        assert!(list[1].security.reasons.contains(&"NOT_ALLOWED".to_string()));
        // Storage scope: the receiver is not checked.
        assert_eq!(list[2].security.allowed, None);

        s.allowlist_scope = "all".into();
        annotate(&mut list, &HashSet::new(), &s, now());
        assert_eq!(list[2].security.allowed, Some(false));

        // A serial-less entry matches every unit of that model.
        s.allowlist[0].serial_number = None;
        list[0].serial_number = Some("ANOTHER".into());
        annotate(&mut list, &HashSet::new(), &s, now());
        assert_eq!(list[0].security.allowed, Some(true));
    }

    #[test]
    fn groups_interfaces_and_disk_into_one_row() {
        let mut list = vec![
            dev("USB\\VID_2009&PID_7100\\2009710066015468AEA4", DeviceKind::Storage, None, None),
            dev("USBSTOR\\DISK&VEN_KINGSTON&PROD_KEYPAD200\\2009710066015468AEA4&0", DeviceKind::Storage, Some("USB\\VID_2009&PID_7100\\2009710066015468AEA4"), None),
            dev("USB\\VID_1234&PID_5678\\ABC", DeviceKind::Other, None, None),
            dev("USB\\VID_1234&PID_5678&MI_00\\7&1&0&0", DeviceKind::Hid, Some("USB\\VID_1234&PID_5678\\ABC"), None),
            dev("USB\\VID_1234&PID_5678&MI_01\\7&1&0&1", DeviceKind::Audio, Some("USB\\VID_1234&PID_5678\\ABC"), None),
        ];
        list[0].name = "Keypad200".into();
        list[0].description = "USB Mass Storage Device".into();
        list[1].name = "Kingston Keypad200 USB Device".into();
        list[1].drive_letters = vec!["H:".into()];
        annotate(&mut list, &HashSet::new(), &Settings::default(), now());
        let grouped = crate::model::group_children(list);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].name, "Keypad200");
        assert_eq!(grouped[0].description, "Kingston Keypad200 USB Device");
        assert_eq!(grouped[0].drive_letters, vec!["H:".to_string()]);
        assert_eq!(grouped[0].children.len(), 1);
        assert_eq!(grouped[0].vendor_name.as_deref(), Some("KINGSTON"));
        assert_eq!(grouped[0].product_name.as_deref(), Some("KEYPAD200"));
        // Composite parent takes the most telling child type (audio over HID).
        assert_eq!(grouped[1].kind, DeviceKind::Audio);
        assert_eq!(grouped[1].children.len(), 2);
    }

    #[test]
    fn hubs_are_never_blocked() {
        let mut list = vec![dev("USB\\ROOT_HUB30\\4&2B&0&0", DeviceKind::Hub, None, None)];
        let s = Settings { allowlist_enabled: true, allowlist_scope: "all".into(), ..Default::default() };
        annotate(&mut list, &HashSet::new(), &s, now());
        assert_eq!(list[0].security.allowed, None);
    }
}
