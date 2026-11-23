//! Rust port of `src/label.ts`, used by the Explorer right-click menu
//! ("UsbLens ▸ Etiketi kopyala") which runs without the web UI.
//! Keep the two in sync; the tests below mirror the TypeScript outputs.

use crate::model::{DeviceKind, UsbDevice};

const NOMINAL_GB: &[f64] = &[
    1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 60.0, 64.0, 120.0, 128.0, 240.0, 250.0, 256.0, 320.0, 480.0, 500.0, 512.0, 640.0,
    750.0, 960.0, 1000.0, 1500.0, 2000.0, 3000.0, 4000.0, 5000.0, 6000.0, 8000.0, 10000.0, 12000.0, 14000.0, 16000.0,
    18000.0, 20000.0, 22000.0, 24000.0,
];

fn is_tr(lang: &str) -> bool {
    lang != "en"
}

/// 57.3 -> "57,3" (tr) / "57.3" (en); 64.0 -> "64".
fn num(x: f64, lang: &str) -> String {
    let r = (x * 10.0).round() / 10.0;
    let s = if r.fract() == 0.0 { format!("{}", r as i64) } else { format!("{r:.1}") };
    if is_tr(lang) {
        s.replace('.', ",")
    } else {
        s
    }
}

fn gb_text(gb: f64, lang: &str) -> String {
    if gb >= 1000.0 {
        format!("{} TB", num(gb / 1000.0, lang))
    } else {
        format!("{} GB", num(gb, lang))
    }
}

/// 61 530 000 000 bytes -> "64 GB" (the size on the box).
pub fn nominal_capacity(bytes: Option<u64>, lang: &str) -> Option<String> {
    let bytes = bytes.filter(|b| *b > 0)?;
    let gb = bytes as f64 / 1e9;
    let mut best: Option<f64> = None;
    for &n in NOMINAL_GB {
        let ratio = gb / n;
        if !(0.9..=1.005).contains(&ratio) {
            continue;
        }
        if best.map(|b| (1.0 - ratio).abs() < (1.0 - gb / b).abs()).unwrap_or(true) {
            best = Some(n);
        }
    }
    Some(match best {
        Some(n) => gb_text(n, lang),
        None => gb_text((gb * 10.0).round() / 10.0, lang),
    })
}

/// Usable size as Windows shows it (binary units): "57,3 GB".
pub fn usable_capacity(bytes: Option<u64>, lang: &str) -> Option<String> {
    let bytes = bytes.filter(|b| *b > 0)?;
    let gib = bytes as f64 / 1024f64.powi(3);
    Some(if gib >= 1024.0 {
        format!("{} TB", num(gib / 1024.0, lang))
    } else {
        format!("{} GB", num(gib, lang))
    })
}

const VENDOR_BY_VID: &[(&str, &str)] = &[
    ("0781", "SanDisk"),
    ("0951", "Kingston"),
    ("8564", "Transcend"),
    ("054C", "Sony"),
    ("0930", "Toshiba"),
    ("18A5", "Verbatim"),
    ("125F", "ADATA"),
    ("0BC2", "Seagate"),
    ("1058", "WD"),
    ("04E8", "Samsung"),
    ("05DC", "Lexar"),
    ("154B", "PNY"),
    ("0EA0", "Ours"),
    ("1005", "Apacer"),
    ("0718", "Imation"),
    ("13FE", "Kingston"),
    ("0480", "Toshiba"),
    ("059F", "LaCie"),
    ("04B4", "Cypress"),
    ("0411", "Buffalo"),
    ("1B1C", "Corsair"),
    ("2009", "iStorage"),
];

const PRETTY: &[(&str, &str)] = &[
    ("SANDISK", "SanDisk"),
    ("KINGSTON", "Kingston"),
    ("TOSHIBA", "Toshiba"),
    ("SAMSUNG", "Samsung"),
    ("TRANSCEND", "Transcend"),
    ("VERBATIM", "Verbatim"),
    ("LEXAR", "Lexar"),
    ("SEAGATE", "Seagate"),
    ("SONY", "Sony"),
    ("APACER", "Apacer"),
    ("CORSAIR", "Corsair"),
    ("INTEGRAL", "Integral"),
    ("PHILIPS", "Philips"),
    ("HIKVISION", "Hikvision"),
    ("NETAC", "Netac"),
    ("TEAM", "Team"),
    ("LACIE", "LaCie"),
    ("WESTERN DIGITAL", "WD"),
    ("WDC", "WD"),
    ("PATRIOT", "Patriot"),
    ("EMTEC", "Emtec"),
    ("INTENSO", "Intenso"),
    ("ISTORAGE", "iStorage"),
];

