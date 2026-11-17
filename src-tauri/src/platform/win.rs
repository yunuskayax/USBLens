//! Windows backend: SetupAPI + CfgMgr32 for enumeration, actions and live
//! notifications. Everything here works for both present and "phantom"
//! (previously connected) devices unless noted.

use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::Duration;

use windows_sys::Win32::Devices::DeviceAndDriverInstallation::*;
use windows_sys::Win32::Devices::Properties::*;
use windows_sys::Win32::Devices::Usb::GUID_DEVINTERFACE_USB_DEVICE;
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, DEVPROPKEY, ERROR_ACCESS_DENIED, ERROR_SUCCESS, HANDLE,
    INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, GetLogicalDrives, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows_sys::Win32::System::Ioctl::{
    GUID_DEVINTERFACE_DISK, IOCTL_STORAGE_GET_DEVICE_NUMBER, STORAGE_DEVICE_NUMBER,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows_sys::Win32::System::IO::DeviceIoControl;
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use crate::model::*;

// ---------------------------------------------------------------- helpers

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

unsafe fn from_wide_ptr(p: *const u16) -> String {
    if p.is_null() {
        return String::new();
    }
    let mut len = 0;
    while *p.add(len) != 0 {
        len += 1;
    }
    String::from_utf16_lossy(std::slice::from_raw_parts(p, len))
}

const ERR_ADMIN: &str = "ADMIN_REQUIRED";
const ERR_NOT_FOUND: &str = "NOT_FOUND";
/// SetupAPI's ERROR_NO_SUCH_DEVINST (0xE000020B).
const ERROR_NO_SUCH_DEVINST: u32 = 0xE000_020B;

/// Turn the thread's last Win32 error into a message the UI understands.
fn last_error() -> String {
    let code = unsafe { GetLastError() };
    match code {
        ERROR_ACCESS_DENIED => ERR_ADMIN.to_string(),
        ERROR_NO_SUCH_DEVINST => ERR_NOT_FOUND.to_string(),
        _ => std::io::Error::from_raw_os_error(code as i32).to_string(),
    }
}

fn check(ok: i32) -> Result<(), String> {
    if ok != 0 {
        Ok(())
    } else {
        Err(last_error())
    }
}

unsafe fn devinfo_data() -> SP_DEVINFO_DATA {
    let mut d: SP_DEVINFO_DATA = zeroed();
    d.cbSize = size_of::<SP_DEVINFO_DATA>() as u32;
    d
}

/// RAII wrapper so device info sets are always freed.
struct DevInfoSet(HDEVINFO);

impl DevInfoSet {
    fn new(h: HDEVINFO) -> Option<Self> {
        (h != INVALID_HANDLE_VALUE as HDEVINFO && h != 0).then_some(Self(h))
    }
}

impl Drop for DevInfoSet {
    fn drop(&mut self) {
        unsafe {
            SetupDiDestroyDeviceInfoList(self.0);
        }
    }
}

struct Handle(HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

// ---------------------------------------------------------------- properties

struct Props<'a> {
    set: HDEVINFO,
    data: &'a SP_DEVINFO_DATA,
}

impl Props<'_> {
    fn raw(&self, key: &DEVPROPKEY) -> Option<(DEVPROPTYPE, Vec<u8>)> {
        unsafe {
            let mut ty: DEVPROPTYPE = 0;
            let mut size = 0u32;
            SetupDiGetDevicePropertyW(self.set, self.data, key, &mut ty, null_mut(), 0, &mut size, 0);
            if size == 0 {
                return None;
            }
            let mut buf = vec![0u8; size as usize];
            let ok = SetupDiGetDevicePropertyW(
                self.set,
                self.data,
                key,
                &mut ty,
                buf.as_mut_ptr(),
                size,
                &mut size,
                0,
            );
            (ok != 0).then_some((ty, buf))
        }
    }

    fn string(&self, key: &DEVPROPKEY) -> Option<String> {
        let (ty, buf) = self.raw(key)?;
        if ty != DEVPROP_TYPE_STRING {
            return None;
        }
        let units: Vec<u16> = buf
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let s = from_wide(&units).trim().to_string();
        (!s.is_empty()).then_some(s)
    }

    fn u32(&self, key: &DEVPROPKEY) -> Option<u32> {
        let (ty, buf) = self.raw(key)?;
        (ty == DEVPROP_TYPE_UINT32 && buf.len() >= 4)
            .then(|| u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]))
    }

    fn boolean(&self, key: &DEVPROPKEY) -> Option<bool> {
        let (ty, buf) = self.raw(key)?;
        (ty == DEVPROP_TYPE_BOOLEAN && !buf.is_empty()).then(|| buf[0] != 0)
    }

    fn filetime(&self, key: &DEVPROPKEY) -> Option<String> {
        let (ty, buf) = self.raw(key)?;
        if ty != DEVPROP_TYPE_FILETIME || buf.len() < 8 {
            return None;
        }
        let mut b = [0u8; 8];
        b.copy_from_slice(&buf[..8]);
        filetime_to_rfc3339(u64::from_le_bytes(b))
    }
}

