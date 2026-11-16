export type DeviceKind =
  | "storage"
  | "hid"
  | "audio"
  | "video"
  | "phone"
  | "printer"
  | "network"
  | "bluetooth"
  | "serial"
  | "hub"
  | "other";

export interface UsbDevice {
  instanceId: string;
  name: string;
  description: string;
  kind: DeviceKind;
  className: string;
  manufacturer: string;
  service: string;
  vendorId: string | null;
  productId: string | null;
  serialNumber: string | null;
  connected: boolean;
  disabled: boolean;
  problemCode: number | null;
  driveLetters: string[];
  location: string | null;
  parentId: string | null;
  driverVersion: string | null;
  firstInstall: string | null;
  lastArrival: string | null;
  lastRemoval: string | null;
}

export interface SystemInfo {
  isWindows: boolean;
  elevated: boolean;
  computerName: string;
  userName: string;
  os: string;
}

export interface UsbEvent {
  action: "arrival" | "removal";
  instanceId: string | null;
  device: UsbDevice | null;
}

export type ExportFormat = "html" | "csv" | "json";
export type Lang = "tr" | "en";
export type StatusFilter = "all" | "connected" | "history" | "disabled";
export type SortKey = "name" | "kind" | "vidpid" | "serial" | "drive" | "lastArrival" | "lastRemoval" | "firstInstall";

export interface Settings {
  lang: Lang;
  theme: "dark" | "light";
  hideHubs: boolean;
  alwaysOnTop: boolean;
  closeToTray: boolean;
  startWithWindows: boolean;
  startMinimized: boolean;
  systemNotifications: boolean;
  notifyOnRemoval: boolean;
  soundOnArrival: boolean;
  soundOnRemoval: boolean;
  soundStyle: "chime" | "voice";
  soundVolume: number;
  bringToFrontOnStorage: boolean;
  openExplorerOnStorage: boolean;
  checkUpdatesOnStart: boolean;
}

export interface UpdateInfo {
  current: string;
  latest: string;
  available: boolean;
  url: string | null;
  notes: string | null;
  date: string | null;
}