/// Names that only say "some USB disk" (same list as the TS regex).
fn is_generic(s: &str) -> bool {
    let l = s.trim().to_lowercase();
    const WORDS: &[&str] = &[
        "generic",
        "usb",
        "mass",
        "storage",
        "flash",
        "disk",
        "usb disk",
        "usb flash",
        "usb flash disk",
        "usb mass storage",
        "general",
    ];
    WORDS.contains(&l.as_str())
        || (l.starts_with("ut") && l[2..].chars().all(|c| c.is_ascii_digit()))
        || (l.starts_with("(standard") && l.ends_with(')'))
}

fn pretty(s: &str) -> String {
    let t = s.trim();
    let up = t.to_uppercase();
    PRETTY.iter().find(|(k, _)| *k == up).map(|(_, v)| v.to_string()).unwrap_or_else(|| t.to_string())
}

pub fn brand_of(d: &UsbDevice) -> Option<String> {
    if let Some(v) = d.vendor_name.as_deref().map(str::trim).filter(|v| !v.is_empty() && !is_generic(v)) {
        return Some(pretty(v));
    }
    if let Some(vid) = d.vendor_id.as_deref() {
        let vid = vid.to_uppercase();
        if let Some((_, name)) = VENDOR_BY_VID.iter().find(|(k, _)| *k == vid) {
            return Some(name.to_string());
        }
    }
    let source = if d.description.is_empty() { &d.name } else { &d.description };
    let first = source.split_whitespace().next().unwrap_or("");
    if first.chars().count() > 2 && !is_generic(first) && first.chars().any(|c| c.is_ascii_alphabetic()) {
        let lower = first.to_lowercase();
        return VENDOR_BY_VID
            .iter()
            .map(|(_, v)| *v)
            .chain(PRETTY.iter().map(|(_, v)| *v))
            .find(|k| k.to_lowercase() == lower)
            .map(str::to_string);
    }
    None
}

pub fn model_of(d: &UsbDevice) -> Option<String> {
    d.product_name.as_deref().map(str::trim).filter(|p| !p.is_empty() && !is_generic(p)).map(str::to_string)
}

fn kind_label(kind: DeviceKind, lang: &str) -> &'static str {
    let tr = is_tr(lang);
    match kind {
        DeviceKind::Storage => if tr { "Depolama" } else { "Storage" },
        DeviceKind::Hid => if tr { "Giriş (HID)" } else { "Input (HID)" },
        DeviceKind::Audio => if tr { "Ses" } else { "Audio" },
        DeviceKind::Video => if tr { "Kamera" } else { "Camera" },
        DeviceKind::Phone => if tr { "Telefon" } else { "Phone" },
        DeviceKind::Printer => if tr { "Yazıcı" } else { "Printer" },
        DeviceKind::Network => if tr { "Ağ" } else { "Network" },
        DeviceKind::Bluetooth => "Bluetooth",
        DeviceKind::Serial => if tr { "Seri port" } else { "Serial port" },
        DeviceKind::Hub => "Hub",
        DeviceKind::Other => if tr { "Diğer" } else { "Other" },
    }
}

pub fn type_text(d: &UsbDevice, lang: &str) -> String {
    if d.kind != DeviceKind::Storage {
        return kind_label(d.kind, lang).to_string();
    }
    let tr = is_tr(lang);
    if d.removable == Some(false) {
        if tr { "Harici Disk" } else { "External Drive" }.into()
    } else {
        if tr { "USB Bellek" } else { "USB Flash Drive" }.into()
    }
}

fn token_value(d: &UsbDevice, lang: &str, key: &str) -> Option<String> {
    Some(match key {
        "marka" | "brand" => brand_of(d).unwrap_or_default(),
        "model" => model_of(d).unwrap_or_default(),
        "kapasite" | "capacity" => nominal_capacity(d.capacity_bytes, lang).unwrap_or_default(),
        "tür" | "tur" | "type" => type_text(d, lang),
        "seri" | "serial" => d.serial_number.clone().unwrap_or_default(),
        "sürücü" | "surucu" | "drive" => d.drive_letters.join(" "),
        "vidpid" => match (&d.vendor_id, &d.product_id) {
            (Some(v), Some(p)) => format!("{v}:{p}"),
            _ => String::new(),
        },
        "ad" | "name" => d.name.clone(),
        _ => return None,
    })
}

