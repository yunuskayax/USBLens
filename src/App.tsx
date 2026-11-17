import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as api from "./api";
import { dict, errorText, kindLabels, reasonText } from "./i18n";
import { vidPid } from "./format";
import type { ChangeEntry, LogEvent, DeviceKind, ExportFormat, Settings, SortKey, StatusFilter, SystemInfo, UpdateInfo, UsbDevice } from "./types";
import { Sidebar } from "./components/Sidebar";
import { DeviceTable } from "./components/DeviceTable";
import { DetailPanel } from "./components/DetailPanel";
import { Timeline } from "./components/Timeline";
import { ConfirmDialog, Toasts, type ConfirmState, type Toast } from "./components/Overlays";
import { AboutDialog } from "./components/About";
import { SettingsDialog, type Section } from "./components/SettingsDialog";
import { EventsView } from "./components/EventsView";
import { CHANGELOG, WhatsNewDialog, versionGt } from "./components/Changelog";
import { playUsbSound } from "./sound";
import { IconClock, IconDownload, IconGlobe, IconInfo, IconList, IconMoon, IconRefresh, IconSearch, IconSettings, IconShield, IconSun, IconTable } from "./components/Icons";

// localStorage can be missing or throw in some webview contexts.
function load<T>(key: string, fallback: T): T {
  try {
    const v = localStorage.getItem(`usblens.${key}`);
    return v == null ? fallback : (JSON.parse(v) as T);
  } catch {
    return fallback;
  }
}
function persist(key: string, value: unknown) {
  try {
    localStorage.setItem(`usblens.${key}`, JSON.stringify(value));
  } catch {
    /* ignore */
  }
}

const sortValue = (d: UsbDevice, key: SortKey): string => {
  switch (key) {
    case "name": return d.name.toLocaleLowerCase();
    case "kind": return d.kind;
    case "vidpid": return vidPid(d);
    case "serial": return d.serialNumber ?? "";
    case "drive": return d.driveLetters[0] ?? "";
    case "lastArrival": return d.lastArrival ?? "";
    case "lastRemoval": return d.lastRemoval ?? "";
    case "firstInstall": return d.firstInstall ?? "";
  }
};