unsafe fn instance_id_of(set: HDEVINFO, data: &SP_DEVINFO_DATA) -> Option<String> {
    let mut buf = [0u16; 512];
    let mut needed = 0u32;
    let ok = SetupDiGetDeviceInstanceIdW(set, data, buf.as_mut_ptr(), buf.len() as u32, &mut needed);
    (ok != 0).then(|| from_wide(&buf))
}

// ---------------------------------------------------------------- drive letters

unsafe fn storage_number(path: &str) -> Option<(u32, u32)> {
    let w = wide(path);
    // Zero access rights: works without admin and without touching the media.
    let h = CreateFileW(
        w.as_ptr(),
        0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        null(),
        OPEN_EXISTING,
        0,
        null_mut(),
    );
    if h == INVALID_HANDLE_VALUE || h.is_null() {
        return None;
    }
    let h = Handle(h);
    let mut sdn: STORAGE_DEVICE_NUMBER = zeroed();
    let mut returned = 0u32;
    let ok = DeviceIoControl(
        h.0,
        IOCTL_STORAGE_GET_DEVICE_NUMBER,
        null(),
        0,
        &mut sdn as *mut _ as *mut c_void,
        size_of::<STORAGE_DEVICE_NUMBER>() as u32,
        &mut returned,
        null_mut(),
    );
    (ok != 0).then_some((sdn.DeviceType, sdn.DeviceNumber))
}

/// Walk up the device tree and return the first ancestor whose id starts with `USB\`.
unsafe fn usb_ancestor(instance_id: &str) -> Option<String> {
    let w = wide(instance_id);
    let mut node = 0u32;
    if CM_Locate_DevNodeW(&mut node, w.as_ptr(), CM_LOCATE_DEVNODE_NORMAL) != CR_SUCCESS {
        return None;
    }
    for _ in 0..8 {
        let mut parent = 0u32;
        if CM_Get_Parent(&mut parent, node, 0) != CR_SUCCESS {
            return None;
        }
        let mut buf = [0u16; 512];
        if CM_Get_Device_IDW(parent, buf.as_mut_ptr(), buf.len() as u32, 0) != CR_SUCCESS {
            return None;
        }
        let id = from_wide(&buf);
        if id.to_ascii_uppercase().starts_with("USB\\") {
            return Some(id);
        }
        node = parent;
    }
    None
}