/// Replaces `{token}`s; unknown tokens stay as written.
fn substitute(s: &str, d: &UsbDevice, lang: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find(['{', '}']) {
            Some(close) if after.as_bytes()[close] == b'}' => {
                let key = &after[..close];
                match token_value(d, lang, &key.trim().to_lowercase()) {
                    Some(v) => out.push_str(&v),
                    None => {
                        out.push('{');
                        out.push_str(key);
                        out.push('}');
                    }
                }
                rest = &after[close + 1..];
            }
            _ => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// A bracketed group counts as empty when it has tokens and all of them are empty.
fn group_has_value(group: &str, d: &UsbDevice, lang: &str) -> bool {
    let mut any_token = false;
    let mut rest = group;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let Some(close) = after.find(['{', '}']) else { break };
        if after.as_bytes()[close] == b'}' {
            any_token = true;
            let v = token_value(d, lang, &after[..close].trim().to_lowercase()).unwrap_or_else(|| "x".into());
            if !v.is_empty() {
                return true;
            }
            rest = &after[close + 1..];
        } else {
            rest = after;
        }
    }
    !any_token
}

/// "{marka} {kapasite} {tür} (S.No: {seri})" -> "Kingston 64 GB USB Bellek (S.No: 2009…)".
pub fn device_label(d: &UsbDevice, template: &str, lang: &str) -> String {
    // Drop "(…)" / "[…]" groups (without nested brackets) whose tokens are all empty.
    let chars: Vec<char> = template.chars().collect();
    let mut kept = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let close = match c {
            '(' => Some(')'),
            '[' => Some(']'),
            _ => None,
        };
        if let Some(close) = close {
            let end = chars[i + 1..]
                .iter()
                .position(|&x| x == close || x == c || (c == '(' && x == '(') || (c == '[' && x == '['));
            if let Some(off) = end {
                let j = i + 1 + off;
                if chars[j] == close {
                    let group: String = chars[i..=j].iter().collect();
                    if group_has_value(&group, d, lang) {
                        kept.push_str(&group);
                    }
                    i = j + 1;
                    continue;
                }
            }
        }
        kept.push(c);
        i += 1;
    }

    let text = substitute(&kept, d, lang);
    // Collapse runs of whitespace, then remove spaces before closing punctuation.
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = String::with_capacity(collapsed.len());
    for ch in collapsed.chars() {
        if matches!(ch, ',' | '.' | ';' | ':' | ')' | ']') && out.ends_with(' ') {
            out.pop();
        }
        out.push(ch);
    }
    out.trim().to_string()
}

/// Label plus every known property as "Name: value" lines ("Tüm bilgileri kopyala").
pub fn details_text(d: &UsbDevice, template: &str, lang: &str) -> String {
    let tr = is_tr(lang);
    let l = |a: &'static str, b: &'static str| if tr { a } else { b };
    let capacity = nominal_capacity(d.capacity_bytes, lang).map(|n| match usable_capacity(d.capacity_bytes, lang) {
        Some(u) if u != n => format!("{n} ({u} {})", l("kullanılabilir", "usable")),
        _ => n,
    });
    let rows: Vec<(&str, Option<String>)> = vec![
        (l("Ad", "Name"), Some(d.name.clone())),
        (l("Tür", "Type"), Some(type_text(d, lang))),
        (l("Marka", "Brand"), brand_of(d)),
        (l("Model", "Model"), model_of(d)),
        (l("Kapasite", "Capacity"), capacity),
        ("VID", d.vendor_id.clone()),
        ("PID", d.product_id.clone()),
        (l("Seri no", "Serial"), d.serial_number.clone()),
        (l("Sürücü", "Drive"), Some(d.drive_letters.join(", "))),
        (l("Açıklama", "Description"), Some(d.description.clone())),
        (l("Üretici", "Manufacturer"), Some(d.manufacturer.clone())),
        (l("Konum", "Location"), d.location.clone()),
        ("Instance ID", Some(d.instance_id.clone())),
    ];
    let body = rows
        .into_iter()
        .filter_map(|(k, v)| v.filter(|v| !v.is_empty()).map(|v| format!("{k}: {v}")))
        .collect::<Vec<_>>()
        .join("\n");
    let label = if d.kind == DeviceKind::Storage { device_label(d, template, lang) } else { String::new() };
    if label.is_empty() {
        body
    } else {
        format!("{label}\n\n{body}")
    }
}

