import type { SystemInfo, UsbDevice } from "./types";

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

const raw: UsbDevice[] = [
  { ...base, instanceId: "USB\\VID_0781&PID_5567\\4C530001234567", name: "SanDisk Cruzer Blade", kind: "storage", className: "USB", service: "USBSTOR", vendorId: "0781", productId: "5567", serialNumber: "4C530001234567", connected: true, driveLetters: ["E:"], firstInstall: "2025-03-02T09:14:00Z", lastArrival: "2026-11-16T07:02:11Z", lastRemoval: "2026-11-12T15:40:00Z" },
  { ...base, instanceId: "USB\\VID_046D&PID_C52B\\6&1A2B3C4D&0&2", name: "Logitech USB Receiver", kind: "hid", className: "HIDClass", service: "HidUsb", vendorId: "046D", productId: "C52B", serialNumber: null, connected: true, driveLetters: [], firstInstall: "2024-01-10T10:00:00Z", lastArrival: "2026-11-16T06:55:00Z", lastRemoval: null },
  { ...base, instanceId: "USB\\VID_04E8&PID_6860\\R58M12ABCDE", name: "Galaxy S23", kind: "phone", className: "WPD", service: "WUDFWpdMtp", vendorId: "04E8", productId: "6860", serialNumber: "R58M12ABCDE", connected: false, driveLetters: [], firstInstall: "2025-08-20T18:30:00Z", lastArrival: "2026-11-14T20:11:00Z", lastRemoval: "2026-11-14T21:05:00Z" },
  { ...base, instanceId: "USB\\VID_0951&PID_1666\\60A44C413A8CF3A0B9A1", name: "Kingston DataTraveler 3.0", kind: "storage", className: "USB", service: "USBSTOR", vendorId: "0951", productId: "1666", serialNumber: "60A44C413A8CF3A0B9A1", connected: false, driveLetters: [], firstInstall: "2026-11-10T11:22:00Z", lastArrival: "2026-11-10T11:22:00Z", lastRemoval: "2026-11-10T11:48:00Z" },
  { ...base, instanceId: "USB\\VID_046D&PID_085E\\C1A2B3C4", name: "Logitech BRIO", kind: "video", className: "Camera", service: "usbvideo", vendorId: "046D", productId: "085E", serialNumber: "C1A2B3C4", connected: true, driveLetters: [], firstInstall: "2025-05-05T08:00:00Z", lastArrival: "2026-11-16T06:55:02Z", lastRemoval: null },
  { ...base, instanceId: "USB\\VID_1A86&PID_7523\\5&2C1A0B&0&3", name: "USB-SERIAL CH340 (COM5)", kind: "serial", className: "Ports", service: "CH341SER_A64", vendorId: "1A86", productId: "7523", serialNumber: null, connected: false, disabled: true, driveLetters: [], firstInstall: "2025-12-01T13:00:00Z", lastArrival: "2026-10-28T09:12:00Z", lastRemoval: "2026-10-28T17:30:00Z" },
  { ...base, instanceId: "USB\\VID_8087&PID_0026\\5&3A2B1C0&0&10", name: "Intel Wireless Bluetooth", kind: "bluetooth", className: "Bluetooth", service: "BTHUSB", vendorId: "8087", productId: "0026", serialNumber: null, connected: true, driveLetters: [], firstInstall: "2024-01-10T09:58:00Z", lastArrival: "2026-11-16T06:55:00Z", lastRemoval: null },
  { ...base, instanceId: "USB\\ROOT_HUB30\\4&2B5D1F3&0&0", name: "USB Root Hub (USB 3.0)", kind: "hub", className: "USB", service: "USBHUB3", vendorId: null, productId: null, serialNumber: null, connected: true, driveLetters: [], firstInstall: "2024-01-10T09:58:00Z", lastArrival: "2026-11-16T06:55:00Z", lastRemoval: null },
];

export const mockDevices: UsbDevice[] = raw.map((d) => ({ ...d, description: d.name }));

export const mockSystem: SystemInfo = {
  isWindows: false,
  elevated: false,
  computerName: "DEMO-PC",
  userName: "demo",
  os: "browser",
};
