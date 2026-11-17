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
  security: Security;
  children: ChildDevice[];
}

export interface ChildDevice {
  instanceId: string;
  name: string;
  kind: DeviceKind;
  className: string;
  service: string;
}

export type RiskLevel = "none" | "info" | "medium" | "high";

export interface Security {
  rootId: string;
  keyboard: boolean;
  storage: boolean;
  level: RiskLevel;
  reasons: string[];
  allowed: boolean | null;
}

export interface AllowEntry {
  vendorId: string;
  productId: string;
  serialNumber: string | null;
  name: string;
  addedAt: string;
}

export interface LogEvent {
  at: string;
  action: "arrival" | "removal";
  instanceId: string;
  name: string;
  kind: DeviceKind | null;
  vendorId: string | null;
  productId: string | null;
  serialNumber: string | null;
  driveLetters: string[];
  user: string;
  alerts: string[];
  level: RiskLevel;
  blocked: boolean;
  blockError: string | null;
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
  alerts: string[];
  level: RiskLevel;
  blocked: boolean;
  blockError: string | null;
}

export type ExportFormat = "html" | "csv" | "json";
export type Lang = "tr" | "en";
export type StatusFilter = "all" | "connected" | "history" | "disabled" | "alerts";
export type SortKey = "name" | "kind" | "vidpid" | "serial" | "drive" | "lastArrival" | "lastRemoval" | "firstInstall";

export interface Settings {
  lang: Lang;
  theme: "dark" | "light";
  hideHubs: boolean;
  sidebarCollapsed: boolean;
  showInterfaces: boolean;
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
  lastSeenVersion: string;
  securityAlerts: boolean;
  newKeyboardAlert: boolean;
  eventLog: boolean;
  allowlistEnabled: boolean;
  allowlistAction: "warn" | "block";
  allowlistScope: "storage" | "all";
  allowlist: AllowEntry[];
}

export interface UpdateInfo {
  current: string;
  latest: string;
  available: boolean;
  url: string | null;
  notes: string | null;
  date: string | null;
  installer: string | null;
  sha256: string | null;
  size: number | null;
}

export type ChangeType = "new" | "improved" | "fixed";
export interface ChangeItem { type: ChangeType; tr: string; en: string }
export interface ChangeEntry { version: string; date: string; items: ChangeItem[] }