/// Finds the device that owns a drive letter. Accepts "E", "E:", "E:\" or a path on it.
pub fn find_by_drive<'a>(devices: &'a [UsbDevice], arg: &str) -> Option<&'a UsbDevice> {
    let letter = arg.trim().trim_matches('"').chars().next()?.to_ascii_uppercase();
    if !letter.is_ascii_alphabetic() {
        return None;
    }
    let want = format!("{letter}:");
    devices
        .iter()
        .find(|d| d.drive_letters.iter().any(|l| l.to_ascii_uppercase().starts_with(&want)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::DEFAULT_COPY_TEMPLATE;

    fn kingston() -> UsbDevice {
        let mut d = UsbDevice {
            instance_id: "USB\\VID_0951&PID_1666\\60A44C413A8CF320E92D0123".into(),
            name: String::new(),
            description: String::new(),
            kind: DeviceKind::Storage,
            class_name: String::new(),
            manufacturer: String::new(),
            service: String::new(),
            vendor_id: None,
            product_id: None,
            serial_number: None,
            connected: true,
            disabled: false,
            problem_code: None,
            drive_letters: vec![],
            capacity_bytes: None,
            removable: None,
            vendor_name: None,
            product_name: None,
            location: None,
            parent_id: None,
            driver_version: None,
            first_install: None,
            last_arrival: None,
            last_removal: None,
            security: Default::default(),
            children: vec![],
        };
        d.name = "DataTraveler 3.0".into();
        d.description = "USB Yığın Depolama Aygıtı".into();
        d.vendor_id = Some("0951".into());
        d.product_id = Some("1666".into());
        d.serial_number = Some("60A44C413A8CF320E92D0123".into());
        d.capacity_bytes = Some(61_530_000_000);
        d.removable = Some(true);
        d.vendor_name = Some("KINGSTON".into());
        d.product_name = Some("DataTraveler 3.0".into());
        d.drive_letters = vec!["E:".into()];
        d
    }

    #[test]
    fn capacity() {
        assert_eq!(nominal_capacity(Some(61_530_000_000), "tr").unwrap(), "64 GB");
        assert_eq!(nominal_capacity(Some(31_000_000_000), "tr").unwrap(), "32 GB");
        assert_eq!(nominal_capacity(Some(1_000_204_886_016), "en").unwrap(), "1 TB");
        assert_eq!(nominal_capacity(Some(45_000_000_000), "tr").unwrap(), "45 GB");
        assert_eq!(usable_capacity(Some(61_530_000_000), "tr").unwrap(), "57,3 GB");
        assert_eq!(usable_capacity(Some(61_530_000_000), "en").unwrap(), "57.3 GB");
        assert_eq!(nominal_capacity(None, "tr"), None);
    }

    #[test]
    fn label_default_template() {
        let d = kingston();
        assert_eq!(
            device_label(&d, DEFAULT_COPY_TEMPLATE, "tr"),
            "Kingston 64 GB USB Bellek (S.No: 60A44C413A8CF320E92D0123)"
        );
        assert_eq!(
            device_label(&d, DEFAULT_COPY_TEMPLATE, "en"),
            "Kingston 64 GB USB Flash Drive (S.No: 60A44C413A8CF320E92D0123)"
        );
    }

    #[test]
    fn empty_groups_are_dropped() {
        let mut d = kingston();
        d.serial_number = None;
        assert_eq!(device_label(&d, DEFAULT_COPY_TEMPLATE, "tr"), "Kingston 64 GB USB Bellek");
        assert_eq!(device_label(&d, "{marka} [{seri}] - {sürücü}", "tr"), "Kingston - E:");
        assert_eq!(device_label(&d, "{MARKA} {model} {bilinmeyen}", "tr"), "Kingston DataTraveler 3.0 {bilinmeyen}");
    }

    #[test]
    fn brand_fallbacks() {
        let mut d = kingston();
        d.vendor_name = Some("Generic".into());
        assert_eq!(brand_of(&d).as_deref(), Some("Kingston")); // by VID
        d.vendor_id = Some("ABCD".into());
        d.description = "SanDisk Cruzer".into();
        assert_eq!(brand_of(&d).as_deref(), Some("SanDisk")); // by first word
        d.product_name = Some("USB Flash Disk".into());
        assert_eq!(model_of(&d), None);
        d.removable = Some(false);
        assert_eq!(type_text(&d, "tr"), "Harici Disk");
    }

    #[test]
    fn drive_lookup_and_details() {
        let list = vec![kingston()];
        assert!(find_by_drive(&list, "E:\\").is_some());
        assert!(find_by_drive(&list, "\"e:\\\"").is_some());
        assert!(find_by_drive(&list, "F:\\").is_none());
        let text = details_text(&list[0], DEFAULT_COPY_TEMPLATE, "tr");
        assert!(text.starts_with("Kingston 64 GB USB Bellek (S.No: "));
        assert!(text.contains("\nKapasite: 64 GB (57,3 GB kullanılabilir)"));
        assert!(text.contains("\nSürücü: E:"));
    }
}
