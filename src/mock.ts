import type { LogEvent, Security, SystemInfo, UsbDevice } from "./types";

// Used when the UI runs in a plain browser (`npm run dev`) instead of Tauri.
const base = {
  manufacturer: "(Standard)",
  description: "",
  disabled: false as boolean,
  problemCode: null as number | null,
  location: "Port_#0002.Hub_#0001",
  parentId: "USB\\ROOT_HUB30\\4&2B5D1F3&0&0",
  driverVersion: "10.0.22621.1",
};

type RawDevice = Omit<UsbDevice, "security" | "children"> & { security?: Partial<Security>; children?: UsbDevice["children"] };

const raw: RawDevice[] = [
  { ...base, instanceId: "USB\\VID_0781&PID_5567\\4C530001234567", name: "SanDisk Cruzer Blade", kind: "storage", className: "USB", service: "USBSTOR", vendorId: "0781", productId: "5567", serialNumber: "4C530001234567", connected: true, driveLetters: ["E:"], firstInstall: "2025-03-02T09:14:00Z", lastArrival: "2026-11-16T07:02:11Z", lastRemoval: "2026-11-12T15:40:00Z" },
  { ...base, instanceId: "USB\\VID_046D&PID_C52B\\6&1A2B3C4D&0&2", name: "Logitech USB Receiver", kind: "hid", className: "HIDClass", service: "HidUsb", vendorId: "046D", productId: "C52B", serialNumber: null, connected: true, driveLetters: [], firstInstall: "2024-01-10T10:00:00Z", lastArrival: "2026-11-16T06:55:00Z", lastRemoval: null },
  { ...base, instanceId: "USB\\VID_04E8&PID_6860\\R58M12ABCDE", name: "Galaxy S23", kind: "phone", className: "WPD", service: "WUDFWpdMtp", vendorId: "04E8", productId: "6860", serialNumber: "R58M12ABCDE", connected: false, driveLetters: [], firstInstall: "2025-08-20T18:30:00Z", lastArrival: "2026-11-14T20:11:00Z", lastRemoval: "2026-11-14T21:05:00Z" },
  { ...base, instanceId: "USB\\VID_0951&PID_1666\\60A44C413A8CF3A0B9A1", name: "Kingston DataTraveler 3.0", kind: "storage", className: "USB", service: "USBSTOR", vendorId: "0951", productId: "1666", serialNumber: "60A44C413A8CF3A0B9A1", connected: false, driveLetters: [], firstInstall: "2026-11-10T11:22:00Z", lastArrival: "2026-11-10T11:22:00Z", lastRemoval: "2026-11-10T11:48:00Z" },
  { ...base, instanceId: "USB\\VID_046D&PID_085E\\C1A2B3C4", name: "Logitech BRIO", kind: "video", className: "Camera", service: "usbvideo", vendorId: "046D", productId: "085E", serialNumber: "C1A2B3C4", connected: true, driveLetters: [], firstInstall: "2025-05-05T08:00:00Z", lastArrival: "2026-11-16T06:55:02Z", lastRemoval: null },
  { ...base, instanceId: "USB\\VID_1A86&PID_7523\\5&2C1A0B&0&3", name: "USB-SERIAL CH340 (COM5)", kind: "serial", className: "Ports", service: "CH341SER_A64", vendorId: "1A86", productId: "7523", serialNumber: null, connected: false, disabled: true, driveLetters: [], firstInstall: "2025-12-01T13:00:00Z", lastArrival: "2026-10-28T09:12:00Z", lastRemoval: "2026-10-28T17:30:00Z" },
  { ...base, instanceId: "USB\\VID_8087&PID_0026\\5&3A2B1C0&0&10", name: "Intel Wireless Bluetooth", kind: "bluetooth", className: "Bluetooth", service: "BTHUSB", vendorId: "8087", productId: "0026", serialNumber: null, connected: true, driveLetters: [], firstInstall: "2024-01-10T09:58:00Z", lastArrival: "2026-11-16T06:55:00Z", lastRemoval: null },
  { ...base, instanceId: "USB\\ROOT_HUB30\\4&2B5D1F3&0&0", name: "USB Root Hub (USB 3.0)", kind: "hub", className: "USB", service: "USBHUB3", vendorId: null, productId: null, serialNumber: null, connected: true, driveLetters: [], firstInstall: "2024-01-10T09:58:00Z", lastArrival: "2026-11-16T06:55:00Z", lastRemoval: null },
  { ...base, instanceId: "USB\\VID_16D0&PID_0753\\5&1C2B3A4&0&4", name: "Digispark Keyboard", kind: "hid", className: "HIDClass", service: "HidUsb", vendorId: "16D0", productId: "0753", serialNumber: null, connected: false, driveLetters: [], firstInstall: "2026-11-15T19:40:00Z", lastArrival: "2026-11-15T19:40:00Z", lastRemoval: "2026-11-15T19:43:00Z",
    security: { keyboard: true, level: "high", reasons: ["PROGRAMMABLE_BOARD:Digispark / MCS"] } },
  { ...base, instanceId: "USB\\VID_1234&PID_ABCD\\0123456789", name: "USB Composite Device", kind: "storage", className: "USB", service: "usbccgp", vendorId: "1234", productId: "ABCD", serialNumber: "0123456789", connected: true, driveLetters: ["F:"], firstInstall: "2026-11-16T21:58:00Z", lastArrival: "2026-11-16T21:58:00Z", lastRemoval: null,
    security: { keyboard: true, storage: true, level: "high", reasons: ["KEYBOARD_AND_STORAGE", "NEW_KEYBOARD"] },
    children: [
      { instanceId: "USB\\VID_1234&PID_ABCD&MI_00\\7&2A&0&0000", name: "USB Input Device", kind: "hid", className: "HIDClass", service: "HidUsb" },
      { instanceId: "USB\\VID_1234&PID_ABCD&MI_01\\7&2A&0&0001", name: "USB Mass Storage Device", kind: "storage", className: "USB", service: "USBSTOR" },
      { instanceId: "USBSTOR\\DISK&VEN_GENERIC&PROD_FLASH\\0123456789&0", name: "Generic Flash Disk USB Device", kind: "storage", className: "DiskDrive", service: "disk" },
    ] },
];

