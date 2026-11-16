//! CSV / JSON / HTML (printable, forensic) report generation.

use sha2::{Digest, Sha256};

use crate::model::{SystemInfo, UsbDevice};

pub struct ReportMeta<'a> {
    pub system: &'a SystemInfo,
    pub generated_at: String,
    pub app_version: &'a str,
    pub lang: &'a str,
}

fn tr<'a>(lang: &str, tr: &'a str, en: &'a str) -> &'a str {
    if lang == "tr" {
        tr
    } else {
        en
    }
}

/// SHA-256 over the canonical JSON of the device list. Recomputing it from the
/// JSON export proves the data was not modified after the report was made.
pub fn digest(devices: &[UsbDevice]) -> String {
    let json = serde_json::to_vec(devices).unwrap_or_default();
    Sha256::digest(&json)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn opt(v: &Option<String>) -> &str {
    v.as_deref().unwrap_or("")
}

fn csv_cell(s: &str) -> String {
    // Also neutralise spreadsheet formula injection.
    let s = if s.starts_with(['=', '+', '-', '@']) {
        format!("'{s}")
    } else {
        s.to_string()
    };
    if s.contains([',', '"', '\n', '\r', ';']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s
    }
}

fn row(d: &UsbDevice, lang: &str) -> Vec<String> {
    vec![
        d.name.clone(),
        format!("{:?}", d.kind).to_lowercase(),
        if d.connected { tr(lang, "Bağlı", "Connected") } else { tr(lang, "Bağlı değil", "Disconnected") }.into(),
        if d.disabled { tr(lang, "Evet", "Yes") } else { tr(lang, "Hayır", "No") }.into(),
        opt(&d.vendor_id).into(),
        opt(&d.product_id).into(),
        opt(&d.serial_number).into(),
        d.drive_letters.join(" "),
        opt(&d.first_install).into(),
        opt(&d.last_arrival).into(),
        opt(&d.last_removal).into(),
        d.description.clone(),
        d.class_name.clone(),
        d.service.clone(),
        d.manufacturer.clone(),
        d.instance_id.clone(),
    ]
}

fn headers(lang: &str) -> Vec<&'static str> {
    if lang == "tr" {
        vec![
            "Ad", "Tür", "Durum", "Devre dışı", "VID", "PID", "Seri No", "Sürücü",
            "İlk kurulum (UTC)", "Son takılma (UTC)", "Son çıkarılma (UTC)", "Açıklama",
            "Sınıf", "Servis", "Üretici", "Instance ID",
        ]
    } else {
        vec![
            "Name", "Type", "Status", "Disabled", "VID", "PID", "Serial", "Drive",
            "First install (UTC)", "Last arrival (UTC)", "Last removal (UTC)", "Description",
            "Class", "Service", "Manufacturer", "Instance ID",
        ]
    }
}

pub fn to_csv(devices: &[UsbDevice], lang: &str) -> String {
    // BOM so Excel opens UTF-8 (Turkish characters) correctly.
    let mut out = String::from("\u{feff}");
    out.push_str(&headers(lang).iter().map(|h| csv_cell(h)).collect::<Vec<_>>().join(","));
    out.push_str("\r\n");
    for d in devices {
        out.push_str(&row(d, lang).iter().map(|c| csv_cell(c)).collect::<Vec<_>>().join(","));
        out.push_str("\r\n");
    }
    out
}

pub fn to_json(devices: &[UsbDevice], meta: &ReportMeta) -> String {
    let value = serde_json::json!({
        "generator": format!("UsbLens {}", meta.app_version),
        "generatedAt": meta.generated_at,
        "computerName": meta.system.computer_name,
        "userName": meta.system.user_name,
        "os": meta.system.os,
        "sha256": digest(devices),
        "devices": devices,
    });
    serde_json::to_string_pretty(&value).unwrap_or_default()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn to_html(devices: &[UsbDevice], meta: &ReportMeta) -> String {
    let l = meta.lang;
    let title = tr(l, "USB Cihaz Raporu", "USB Device Report");
    let connected = devices.iter().filter(|d| d.connected).count();
    let storage = devices
        .iter()
        .filter(|d| matches!(d.kind, crate::model::DeviceKind::Storage))
        .count();

    let head: String = headers(l)
        .iter()
        .take(11)
        .map(|h| format!("<th>{}</th>", esc(h)))
        .collect();
    let body: String = devices
        .iter()
        .map(|d| {
            let cells: String = row(d, l)
                .iter()
                .take(11)
                .map(|c| format!("<td>{}</td>", esc(c)))
                .collect();
            format!("<tr title=\"{}\">{cells}</tr>", esc(&d.instance_id))
        })
        .collect();

    format!(
        r#"<!DOCTYPE html>
<html lang="{l}"><head><meta charset="utf-8"><title>{title}</title>
<style>
  body {{ font: 12px/1.45 "Segoe UI", system-ui, sans-serif; color:#111; margin:32px; }}
  h1 {{ font-size:20px; margin:0 0 4px; }}
  .meta {{ display:grid; grid-template-columns:max-content 1fr; gap:2px 16px; margin:16px 0; }}
  .meta b {{ color:#555; font-weight:600; }}
  code {{ font: 11px ui-monospace, Consolas, monospace; word-break:break-all; }}
  table {{ border-collapse:collapse; width:100%; }}
  th, td {{ border:1px solid #ddd; padding:4px 6px; text-align:left; vertical-align:top; }}
  th {{ background:#f3f4f6; font-weight:600; }}
  tr:nth-child(even) td {{ background:#fafafa; }}
  .foot {{ margin-top:16px; color:#666; font-size:11px; }}
  @media print {{ body {{ margin:12mm; }} @page {{ size: A4 landscape; }} }}
</style></head><body>
<h1>{title}</h1>
<div class="meta">
  <b>{l_gen}</b><span>{gen}</span>
  <b>{l_pc}</b><span>{pc}</span>
  <b>{l_user}</b><span>{user}</span>
  <b>{l_os}</b><span>{os}</span>
  <b>{l_count}</b><span>{count} ({l_conn}: {connected}, {l_stor}: {storage})</span>
  <b>SHA-256</b><code>{hash}</code>
</div>
<table><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table>
<p class="foot">{foot} — UsbLens {ver}</p>
</body></html>"#,
        l_gen = tr(l, "Oluşturulma", "Generated"),
        gen = esc(&meta.generated_at),
        l_pc = tr(l, "Bilgisayar", "Computer"),
        pc = esc(&meta.system.computer_name),
        l_user = tr(l, "Kullanıcı", "User"),
        user = esc(&meta.system.user_name),
        l_os = tr(l, "İşletim sistemi", "Operating system"),
        os = esc(&meta.system.os),
        l_count = tr(l, "Cihaz sayısı", "Devices"),
        count = devices.len(),
        l_conn = tr(l, "bağlı", "connected"),
        l_stor = tr(l, "depolama", "storage"),
        hash = digest(devices),
        foot = tr(
            l,
            "SHA-256 özeti, aynı anda alınan JSON dışa aktarımındaki cihaz listesinden hesaplanır.",
            "The SHA-256 digest is computed over the device list in a JSON export taken at the same time."
        ),
        ver = meta.app_version,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_escapes() {
        assert_eq!(csv_cell("a,b"), "\"a,b\"");
        assert_eq!(csv_cell("=cmd"), "'=cmd");
        assert_eq!(csv_cell("x\"y"), "\"x\"\"y\"");
    }
}
