import type { AllowEntry, ExportFormat, Lang, LogEvent, Settings, SystemInfo, UpdateInfo, UsbDevice, UsbEvent } from "./types";
import { mockDevices, mockEvents, mockSystem } from "./mock";

/** True inside the Tauri webview, false in a plain browser tab. */
export const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

export function allowMatches(e: AllowEntry, d: Pick<UsbDevice, "vendorId" | "productId" | "serialNumber">): boolean {
  if (!d.vendorId || !d.productId) return false;
  if (e.vendorId.toUpperCase() !== d.vendorId.toUpperCase() || e.productId.toUpperCase() !== d.productId.toUpperCase()) return false;
  if (!e.serialNumber) return true;
  return !!d.serialNumber && e.serialNumber.toUpperCase() === d.serialNumber.toUpperCase();
}

export async function listDevices(): Promise<UsbDevice[]> {
  if (!inTauri) {
    // Browser preview: approximate the Rust allowlist check so the UI can be tried out.
    const s = await getSettings();
    return structuredClone(mockDevices).map((d) => {
      const inScope = s.allowlistEnabled && d.kind !== "hub" && (s.allowlistScope === "all" || d.kind === "storage");
      if (!inScope) return d;
      const ok = s.allowlist.some((e) => allowMatches(e, d));
      const reasons = ok ? d.security.reasons : [...d.security.reasons, "NOT_ALLOWED"];
      return { ...d, security: { ...d.security, allowed: ok, reasons, level: ok ? d.security.level : "high" } };
    });
  }
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

/** Drive picked in Explorer ("UsbLens'te göster") before the window was ready. `?showDrive=E:` in a browser. */
export async function takePendingDrive(): Promise<string | null> {
  if (!inTauri) return new URLSearchParams(location.search).get("showDrive");
  return invoke<string | null>("take_pending_drive");
}

/** "UsbLens'te göster" while UsbLens is already running. */
export async function onSelectDrive(handler: (drive: string) => void): Promise<() => void> {
  if (!inTauri) return () => {};
  const { listen } = await import("@tauri-apps/api/event");
  return listen<string>("select-drive", (e) => handler(e.payload));
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

export const WEBSITE = "https://usblens.com";

export async function appVersion(): Promise<string> {
  if (!inTauri) return "0.3.2";
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
  showInterfaces: false,
  copyTemplate: "{marka} {kapasite} {tür} (S.No: {seri})",
  explorerContextMenu: true,
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
  lastSeenVersion: "",
  securityAlerts: true,
  newKeyboardAlert: true,
  eventLog: true,
  allowlistEnabled: false,
  allowlistAction: "warn",
  allowlistScope: "storage",
  allowlist: [],
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

// Browser preview only: `?mockUpdate` simulates an available update to test the UI.
const mockUpdate = typeof location !== "undefined" && new URLSearchParams(location.search).has("mockUpdate");
let mockProgress: ((d: number, t: number | null) => void) | null = null;

export async function checkUpdate(): Promise<UpdateInfo> {
  if (!inTauri && mockUpdate) {
    return {
      current: "0.3.2", latest: "0.3.3", available: true, date: "2026-11-20",
      url: "https://github.com/yunuskayax/USBLens/releases/latest",
      installer: "https://github.com/yunuskayax/USBLens/releases/download/v0.3.3/UsbLens-setup.exe",
      sha256: null, size: 2_100_000,
      notes: "### Yenilikler\n- **Yeni:** Uygulama içinden tek tıkla güncelleme\n- **Düzeltme:** Küçük hata düzeltmeleri\n\n### What's new\n- **New:** One-click in-app updates\n- **Fixed:** Small bug fixes",
    };
  }
  if (!inTauri) {
    return { current: "0.3.2", latest: "0.3.2", available: false, url: null, notes: null, date: null, installer: null, sha256: null, size: null };
  }
  return invoke<UpdateInfo>("check_update");
}

export async function onOpenSettings(handler: () => void): Promise<() => void> {
  if (!inTauri) return () => {};
  const { listen } = await import("@tauri-apps/api/event");
  return listen("open-settings", () => handler());
}

/** Download the new installer (progress via onUpdateProgress). Returns the local file path. */
export async function downloadUpdate(url: string, sha256: string | null): Promise<string> {
  if (!inTauri && mockUpdate) {
    for (let d = 0; d <= 2_100_000; d += 300_000) {
      mockProgress?.(d, 2_100_000);
      await new Promise((r) => setTimeout(r, 250));
    }
    return "mock";
  }
  if (!inTauri) throw "WINDOWS_ONLY";
  return invoke<string>("download_update", { url, sha256 });
}

/** Run the installer in passive mode; the app closes and restarts updated. */
export async function installUpdate(path: string): Promise<void> {
  if (!inTauri && mockUpdate) return new Promise(() => {}); // stays on "installing"
  if (!inTauri) throw "WINDOWS_ONLY";
  return invoke("install_update", { path });
}

export async function onUpdateProgress(handler: (downloaded: number, total: number | null) => void): Promise<() => void> {
  if (!inTauri) {
    mockProgress = handler;
    return () => (mockProgress = null);
  }
  const { listen } = await import("@tauri-apps/api/event");
  return listen<{ downloaded: number; total: number | null }>("update-progress", (e) => handler(e.payload.downloaded, e.payload.total));
}

// ---------------------------------------------------------------- event log

export async function listEvents(limit = 2000): Promise<LogEvent[]> {
  if (!inTauri) return structuredClone(mockEvents);
  return invoke<LogEvent[]>("list_events", { limit });
}

export async function clearEvents(): Promise<void> {
  if (!inTauri) {
    mockEvents.length = 0;
    return;
  }
  return invoke("clear_events");
}

export async function exportEvents(format: "csv" | "json", lang: Lang, computerName: string): Promise<string | null> {
  const stamp = new Date().toISOString().slice(0, 16).replace(/[:T]/g, "-");
  const fileName = `usblens-olaylar-${computerName || "pc"}-${stamp}.${format}`;
  if (!inTauri) return null;
  const { save } = await import("@tauri-apps/plugin-dialog");
  const path = await save({ defaultPath: fileName, filters: [{ name: format.toUpperCase(), extensions: [format] }] });
  if (!path) return null;
  await invoke("export_events", { path, format, lang });
  return path;
}