export const mockDevices: UsbDevice[] = raw.map((d) => ({
  ...d,
  description: d.name,
  children: d.children ?? [],
  security: {
    rootId: d.instanceId,
    keyboard: d.kind === "hid",
    storage: d.kind === "storage",
    level: "none",
    reasons: [],
    allowed: null,
    ...d.security,
  },
}));

const ev = (at: string, action: LogEvent["action"], d: UsbDevice, extra: Partial<LogEvent> = {}): LogEvent => ({
  at, action, instanceId: d.instanceId, name: d.name, kind: d.kind, vendorId: d.vendorId, productId: d.productId,
  serialNumber: d.serialNumber, driveLetters: action === "arrival" ? d.driveLetters : [], user: "yunus",
  alerts: [], level: "none", blocked: false, blockError: null, ...extra,
});
const byName = (n: string) => mockDevices.find((d) => d.name.startsWith(n))!;

export const mockEvents: LogEvent[] = [
  ev("2026-11-16T21:58:05Z", "arrival", byName("USB Composite"), { alerts: ["KEYBOARD_AND_STORAGE", "NEW_KEYBOARD"], level: "high" }),
  ev("2026-11-16T19:02:11Z", "arrival", byName("SanDisk")),
  ev("2026-11-16T15:40:00Z", "removal", byName("SanDisk")),
  ev("2026-11-16T13:12:30Z", "arrival", byName("Kingston"), { alerts: ["NOT_ALLOWED"], level: "high", blocked: true }),
  ev("2026-11-16T13:12:00Z", "arrival", byName("SanDisk")),
  ev("2026-11-15T19:43:00Z", "removal", byName("Digispark")),
  ev("2026-11-15T19:40:02Z", "arrival", byName("Digispark"), { alerts: ["PROGRAMMABLE_BOARD:Digispark / MCS", "NEW_KEYBOARD"], level: "high" }),
  ev("2026-11-14T21:05:00Z", "removal", byName("Galaxy")),
  ev("2026-11-14T20:11:00Z", "arrival", byName("Galaxy")),
];

export const mockSystem: SystemInfo = {
  isWindows: false,
  elevated: false,
  computerName: "DEMO-PC",
  userName: "demo",
  os: "browser",
};