export default function App() {
  // Cached copy for the first paint; the real settings come from Rust.
  const [settings, setSettings] = useState<Settings>(() => ({ ...api.defaultSettings, ...load<Partial<Settings>>("settingsCache", {}) }));
  const settingsRef = useRef(settings);
  settingsRef.current = settings;
  const saveTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const updateSettings = useCallback((patch: Partial<Settings>) => {
    setSettings((cur) => {
      const next = { ...cur, ...patch };
      clearTimeout(saveTimer.current);
      // Debounced so dragging the volume slider doesn't hammer the disk.
      saveTimer.current = setTimeout(() => {
        api.saveSettings(next).catch(() => {});
      }, 250);
      return next;
    });
  }, []);
  const { lang, theme, hideHubs } = settings;
  const setLang = (l: Settings["lang"]) => updateSettings({ lang: l });
  const setTheme = (th: Settings["theme"]) => updateSettings({ theme: th });
  const setHideHubs = (v: boolean) => updateSettings({ hideHubs: v });
  const t = dict[lang];

  const [devices, setDevices] = useState<UsbDevice[]>([]);
  const [sys, setSys] = useState<SystemInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [query, setQuery] = useState("");
  const [status, setStatus] = useState<StatusFilter>("all");
  const [kinds, setKinds] = useState<Set<DeviceKind>>(new Set());
  const [view, setView] = useState<"table" | "timeline" | "events">("table");
  const [events, setEvents] = useState<LogEvent[]>([]);
  const loadEvents = useCallback(() => {
    api.listEvents().then(setEvents).catch(() => {});
  }, []);
  const [sort, setSort] = useState<{ key: SortKey; asc: boolean }>({ key: "lastArrival", asc: false });
  const [selected, setSelected] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [confirm, setConfirm] = useState<ConfirmState | null>(null);
  const [flash, setFlash] = useState<Set<string>>(new Set());
  const [exportOpen, setExportOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [settingsInit, setSettingsInit] = useState<{ section?: Section; autoUpdate?: boolean }>({});
  const openSettings = useCallback((init: { section?: Section; autoUpdate?: boolean } = {}) => {
    setSettingsInit(init);
    setSettingsOpen(true);
  }, []);
  const closeSettings = useCallback(() => setSettingsOpen(false), []);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [whatsNew, setWhatsNew] = useState<{ entries: ChangeEntry[]; current: string } | null>(null);
  const closeWhatsNew = useCallback(() => setWhatsNew(null), []);
  const closeAbout = useCallback(() => setAboutOpen(false), []);
  const searchRef = useRef<HTMLInputElement>(null);
  const langRef = useRef(lang);
  langRef.current = lang;

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    document.documentElement.lang = lang;
    persist("settingsCache", settings);
  }, [theme, lang, settings]);

  const toast = useCallback((x: Omit<Toast, "id">) => {
    setToasts((ts) => [...ts.slice(-3), { ...x, id: Date.now() + Math.random() }]);
  }, []);
  const dismiss = useCallback((id: number) => setToasts((ts) => ts.filter((x) => x.id !== id)), []);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      setDevices(await api.listDevices());
    } catch (e) {
      toast({ kind: "error", title: errorText(langRef.current, e) });
    } finally {
      setLoading(false);
    }
  }, [toast]);

  // Initial load + system info.
  useEffect(() => {
    refresh();
    api.systemInfo().then(setSys).catch(() => {});
  }, [refresh]);

  // Load settings from Rust, then (optionally) check for updates once.
  useEffect(() => {
    let timer: ReturnType<typeof setTimeout> | undefined;
    api
      .getSettings()
      .then(async (s) => {
        setSettings(s);
        // "What's new" after an update: show the entries between the last seen and the running version.
        try {
          const current = await api.appVersion();
          if (s.lastSeenVersion && s.lastSeenVersion !== current) {
            const entries =
              s.lastSeenVersion === "legacy"
                ? CHANGELOG.filter((e) => e.version === current)
                : CHANGELOG.filter((e) => versionGt(e.version, s.lastSeenVersion) && !versionGt(e.version, current));
            if (entries.length) setWhatsNew({ entries, current });
          }
          if (s.lastSeenVersion !== current) updateSettings({ lastSeenVersion: current });
        } catch {
          /* ignore */
        }
        if (!s.checkUpdatesOnStart) return;
        timer = setTimeout(async () => {
          try {
            const info = await api.checkUpdate();
            setUpdateInfo(info);
            if (info.available) {
              const l = dict[settingsRef.current.lang];
              toast({
                kind: "info",
                title: `${l.updateAvailable}: ${info.latest}`,
                body: info.installer ? l.updateHint : undefined,
                action: info.installer
                  ? { label: l.updateNow, onClick: () => openSettings({ section: "updates", autoUpdate: true }) }
                  : info.url
                    ? { label: l.download, onClick: () => api.openUrl(info.url!) }
                    : undefined,
              });
            }
          } catch {
            /* silent on startup */
          }
        }, 3000);
      })
      .catch(() => {});
    const unlisten = api.onOpenSettings(() => openSettings());
    return () => {
      clearTimeout(timer);
      unlisten.then((fn) => fn());
    };
  }, [toast, updateSettings]);

  // Security settings change the analysis done in Rust: re-read the list after they are saved.
  const securityKey = JSON.stringify([
    settings.securityAlerts, settings.newKeyboardAlert, settings.allowlistEnabled,
    settings.allowlistAction, settings.allowlistScope, settings.allowlist, settings.showInterfaces,
  ]);
  const firstSecurityKey = useRef(true);
  useEffect(() => {
    if (firstSecurityKey.current) {
      firstSecurityKey.current = false;
      return;
    }
    const h = setTimeout(refresh, 500);
    return () => clearTimeout(h);
  }, [securityKey, refresh]);

  useEffect(() => {
    loadEvents();
  }, [loadEvents]);

  // Live USB events from the Rust side.
  useEffect(() => {
    let timer: ReturnType<typeof setTimeout> | undefined;
    const unlisten = api.onUsbEvent((ev) => {
      const st = settingsRef.current;
      const l = dict[st.lang];
      const name = ev.device?.name ?? ev.instanceId ?? "USB";
      const arrival = ev.action === "arrival";
      const extra = [ev.device && vidPid(ev.device), ev.device?.serialNumber, ev.device?.driveLetters.join(" ")]
        .filter(Boolean)
        .join(" · ");
      const serious = ev.level === "high" || ev.level === "medium";
      if (ev.alerts.length) {
        const title = ev.blocked ? `⛔ ${l.blocked}: ${name}` : `⚠ ${l.securityToastTitle}: ${name}`;
        const lines = ev.alerts.map((a) => reasonText(st.lang, a));
        if (ev.blockError) lines.push(`${l.blockFailed}: ${errorText(st.lang, ev.blockError)}`);
        toast({ kind: serious ? "error" : "info", title, body: lines.join("\n") });
      } else {
        toast({ kind: arrival ? "in" : "out", title: `${name} ${arrival ? l.plugged : l.unplugged}`, body: extra || undefined });
      }
      if ((arrival ? st.soundOnArrival : st.soundOnRemoval) && ev.device) {
        playUsbSound({ arrival, style: st.soundStyle, volume: st.soundVolume, lang: st.lang, name, storage: ev.device.kind === "storage" });
      }
      // Serious security alerts are already sent as Windows notifications by the Rust side.
      const wantNotify = st.systemNotifications && (arrival || st.notifyOnRemoval) && !serious;
      if (wantNotify && (document.hidden || !document.hasFocus())) {
        api.systemNotify(`UsbLens — ${arrival ? l.arrived : l.removed}`, `${name}${extra ? `\n${extra}` : ""}`);
      }
      if (ev.instanceId) {
        const id = ev.instanceId;
        setFlash((f) => new Set(f).add(id));
        setTimeout(() => setFlash((f) => { const n = new Set(f); n.delete(id); return n; }), 3000);
      }
      // Debounced refresh; drive letters often appear a moment later.
      clearTimeout(timer);
      timer = setTimeout(refresh, 1200);
      loadEvents();
    });
    return () => {
      clearTimeout(timer);
      unlisten.then((fn) => fn());
    };
  }, [refresh, toast, loadEvents]);

  // Keyboard: Ctrl+F focuses search, F5 refreshes.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "f") {
        e.preventDefault();
        searchRef.current?.focus();
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "b") {
        e.preventDefault();
        updateSettings({ sidebarCollapsed: !settingsRef.current.sidebarCollapsed });
      } else if (e.key === "F5") {
        e.preventDefault();
        refresh();
      } else if (e.key === "Escape" && !confirm && !aboutOpen && !settingsOpen && !whatsNew) {
        setSelected(null);
        setExportOpen(false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [refresh, confirm, aboutOpen, settingsOpen, whatsNew, updateSettings]);

  const filtered = useMemo(() => {
    const q = query.trim().toLocaleLowerCase(lang);
    const list = devices.filter((d) => {
      if (hideHubs && d.kind === "hub") return false;
      if (status === "connected" && !d.connected) return false;
      if (status === "history" && d.connected) return false;
      if (status === "disabled" && !d.disabled) return false;
      if (status === "alerts" && d.security.level !== "high" && d.security.level !== "medium") return false;
      if (kinds.size && !kinds.has(d.kind)) return false;
      if (!q) return true;
      const hay = [d.name, d.description, d.serialNumber, vidPid(d), d.instanceId, d.manufacturer, d.className, kindLabels[lang][d.kind], ...d.driveLetters]
        .filter(Boolean)
        .join(" ")
        .toLocaleLowerCase(lang);
      return q.split(/\s+/).every((part) => hay.includes(part));
    });
    const dir = sort.asc ? 1 : -1;
    return list.sort((a, b) => {
      const av = sortValue(a, sort.key);
      const bv = sortValue(b, sort.key);
      // Empty values always last.
      if (!av && bv) return 1;
      if (av && !bv) return -1;
      return av.localeCompare(bv, lang) * dir;
    });
  }, [devices, query, status, kinds, hideHubs, sort, lang]);

  const selectedDevice = devices.find((d) => d.instanceId === selected) ?? null;
  const selectedRoot = selectedDevice ? (devices.find((d) => d.instanceId === selectedDevice.security.rootId) ?? selectedDevice) : null;
  const toggleAllow = (root: UsbDevice) => {
    const list = settingsRef.current.allowlist;
    const matches = list.filter((e) => api.allowMatches(e, root));
    if (matches.length) {
      updateSettings({ allowlist: list.filter((e) => !matches.includes(e)) });
    } else if (root.vendorId && root.productId) {
      const addedAt = new Date().toISOString().slice(0, 19) + "Z";
      updateSettings({
        allowlist: [...list, { vendorId: root.vendorId, productId: root.productId, serialNumber: root.serialNumber, name: root.name, addedAt }],
      });
    }
  };

  const run = async (fn: () => Promise<void>) => {
    setBusy(true);
    try {
      await fn();
      toast({ kind: "ok", title: t.done });
      await refresh();
    } catch (e) {
      toast({ kind: "error", title: errorText(lang, e) });
    } finally {
      setBusy(false);
    }
  };

  const onToggleEnabled = (d: UsbDevice) => {
    if (d.disabled) return run(() => api.setDeviceEnabled(d.instanceId, true));
    setConfirm({
      title: t.disableTitle,
      body: `${d.name}\n\n${t.disableBody}`,
      danger: true,
      onConfirm: () => run(() => api.setDeviceEnabled(d.instanceId, false)),
    });
  };
  const onEject = (d: UsbDevice) => run(() => api.ejectDevice(d.instanceId));
  const onUninstall = (d: UsbDevice) =>
    setConfirm({
      title: t.uninstallTitle,
      body: `${d.name}\n\n${t.uninstallBody}`,
      danger: true,
      onConfirm: () =>
        run(async () => {
          await api.uninstallDevice(d.instanceId);
          setSelected(null);
        }),
    });

  const doExport = async (format: ExportFormat) => {
    setExportOpen(false);
    try {
      const path = await api.exportDevices(format, lang, filtered, sys?.computerName ?? "");
      if (path) toast({ kind: "ok", title: t.exported, body: path });
    } catch (e) {
      toast({ kind: "error", title: errorText(lang, e) });
    }
  };

  const toggleKind = (k: DeviceKind) =>
    setKinds((s) => {
      const n = new Set(s);
      if (n.has(k)) n.delete(k);
      else n.add(k);
      return n;
    });

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <img src="/logo.png" alt="" width={26} height={26} />
          <span>UsbLens</span>
        </div>

        <div className="search">
          <IconSearch />
          <input ref={searchRef} value={query} onChange={(e) => setQuery(e.target.value)} placeholder={t.search} spellCheck={false} />
          {query && <button className="clear" onClick={() => setQuery("")}>×</button>}
        </div>

        <div className="segmented">
          <button className={view === "table" ? "on" : ""} onClick={() => setView("table")}>
            <IconTable size={15} /> {t.table}
          </button>
          <button className={view === "timeline" ? "on" : ""} onClick={() => setView("timeline")}>
            <IconClock size={15} /> {t.timeline}
          </button>
          <button
            className={view === "events" ? "on" : ""}
            onClick={() => {
              setView("events");
              loadEvents();
            }}
          >
            <IconList size={15} /> {t.events}
            {events.some((e) => e.level === "high") && <span className="badge-dot" />}
          </button>
        </div>

        <div className="top-actions">
          <button className="btn" onClick={refresh} disabled={loading} title="F5">
            <IconRefresh size={15} /> <span className="hide-sm">{t.refresh}</span>
          </button>
          <div className="menu-wrap">
            <button className="btn primary" onClick={() => setExportOpen((o) => !o)}>
              <IconDownload size={15} /> <span className="hide-sm">{t.export}</span>
            </button>
            {exportOpen && (
              <div className="menu" onMouseLeave={() => setExportOpen(false)}>
                <button onClick={() => doExport("html")}>{t.exportHtml}</button>
                <button onClick={() => doExport("csv")}>{t.exportCsv}</button>
                <button onClick={() => doExport("json")}>{t.exportJson}</button>
              </div>
            )}
          </div>
          <button className="icon-btn" title={t.theme} onClick={() => setTheme(theme === "dark" ? "light" : "dark")}>
            {theme === "dark" ? <IconSun /> : <IconMoon />}
          </button>
          <button className="icon-btn lang" title="Dil / Language" onClick={() => setLang(lang === "tr" ? "en" : "tr")}>
            <IconGlobe size={15} /> {lang.toUpperCase()}
          </button>
          <button className="icon-btn badge-wrap" title={t.settings} onClick={() => openSettings()}>
            <IconSettings />
            {updateInfo?.available && <span className="badge-dot" />}
          </button>
          <button className="icon-btn" title={t.about} onClick={() => setAboutOpen(true)}>
            <IconInfo />
          </button>
        </div>
      </header>

      {sys && !sys.isWindows && <div className="banner">{t.demo}</div>}

      <div className="body">
        <Sidebar
          lang={lang}
          devices={devices}
          status={status}
          setStatus={setStatus}
          kinds={kinds}
          toggleKind={toggleKind}
          hideHubs={hideHubs}
          setHideHubs={setHideHubs}
          collapsed={settings.sidebarCollapsed}
          setCollapsed={(v) => updateSettings({ sidebarCollapsed: v })}
        />

        <main className="content">
          {loading && devices.length === 0 ? (
            <div className="empty">{t.loading}</div>
          ) : view === "events" ? (
            <EventsView
              lang={lang}
              events={events}
              enabled={settings.eventLog}
              query={query}
              onSelect={(id) => setSelected(devices.some((d) => d.instanceId === id) ? id : null)}
              onExport={async (format) => {
                try {
                  const path = await api.exportEvents(format, lang, sys?.computerName ?? "");
                  if (path) toast({ kind: "ok", title: t.exported, body: path });
                } catch (e) {
                  toast({ kind: "error", title: errorText(lang, e) });
                }
              }}
              onClear={() =>
                setConfirm({
                  title: t.clearLogTitle,
                  body: t.clearLogBody,
                  danger: true,
                  onConfirm: () => api.clearEvents().then(loadEvents).catch((e) => toast({ kind: "error", title: errorText(lang, e) })),
                })
              }
            />
          ) : view === "table" ? (
            <DeviceTable lang={lang} devices={filtered} selected={selected} onSelect={setSelected} sort={sort} setSort={setSort} flash={flash} />
          ) : (
            <Timeline lang={lang} devices={filtered} onSelect={setSelected} />
          )}
        </main>

        {selectedDevice && (
          <DetailPanel
            lang={lang}
            device={selectedDevice}
            busy={busy}
            onClose={() => setSelected(null)}
            onToggleEnabled={onToggleEnabled}
            onEject={onEject}
            onUninstall={onUninstall}
            root={selectedRoot}
            isAllowlisted={!!selectedRoot && settings.allowlist.some((e) => api.allowMatches(e, selectedRoot))}
            onToggleAllow={toggleAllow}
          />
        )}
      </div>

      <footer className="statusbar">
        <span>
          {filtered.length} / {devices.length} {t.devices}
        </span>
        <span className="grow" />
        {updateInfo?.available && (
          <button className="link" onClick={() => openSettings({ section: "updates" })}>
            {t.updateAvailable}: {updateInfo.latest}
          </button>
        )}
        <button className="link muted" onClick={() => openSettings()}>
          {t.settings}
        </button>
        <button className="link muted" onClick={() => setAboutOpen(true)}>
          {t.about}
        </button>
        {sys && (
          <>
            <span>{sys.computerName}</span>
            {sys.elevated ? (
              <span className="admin on"><IconShield size={13} /> {t.admin}</span>
            ) : (
              <button
                className="admin"
                onClick={() => api.relaunchAsAdmin().catch((e) => toast({ kind: "error", title: errorText(lang, e) }))}
                title={t.runAsAdmin}
              >
                <IconShield size={13} /> {t.notAdmin} — {t.runAsAdmin}
              </button>
            )}
          </>
        )}
      </footer>

      <Toasts toasts={toasts} dismiss={dismiss} />
      {settingsOpen && (
        <SettingsDialog
          devices={devices}
          settings={settings}
          update={updateSettings}
          close={closeSettings}
          updateInfo={updateInfo}
          setUpdateInfo={setUpdateInfo}
          initialSection={settingsInit.section}
          autoUpdate={settingsInit.autoUpdate}
        />
      )}
      {aboutOpen && (
        <AboutDialog
          lang={lang}
          close={closeAbout}
          showNotes={() => {
            setAboutOpen(false);
            api.appVersion().then((v) => setWhatsNew({ entries: CHANGELOG, current: v })).catch(() => {});
          }}
        />
      )}
      {whatsNew && <WhatsNewDialog lang={lang} entries={whatsNew.entries} current={whatsNew.current} close={closeWhatsNew} />}
      {confirm && <ConfirmDialog lang={lang} state={confirm} close={() => setConfirm(null)} />}
    </div>
  );
}
