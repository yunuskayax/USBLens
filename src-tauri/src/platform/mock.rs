//! Sample data for non-Windows hosts so the UI can be developed anywhere.

use crate::model::*;

const NOT_WINDOWS: &str = "WINDOWS_ONLY";

fn dev(
    id: &str,
    name: &str,
    class_name: &str,
    service: &str,
    connected: bool,
    letters: &[&str],
    first: &str,
    arrival: &str,
    removal: Option<&str>,
) -> UsbDevice {
    let (vendor_id, product_id, serial_number) = parse_instance_id(id);
    UsbDevice {
        instance_id: id.into(),
        name: name.into(),
        description: name.into(),
        kind: classify(class_name, service, name, id),
        class_name: class_name.into(),
        manufacturer: "(Standard)".into(),
        service: service.into(),
        vendor_id,
        product_id,
        serial_number,
        connected,
        disabled: false,
        problem_code: None,
        drive_letters: letters.iter().map(|s| s.to_string()).collect(),
        capacity_bytes: None,
        removable: None,
        vendor_name: None,
        product_name: None,
        location: Some("Port_#0002.Hub_#0001".into()),
        parent_id: Some("USB\\ROOT_HUB30\\4&2B5D1F3&0&0".into()),
        driver_version: Some("10.0.22621.1".into()),
        first_install: Some(first.into()),
        last_arrival: Some(arrival.into()),
        last_removal: removal.map(Into::into),
        security: Default::default(),
        children: Vec::new(),
    }
}

pub fn list_devices() -> Result<Vec<UsbDevice>, String> {
    Ok(vec![
        dev("USB\\VID_0781&PID_5567\\4C530001234567", "SanDisk Cruzer Blade", "USB", "USBSTOR", true, &["E:"],
            "2025-03-02T09:14:00Z", "2026-11-16T07:02:11Z", Some("2026-11-12T15:40:00Z")),
        dev("USB\\VID_046D&PID_C52B\\6&1A2B3C4D&0&2", "Logitech USB Receiver", "HIDClass", "HidUsb", true, &[],
            "2024-01-10T10:00:00Z", "2026-11-16T06:55:00Z", None),
        dev("USB\\VID_04E8&PID_6860\\R58M12ABCDE", "Galaxy S23", "WPD", "WUDFWpdMtp", false, &[],
            "2025-08-20T18:30:00Z", "2026-11-14T20:11:00Z", Some("2026-11-14T21:05:00Z")),
        dev("USB\\VID_0951&PID_1666\\60A44C413A8CF3A0B9A1", "Kingston DataTraveler 3.0", "USB", "USBSTOR", false, &[],
            "2026-11-10T11:22:00Z", "2026-11-10T11:22:00Z", Some("2026-11-10T11:48:00Z")),
        dev("USB\\VID_046D&PID_085E\\C1A2B3C4", "Logitech BRIO", "Camera", "usbvideo", true, &[],
            "2025-05-05T08:00:00Z", "2026-11-16T06:55:02Z", None),
        dev("USB\\VID_8087&PID_0026\\5&3A2B1C0&0&10", "Intel Wireless Bluetooth", "Bluetooth", "BTHUSB", true, &[],
            "2024-01-10T09:58:00Z", "2026-11-16T06:55:00Z", None),
        dev("USB\\VID_16D0&PID_0753\\5&1C2B3A4&0&4", "Digispark Keyboard", "HIDClass", "HidUsb", false, &[],
            "2026-11-15T19:40:00Z", "2026-11-15T19:40:00Z", Some("2026-11-15T19:43:00Z")),
        dev("USB\\ROOT_HUB30\\4&2B5D1F3&0&0", "USB Root Hub (USB 3.0)", "USB", "USBHUB3", true, &[],
            "2024-01-10T09:58:00Z", "2026-11-16T06:55:00Z", None),
    ])
}

#[allow(dead_code)]
/// The mock Logitech receiver and Digispark act as keyboards.
pub fn keyboard_parent_ids() -> std::collections::HashSet<String> {
    ["USB\\VID_046D&PID_C52B\\6&1A2B3C4D&0&2", "USB\\VID_16D0&PID_0753\\5&1C2B3A4&0&4"]
        .into_iter()
        .map(String::from)
        .collect()
}

#[allow(dead_code)]
pub fn get_device(instance_id: &str) -> Option<UsbDevice> {
    list_devices().ok()?.into_iter().find(|d| d.instance_id == instance_id)
}

pub fn set_enabled(_id: &str, _enable: bool) -> Result<(), String> {
    Err(NOT_WINDOWS.into())
}

pub fn uninstall(_id: &str) -> Result<(), String> {
    Err(NOT_WINDOWS.into())
}

pub fn eject(_id: &str) -> Result<(), String> {
    Err(NOT_WINDOWS.into())
}

pub fn is_elevated() -> bool {
    false
}

pub fn relaunch_as_admin() -> Result<(), String> {
    Err(NOT_WINDOWS.into())
}

pub fn start_monitor<F>(_emit: F) -> Result<(), String>
where
    F: Fn(UsbEvent) + Send + 'static,
{
    Ok(())
}