/// Map (upper-cased) instance ids of disks and their USB parents to drive letters.
fn drive_letter_map() -> HashMap<String, Vec<String>> {
    let mut by_number: HashMap<(u32, u32), Vec<String>> = HashMap::new();
    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    unsafe {
        let mask = GetLogicalDrives();
        for i in 0..26u32 {
            if mask & (1 << i) == 0 {
                continue;
            }
            let letter = (b'A' + i as u8) as char;
            if let Some(n) = storage_number(&format!("\\\\.\\{letter}:")) {
                by_number.entry(n).or_default().push(format!("{letter}:"));
            }
        }
        if by_number.is_empty() {
            return result;
        }

        let Some(set) = DevInfoSet::new(SetupDiGetClassDevsW(
            &GUID_DEVINTERFACE_DISK,
            null(),
            null_mut(),
            DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
        )) else {
            return result;
        };

        let mut index = 0;
        loop {
            let mut iface: SP_DEVICE_INTERFACE_DATA = zeroed();
            iface.cbSize = size_of::<SP_DEVICE_INTERFACE_DATA>() as u32;
            if SetupDiEnumDeviceInterfaces(set.0, null(), &GUID_DEVINTERFACE_DISK, index, &mut iface) == 0 {
                break;
            }
            index += 1;

            let mut needed = 0u32;
            SetupDiGetDeviceInterfaceDetailW(set.0, &iface, null_mut(), 0, &mut needed, null_mut());
            if needed == 0 {
                continue;
            }
            // u64 storage keeps the struct correctly aligned.
            let mut storage = vec![0u64; (needed as usize).div_ceil(8) + 1];
            let detail = storage.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
            (*detail).cbSize = size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;
            let mut dev = devinfo_data();
            if SetupDiGetDeviceInterfaceDetailW(set.0, &iface, detail, needed, null_mut(), &mut dev) == 0 {
                continue;
            }
            let path = from_wide_ptr((*detail).DevicePath.as_ptr());
            let Some(num) = storage_number(&path) else { continue };
            let Some(letters) = by_number.get(&num) else { continue };
            let Some(disk_id) = instance_id_of(set.0, &dev) else { continue };

            if let Some(usb) = usb_ancestor(&disk_id) {
                result
                    .entry(usb.to_ascii_uppercase())
                    .or_default()
                    .extend(letters.iter().cloned());
            }
            result
                .entry(disk_id.to_ascii_uppercase())
                .or_default()
                .extend(letters.iter().cloned());
        }
    }

    for v in result.values_mut() {
        v.sort();
        v.dedup();
    }
    result
}

// ---------------------------------------------------------------- enumeration

const CONFIGFLAG_DISABLED: u32 = 0x0000_0001;
const CM_PROB_DISABLED_CODE: u32 = 22;

unsafe fn build_device(
    set: HDEVINFO,
    data: &SP_DEVINFO_DATA,
    letters: &HashMap<String, Vec<String>>,
) -> Option<UsbDevice> {
    let instance_id = instance_id_of(set, data)?;
    let p = Props { set, data };

    let friendly = p.string(&DEVPKEY_Device_FriendlyName);
    let bus_name = p.string(&DEVPKEY_Device_BusReportedDeviceDesc);
    let description = p.string(&DEVPKEY_Device_DeviceDesc).unwrap_or_default();
    let name = friendly.or(bus_name).unwrap_or_else(|| description.clone());
    let name = if name.is_empty() { instance_id.clone() } else { name };

    let class_name = p.string(&DEVPKEY_Device_Class).unwrap_or_default();
    let service = p.string(&DEVPKEY_Device_Service).unwrap_or_default();
    let manufacturer = p.string(&DEVPKEY_Device_Manufacturer).unwrap_or_default();

    let connected = p.boolean(&DEVPKEY_Device_IsPresent).unwrap_or_else(|| {
        let w = wide(&instance_id);
        let mut node = 0u32;
        CM_Locate_DevNodeW(&mut node, w.as_ptr(), CM_LOCATE_DEVNODE_NORMAL) == CR_SUCCESS
    });
    let problem_code = if connected {
        p.u32(&DEVPKEY_Device_ProblemCode).filter(|c| *c != 0)
    } else {
        None
    };
    let config_flags = p.u32(&DEVPKEY_Device_ConfigFlags).unwrap_or(0);
    let disabled =
        config_flags & CONFIGFLAG_DISABLED != 0 || problem_code == Some(CM_PROB_DISABLED_CODE);

    let (vendor_id, product_id, serial_number) = parse_instance_id(&instance_id);
    let kind = classify(&class_name, &service, &name, &instance_id);

    Some(UsbDevice {
        drive_letters: letters
            .get(&instance_id.to_ascii_uppercase())
            .cloned()
            .unwrap_or_default(),
        location: p.string(&DEVPKEY_Device_LocationInfo),
        parent_id: p.string(&DEVPKEY_Device_Parent),
        driver_version: p.string(&DEVPKEY_Device_DriverVersion),
        first_install: p
            .filetime(&DEVPKEY_Device_FirstInstallDate)
            .or_else(|| p.filetime(&DEVPKEY_Device_InstallDate)),
        last_arrival: p.filetime(&DEVPKEY_Device_LastArrivalDate),
        last_removal: p.filetime(&DEVPKEY_Device_LastRemovalDate),
        security: Default::default(),
        children: Vec::new(),
        instance_id,
        name,
        description,
        kind,
        class_name,
        manufacturer,
        service,
        vendor_id,
        product_id,
        serial_number,
        connected,
        disabled,
        problem_code,
    })
}

