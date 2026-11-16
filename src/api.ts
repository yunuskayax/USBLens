import type { ExportFormat, Lang, Settings, SystemInfo, UpdateInfo, UsbDevice, UsbEvent } from "./types";
import { mockDevices, mockSystem } from "./mock";

/** True inside the Tauri webview, false in a plain browser tab. */
export const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

export async function listDevices(): Promise<UsbDevice[]> {
  if (!inTauri) return structuredClone(mockDevices);
  return invoke<UsbDevice[]>("list_devices");
}

export async function systemInfo(): Promise<SystemInfo> {
  if (!inTauri) return mockSystem;
  return invoke<SystemInfo>("system_info");
}

export async function setDeviceEnabled(instanceId: string, enabled: boolean): Promise<void> {
  if (!inTauri) throw "WINDOWS_ONLY";
  return invoke("set_device_enabled", { instanceId, enabled });
}

export async function uninstallDevice(instanceId: string): Promise<void> {
  if (!inTauri) throw "WINDOWS_ONLY";
  return invoke("uninstall_device", { instanceId });
}

export async function ejectDevice(instanceId: string): Promise<void> {
  if (!inTauri) throw "WINDOWS_ONLY";
  return invoke("eject_device", { instanceId });
}

export async function relaunchAsAdmin(): Promise<void> {
  if (!inTauri) throw "WINDOWS_ONLY";
  return invoke("relaunch_as_admin");
}

/** Ask for a path, write the file in Rust. Returns the saved path or null if cancelled. */
export async function exportDevices(
  format: ExportFormat,
  lang: Lang,
  devices: UsbDevice[],
  computerName: string,
): Promise<string | null> {
  const stamp = new Date().toISOString().slice(0, 16).replace(/[:T]/g, "-");
  const fileName = `usblens-${computerName || "pc"}-${stamp}.${format}`;

  if (!inTauri) {
    // Browser fallback: download JSON/CSV-ish content for UI testing.
    const blob = new Blob([JSON.stringify(devices, null, 2)], { type: "application/json" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = fileName.replace(/\.\w+$/, ".json");
    a.click();
    return a.download;
  }

  const { save } = await import("@tauri-apps/plugin-dialog");
  const names: Record<ExportFormat, string> = { html: "HTML", csv: "CSV", json: "JSON" };
  const path = await save({
    defaultPath: fileName,
    filters: [{ name: names[format], extensions: [format] }],
  });
  if (!path) return null;
  await invoke<string>("export_devices", { path, format, lang, devices });
  return path;
}

export async function onUsbEvent(handler: (e: UsbEvent) => void): Promise<() => void> {
  if (!inTauri) return () => {};
  const { listen } = await import("@tauri-apps/api/event");
  return listen<UsbEvent>("usb-event", (e) => handler(e.payload));
}

export async function systemNotify(title: string, body: string): Promise<void> {
  if (!inTauri) return;
  try {
    const n = await import("@tauri-apps/plugin-notification");
    let granted = await n.isPermissionGranted();
    if (!granted) granted = (await n.requestPermission()) === "granted";
    if (granted) n.sendNotification({ title, body });
  } catch {
    /* notifications are best-effort */
  }
}

export const WEBSITE = "https://yunuskaya.com.tr";
export const PROJECT_SITE = "https://usblens.com";

export async function appVersion(): Promise<string> {
  if (!inTauri) return "0.2.2";
  const { getVersion } = await import("@tauri-apps/api/app");
  return getVersion();
}

/** Open a link in the user's default browser. */
export async function openUrl(url: string): Promise<void> {
  if (!inTauri) {
    window.open(url, "_blank", "noopener");
    return;
  }
  const { openUrl } = await import("@tauri-apps/plugin-opener");
  await openUrl(url);
}

export const defaultSettings: Settings = {
  lang: "tr",
  theme: "dark",
  hideHubs: true,
  sidebarCollapsed: false,
  alwaysOnTop: false,
  closeToTray: true,
  startWithWindows: false,
  startMinimized: false,
  systemNotifications: true,
  notifyOnRemoval: true,
  soundOnArrival: true,
  soundOnRemoval: false,
  soundStyle: "chime",
  soundVolume: 70,
  bringToFrontOnStorage: false,
  openExplorerOnStorage: false,
  checkUpdatesOnStart: true,
};

// Browser preview keeps settings in localStorage; the app keeps them in Rust.
const LS_KEY = "usblens.settings";

export async function getSettings(): Promise<Settings> {
  if (!inTauri) {
    try {
      return { ...defaultSettings, ...JSON.parse(localStorage.getItem(LS_KEY) ?? "{}") };
    } catch {
      return defaultSettings;
    }
  }
  return invoke<Settings>("get_settings");
}

export async function saveSettings(settings: Settings): Promise<Settings> {
  if (!inTauri) {
    try {
      localStorage.setItem(LS_KEY, JSON.stringify(settings));
    } catch {
      /* ignore */
    }
    return settings;
  }
  return invoke<Settings>("set_settings", { settings });
}

export async function checkUpdate(): Promise<UpdateInfo> {
  if (!inTauri) {
    return { current: "0.2.2", latest: "0.2.2", available: false, url: null, notes: null, date: null };
  }
  return invoke<UpdateInfo>("check_update");
}

export async function onOpenSettings(handler: () => void): Promise<() => void> {
  if (!inTauri) return () => {};
  const { listen } = await import("@tauri-apps/api/event");
  return listen("open-settings", () => handler());
}