pub fn list_devices() -> Result<Vec<UsbDevice>, String> {
    let letters = drive_letter_map();
    let mut out = Vec::new();

    for enumerator in ["USB", "USBSTOR"] {
        let e = wide(enumerator);
        unsafe {
            // No DIGCF_PRESENT: we also want devices that were connected in the past.
            let Some(set) = DevInfoSet::new(SetupDiGetClassDevsW(
                null(),
                e.as_ptr(),
                null_mut(),
                DIGCF_ALLCLASSES,
            )) else {
                continue;
            };
            let mut index = 0;
            loop {
                let mut data = devinfo_data();
                if SetupDiEnumDeviceInfo(set.0, index, &mut data) == 0 {
                    break;
                }
                index += 1;
                if let Some(d) = build_device(set.0, &data, &letters) {
                    out.push(d);
                }
            }
        }
    }
    Ok(out)
}

/// Parent instance ids (upper-cased) of every HID keyboard Windows knows about, connected or not.
/// The parent is the USB interface (`USB\VID_x&PID_y&MI_00\...`) or the USB device itself.
pub fn keyboard_parent_ids() -> HashSet<String> {
    let mut out = HashSet::new();
    let e = wide("HID");
    unsafe {
        let Some(set) = DevInfoSet::new(SetupDiGetClassDevsW(null(), e.as_ptr(), null_mut(), DIGCF_ALLCLASSES)) else {
            return out;
        };
        let mut index = 0;
        loop {
            let mut data = devinfo_data();
            if SetupDiEnumDeviceInfo(set.0, index, &mut data) == 0 {
                break;
            }
            index += 1;
            let p = Props { set: set.0, data: &data };
            let is_keyboard = p
                .string(&DEVPKEY_Device_Class)
                .map(|c| c.eq_ignore_ascii_case("Keyboard"))
                .unwrap_or(false);
            if !is_keyboard {
                continue;
            }
            if let Some(parent) = p.string(&DEVPKEY_Device_Parent) {
                out.insert(parent.to_ascii_uppercase());
            }
        }
    }
    out
}

unsafe fn open_single(instance_id: &str) -> Result<(DevInfoSet, SP_DEVINFO_DATA), String> {
    let set = DevInfoSet::new(SetupDiCreateDeviceInfoList(null(), null_mut())).ok_or_else(last_error)?;
    let mut data = devinfo_data();
    let w = wide(instance_id);
    check(SetupDiOpenDeviceInfoW(set.0, w.as_ptr(), null_mut(), 0, &mut data))?;
    Ok((set, data))
}

pub fn get_device(instance_id: &str) -> Option<UsbDevice> {
    unsafe {
        let (set, data) = open_single(instance_id).ok()?;
        build_device(set.0, &data, &drive_letter_map())
    }
}

// ---------------------------------------------------------------- actions

pub fn set_enabled(instance_id: &str, enable: bool) -> Result<(), String> {
    unsafe {
        let (set, data) = open_single(instance_id)?;
        let params = SP_PROPCHANGE_PARAMS {
            ClassInstallHeader: SP_CLASSINSTALL_HEADER {
                cbSize: size_of::<SP_CLASSINSTALL_HEADER>() as u32,
                InstallFunction: DIF_PROPERTYCHANGE,
            },
            StateChange: if enable { DICS_ENABLE } else { DICS_DISABLE },
            Scope: DICS_FLAG_GLOBAL,
            HwProfile: 0,
        };
        check(SetupDiSetClassInstallParamsW(
            set.0,
            &data,
            &params.ClassInstallHeader,
            size_of::<SP_PROPCHANGE_PARAMS>() as u32,
        ))?;
        check(SetupDiCallClassInstaller(DIF_PROPERTYCHANGE, set.0, &data))
    }
}

pub fn uninstall(instance_id: &str) -> Result<(), String> {
    unsafe {
        let (set, data) = open_single(instance_id)?;
        check(DiUninstallDevice(null_mut(), set.0, &data, 0, null_mut()))
    }
}

pub fn eject(instance_id: &str) -> Result<(), String> {
    unsafe {
        let w = wide(instance_id);
        let mut node = 0u32;
        if CM_Locate_DevNodeW(&mut node, w.as_ptr(), CM_LOCATE_DEVNODE_NORMAL) != CR_SUCCESS {
            return Err(ERR_NOT_FOUND.into());
        }
        let mut veto: PNP_VETO_TYPE = 0;
        let mut veto_name = [0u16; 512];
        let cr = CM_Request_Device_EjectW(
            node,
            &mut veto,
            veto_name.as_mut_ptr(),
            veto_name.len() as u32,
            0,
        );
        if cr == CR_SUCCESS && veto == 0 {
            Ok(())
        } else if cr == CR_ACCESS_DENIED {
            Err(ERR_ADMIN.into())
        } else {
            Err(format!("VETOED:{}", from_wide(&veto_name)))
        }
    }
}

pub fn is_elevated() -> bool {
    unsafe {
        let mut token: HANDLE = null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let token = Handle(token);
        let mut elevation: TOKEN_ELEVATION = zeroed();
        let mut size = 0u32;
        GetTokenInformation(
            token.0,
            TokenElevation,
            &mut elevation as *mut _ as *mut c_void,
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        ) != 0
            && elevation.TokenIsElevated != 0
    }
}

/// Start a new elevated copy of ourselves. The caller exits the current one.
pub fn relaunch_as_admin() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe = wide(&exe.to_string_lossy());
    let verb = wide("runas");
    unsafe {
        let r = ShellExecuteW(
            null_mut(),
            verb.as_ptr(),
            exe.as_ptr(),
            null(),
            null(),
            SW_SHOWNORMAL,
        );
        if r as isize > 32 {
            Ok(())
        } else {
            Err("CANCELLED".into())
        }
    }
}

// ---------------------------------------------------------------- live monitor

static EVENTS: OnceLock<Mutex<mpsc::Sender<(bool, String)>>> = OnceLock::new();

/// Wrapper so the notification handle can live in a static.
struct Registration(#[allow(dead_code)] HCMNOTIFICATION);
unsafe impl Send for Registration {}
unsafe impl Sync for Registration {}
static REGISTRATION: OnceLock<Registration> = OnceLock::new();

unsafe extern "system" fn on_notify(
    _h: HCMNOTIFICATION,
    _ctx: *const c_void,
    action: CM_NOTIFY_ACTION,
    data: *const CM_NOTIFY_EVENT_DATA,
    _size: u32,
) -> u32 {
    let arrival = action == CM_NOTIFY_ACTION_DEVICEINTERFACEARRIVAL;
    let removal = action == CM_NOTIFY_ACTION_DEVICEINTERFACEREMOVAL;
    if (arrival || removal) && !data.is_null() {
        let link = from_wide_ptr((*data).u.DeviceInterface.SymbolicLink.as_ptr());
        if let Some(tx) = EVENTS.get() {
            if let Ok(tx) = tx.lock() {
                let _ = tx.send((arrival, link));
            }
        }
    }
    ERROR_SUCCESS
}

/// Register for USB arrival/removal. `emit` is called on a worker thread.
pub fn start_monitor<F>(emit: F) -> Result<(), String>
where
    F: Fn(UsbEvent) + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<(bool, String)>();
    EVENTS
        .set(Mutex::new(tx))
        .map_err(|_| "monitor already running".to_string())?;

    std::thread::spawn(move || {
        for (arrival, link) in rx {
            if arrival {
                // Give PnP a moment to finish installing / assigning letters.
                std::thread::sleep(Duration::from_millis(800));
            }
            let instance_id = symlink_to_instance_id(&link);
            let device = instance_id.as_deref().and_then(get_device);
            emit(UsbEvent {
                action: if arrival { "arrival" } else { "removal" }.into(),
                instance_id,
                device,
                alerts: Vec::new(),
                level: Default::default(),
                blocked: false,
                block_error: None,
            });
        }
    });

    unsafe {
        let mut filter: CM_NOTIFY_FILTER = zeroed();
        filter.cbSize = size_of::<CM_NOTIFY_FILTER>() as u32;
        filter.FilterType = CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE;
        filter.u.DeviceInterface.ClassGuid = GUID_DEVINTERFACE_USB_DEVICE;
        let mut handle: HCMNOTIFICATION = null_mut();
        let cr = CM_Register_Notification(&filter, null(), Some(on_notify), &mut handle);
        if cr != CR_SUCCESS {
            return Err(format!("CM_Register_Notification failed: {cr}"));
        }
        // Keep the registration alive for the whole process lifetime.
        let _ = REGISTRATION.set(Registration(handle));
    }
    Ok(())
}
