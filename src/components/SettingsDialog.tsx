import { useEffect, useState } from "react";
import type { AllowEntry, Settings, UpdateInfo, UsbDevice } from "../types";
import { dict, errorText } from "../i18n";
import * as api from "../api";
import { playUsbSound } from "../sound";
import { CHANGELOG, ChangelogList, RemoteNotes } from "./Changelog";
import { TEMPLATE_TOKENS, deviceLabel } from "../label";
import { IconBell, IconDownload, IconGlobe, IconRefresh, IconShieldAlert, IconTrash, IconWindow, IconX, KindIcon } from "./Icons";

export type Section = "general" | "window" | "notifications" | "storage" | "security" | "updates";

interface Props {
  devices: UsbDevice[];
  settings: Settings;
  update: (patch: Partial<Settings>) => void;
  close: () => void;
  updateInfo: UpdateInfo | null;
  setUpdateInfo: (u: UpdateInfo | null) => void;
  initialSection?: Section;
  /** Start downloading the update right away (from the "Update" toast). */
  autoUpdate?: boolean;
}

type UpdatePhase =
  | { kind: "idle" }
  | { kind: "downloading"; done: number; total: number | null }
  | { kind: "installing" };

function Toggle({ label, desc, checked, onChange, disabled }: {
  label: string;
  desc?: string;
  checked: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <label className={`setting ${disabled ? "disabled" : ""}`}>
      <span className="grow">
        <span className="setting-label">{label}</span>
        {desc && <span className="setting-desc">{desc}</span>}
      </span>
      <span className={`switch ${checked ? "on" : ""}`}>
        <input type="checkbox" checked={checked} disabled={disabled} onChange={(e) => onChange(e.target.checked)} />
        <span className="knob" />
      </span>
    </label>
  );
}

function Choice<T extends string>({ value, options, onChange }: {
  value: T;
  options: { value: T; label: string }[];
  onChange: (v: T) => void;
}) {
  return (
    <div className="segmented small">
      {options.map((o) => (
        <button key={o.value} className={value === o.value ? "on" : ""} onClick={() => onChange(o.value)}>
          {o.label}
        </button>
      ))}
    </div>
  );
}

export function SettingsDialog({ devices, settings: s, update, close, updateInfo, setUpdateInfo, initialSection, autoUpdate }: Props) {
  const t = dict[s.lang];
  const [section, setSection] = useState<Section>(initialSection ?? "general");
  const sample =
    devices.find((d) => d.kind === "storage" && d.connected && d.capacityBytes) ??
    devices.find((d) => d.kind === "storage") ??
    devices[0];
  const previewLabel = sample ? deviceLabel(sample, s.copyTemplate, s.lang) : "—";
  const [phase, setPhase] = useState<UpdatePhase>({ kind: "idle" });
  const busyUpdating = phase.kind !== "idle";

  const startUpdate = async (info: UpdateInfo) => {
    if (!info.installer) return;
    setUpdateError(null);
    setErrorDetail(null);
    setPhase({ kind: "downloading", done: 0, total: info.size });
    const unlisten = await api.onUpdateProgress((done, total) => setPhase({ kind: "downloading", done, total: total ?? info.size }));
    try {
      const path = await api.downloadUpdate(info.installer, info.sha256);
      setPhase({ kind: "installing" });
      await api.installUpdate(path);
    } catch (e) {
      setPhase({ kind: "idle" });
      fail(e);
    } finally {
      unlisten();
    }
  };

  useEffect(() => {
    if (autoUpdate && updateInfo?.available && updateInfo.installer) void startUpdate(updateInfo);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  const [checking, setChecking] = useState(false);
  const [updateError, setUpdateError] = useState<string | null>(null);
  const [errorDetail, setErrorDetail] = useState<string | null>(null);
  const fail = (e: unknown) => {
    const raw = String(e ?? "");
    setUpdateError(errorText(s.lang, raw));
    setErrorDetail(raw.startsWith("UPDATE_FAILED:") ? raw.slice("UPDATE_FAILED:".length) : null);
  };
  const [version, setVersion] = useState("");

  useEffect(() => {
    api.appVersion().then(setVersion).catch(() => {});
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && close();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [close]);

  const checkNow = async () => {
    setChecking(true);
    setUpdateError(null);
    setErrorDetail(null);
    try {
      setUpdateInfo(await api.checkUpdate());
    } catch (e) {
      fail(e);
    } finally {
      setChecking(false);
    }
  };

  const testSound = (arrival: boolean) =>
    playUsbSound({ arrival, style: s.soundStyle, volume: s.soundVolume, lang: s.lang, name: "SanDisk Cruzer Blade", storage: true });

  const nav: { key: Section; label: string; icon: React.ReactNode }[] = [
    { key: "general", label: t.sGeneral, icon: <IconGlobe size={16} /> },
    { key: "window", label: t.sWindow, icon: <IconWindow size={16} /> },
    { key: "notifications", label: t.sNotifications, icon: <IconBell size={16} /> },
    { key: "storage", label: t.sStorage, icon: <KindIcon kind="storage" size={16} /> },
    { key: "security", label: t.sSecurity, icon: <IconShieldAlert size={16} /> },
    { key: "updates", label: t.sUpdates, icon: <IconRefresh size={16} /> },
  ];

  return (
    <div className="modal-bg" onClick={close}>
      <div className="modal settings" onClick={(e) => e.stopPropagation()} role="dialog" aria-modal aria-label={t.settings}>
        <nav className="settings-nav">
          <h3>{t.settings}</h3>
          {nav.map((n) => (
            <button key={n.key} className={`side-item ${section === n.key ? "active" : ""}`} onClick={() => setSection(n.key)}>
              {n.icon}
              <span className="grow">{n.label}</span>
              {n.key === "updates" && updateInfo?.available && <span className="badge-dot" />}
            </button>
          ))}
        </nav>

        <section className="settings-body">
          <header className="settings-head">
            <h2>{nav.find((n) => n.key === section)?.label}</h2>
            <button className="icon-btn" onClick={close} aria-label={t.close}>
              <IconX />
            </button>
          </header>

          {section === "general" && (
            <div className="settings-list">
              <div className="setting">
                <span className="grow"><span className="setting-label">{t.language}</span></span>
                <Choice value={s.lang} onChange={(lang) => update({ lang })} options={[{ value: "tr", label: "Türkçe" }, { value: "en", label: "English" }]} />
              </div>
              <div className="setting">
                <span className="grow"><span className="setting-label">{t.theme}</span></span>
                <Choice value={s.theme} onChange={(theme) => update({ theme })} options={[{ value: "dark", label: t.themeDark }, { value: "light", label: t.themeLight }]} />
              </div>
              <Toggle label={t.hideHubs} desc={t.hideHubsDesc} checked={s.hideHubs} onChange={(hideHubs) => update({ hideHubs })} />
              <Toggle label={t.showInterfaces} desc={t.showInterfacesDesc} checked={s.showInterfaces} onChange={(showInterfaces) => update({ showInterfaces })} />
            </div>
          )}

          {section === "window" && (
            <div className="settings-list">
              <Toggle label={t.alwaysOnTop} desc={t.alwaysOnTopDesc} checked={s.alwaysOnTop} onChange={(alwaysOnTop) => update({ alwaysOnTop })} />
              <Toggle label={t.closeToTray} desc={t.closeToTrayDesc} checked={s.closeToTray} onChange={(closeToTray) => update({ closeToTray })} />
              <Toggle label={t.startWithWindows} desc={t.startWithWindowsDesc} checked={s.startWithWindows} onChange={(startWithWindows) => update({ startWithWindows })} />
              <Toggle label={t.startMinimized} desc={t.startMinimizedDesc} checked={s.startMinimized} onChange={(startMinimized) => update({ startMinimized })} />
            </div>
          )}

          {section === "notifications" && (
            <div className="settings-list">
              <Toggle label={t.systemNotifications} desc={t.systemNotificationsDesc} checked={s.systemNotifications} onChange={(systemNotifications) => update({ systemNotifications })} />
              <Toggle label={t.notifyOnRemoval} checked={s.notifyOnRemoval} disabled={!s.systemNotifications} onChange={(notifyOnRemoval) => update({ notifyOnRemoval })} />
              <Toggle label={t.soundOnArrival} checked={s.soundOnArrival} onChange={(soundOnArrival) => update({ soundOnArrival })} />
              <Toggle label={t.soundOnRemoval} checked={s.soundOnRemoval} onChange={(soundOnRemoval) => update({ soundOnRemoval })} />
              <div className={`setting col ${s.soundOnArrival || s.soundOnRemoval ? "" : "disabled"}`}>
                <span className="setting-label">{t.soundStyle}</span>
                <div className="radio-row">
                  {(["chime", "voice"] as const).map((v) => (
                    <label key={v} className="radio">
                      <input type="radio" name="soundStyle" checked={s.soundStyle === v} onChange={() => update({ soundStyle: v })} />
                      {v === "chime" ? t.soundChime : t.soundVoice}
                    </label>
                  ))}
                </div>
                <div className="volume-row">
                  <span className="setting-desc">{t.volume}</span>
                  <input type="range" min={0} max={100} step={5} value={s.soundVolume} onChange={(e) => update({ soundVolume: Number(e.target.value) })} />
                  <span className="mono vol">{s.soundVolume}%</span>
                  <button className="btn" onClick={() => testSound(true)}>{t.test} ▲</button>
                  <button className="btn" onClick={() => testSound(false)}>{t.test} ▼</button>
                </div>
              </div>
            </div>
          )}

          {section === "security" && (
            <div className="settings-list">
              <Toggle label={t.securityAlerts} desc={t.securityAlertsDesc} checked={s.securityAlerts} onChange={(securityAlerts) => update({ securityAlerts })} />
              <Toggle label={t.newKeyboardAlert} desc={t.newKeyboardAlertDesc} checked={s.newKeyboardAlert} onChange={(newKeyboardAlert) => update({ newKeyboardAlert })} />
              <Toggle label={t.eventLog} desc={t.eventLogDesc} checked={s.eventLog} onChange={(eventLog) => update({ eventLog })} />

              <h3 className="settings-sub">{t.allowlist}</h3>
              <Toggle label={t.allowlistEnabled} desc={t.allowlistDesc} checked={s.allowlistEnabled} onChange={(allowlistEnabled) => update({ allowlistEnabled })} />
              <div className={`setting ${s.allowlistEnabled ? "" : "disabled"}`}>
                <span className="grow">
                  <span className="setting-label">{t.allowlistAction}</span>
                  {s.allowlistAction === "block" && <span className="setting-desc">{t.blockNeedsAdmin}</span>}
                </span>
                <Choice value={s.allowlistAction} onChange={(allowlistAction) => update({ allowlistAction })} options={[{ value: "warn", label: t.actionWarn }, { value: "block", label: t.actionBlock }]} />
              </div>
              <div className={`setting ${s.allowlistEnabled ? "" : "disabled"}`}>
                <span className="grow"><span className="setting-label">{t.allowlistScope}</span></span>
                <Choice value={s.allowlistScope} onChange={(allowlistScope) => update({ allowlistScope })} options={[{ value: "storage", label: t.scopeStorage }, { value: "all", label: t.scopeAll }]} />
              </div>
              {s.allowlistEnabled && s.allowlistScope === "all" && s.allowlistAction === "block" && <p className="settings-warn">{t.scopeAllWarn}</p>}

              <div className="allow-list">
                {s.allowlist.length === 0 ? (
                  <p className="setting-desc">{t.allowlistEmpty}</p>
                ) : (
                  s.allowlist.map((e, i) => (
                    <div key={`${e.vendorId}:${e.productId}:${e.serialNumber ?? "*"}`} className="allow-row">
                      <div className="grow">
                        <div className="setting-label">{e.name || `${e.vendorId}:${e.productId}`}</div>
                        <div className="setting-desc mono">
                          {e.vendorId}:{e.productId} · {e.serialNumber ?? t.anySerial}
                        </div>
                      </div>
                      <button className="btn" onClick={() => update({ allowlist: s.allowlist.filter((_, j) => j !== i) })}>
                        <IconTrash size={14} /> {t.remove}
                      </button>
                    </div>
                  ))
                )}
              </div>
              <div className="update-actions" style={{ marginTop: 12 }}>
                <button className="btn" onClick={() => update({ allowlist: addConnected(devices, s) })}>
                  {t.addConnected}
                </button>
              </div>
            </div>
          )}

          {section === "storage" && (
            <div className="settings-list">
              <p className="settings-note">{t.sStorageNote}</p>
              <Toggle label={t.bringToFront} desc={t.bringToFrontDesc} checked={s.bringToFrontOnStorage} onChange={(bringToFrontOnStorage) => update({ bringToFrontOnStorage })} />
              <Toggle label={t.openExplorer} desc={t.openExplorerDesc} checked={s.openExplorerOnStorage} onChange={(openExplorerOnStorage) => update({ openExplorerOnStorage })} />

              <Toggle label={t.contextMenu} desc={t.contextMenuDesc} checked={s.explorerContextMenu} onChange={(explorerContextMenu) => update({ explorerContextMenu })} />

              <h3 className="settings-sub">{t.copyTemplate}</h3>
              <p className="setting-desc" style={{ margin: "0 0 10px" }}>{t.copyTemplateDesc}</p>
              <input
                className="template-input"
                value={s.copyTemplate}
                spellCheck={false}
                onChange={(e) => update({ copyTemplate: e.target.value })}
              />
              <div className="setting col" style={{ borderBottom: 0, cursor: "default" }}>
                <span className="setting-desc">{t.tokens}</span>
                <div className="token-row">
                  {TEMPLATE_TOKENS.map((tok) => (
                    <button key={tok} className="token" onClick={() => update({ copyTemplate: `${s.copyTemplate.trimEnd()} ${tok}`.trim() })}>
                      {tok}
                    </button>
                  ))}
                  <button className="link muted" onClick={() => update({ copyTemplate: api.defaultSettings.copyTemplate })}>
                    {t.resetTemplate}
                  </button>
                </div>
                <span className="setting-desc">{t.preview}</span>
                <div className="template-preview">{previewLabel}</div>
              </div>
            </div>
          )}

          {section === "updates" && (
            <div className="settings-list">
              <Toggle label={t.checkOnStart} checked={s.checkUpdatesOnStart} onChange={(checkUpdatesOnStart) => update({ checkUpdatesOnStart })} />
              <div className="update-card">
                <div className="update-rows">
                  <div><span className="setting-desc">{t.currentVersion}</span><b>{updateInfo?.current ?? version}</b></div>
                  {updateInfo && <div><span className="setting-desc">{t.latestVersion}</span><b>{updateInfo.latest}</b></div>}
                </div>
                {updateInfo && !checking && (
                  <div className={`update-status ${updateInfo.available ? "new" : "ok"}`}>
                    {updateInfo.available ? `${t.updateAvailable}: ${updateInfo.latest}` : t.upToDate}
                    {updateInfo.date && <span className="setting-desc"> · {updateInfo.date}</span>}
                  </div>
                )}
                {updateInfo?.available && updateInfo.notes && (
                  <div className="update-remote">
                    <div className="setting-label">
                      {t.inNewVersion} · v{updateInfo.latest}
                    </div>
                    <RemoteNotes lang={s.lang} text={updateInfo.notes} />
                  </div>
                )}
                {phase.kind === "downloading" && (
                  <div className="update-progress">
                    <div className="bar">
                      <span style={{ width: `${phase.total ? Math.min(100, (phase.done / phase.total) * 100) : 30}%` }} className={phase.total ? "" : "indeterminate"} />
                    </div>
                    <span className="setting-desc">
                      {t.downloadingUpdate} · {(phase.done / 1048576).toFixed(1)}
                      {phase.total ? ` / ${(phase.total / 1048576).toFixed(1)}` : ""} MB
                      {phase.total ? ` · ${Math.floor((phase.done / phase.total) * 100)}%` : ""}
                    </span>
                  </div>
                )}
                {phase.kind === "installing" && <div className="update-status new">{t.installingUpdate}</div>}
                {updateError && <div className="update-status err">{updateError}</div>}
                {updateError && errorDetail && (
                  <details className="err-detail">
                    <summary>{t.errorDetail}</summary>
                    <code>{errorDetail}</code>
                  </details>
                )}
                <div className="update-actions">
                  {updateInfo?.available && updateInfo.installer ? (
                    <button className="btn primary" onClick={() => startUpdate(updateInfo)} disabled={busyUpdating}>
                      <IconDownload size={15} /> {updateError ? t.retry : t.updateNow}
                    </button>
                  ) : (
                    updateInfo?.available &&
                    updateInfo.url && (
                      <button className="btn primary" onClick={() => api.openUrl(updateInfo.url!)}>
                        <IconDownload size={15} /> {t.download}
                      </button>
                    )
                  )}
                  <button className="btn" onClick={checkNow} disabled={checking || busyUpdating}>
                    <IconRefresh size={15} /> {checking ? t.checking : t.checkNow}
                  </button>
                  {updateInfo?.available && updateInfo.installer && updateInfo.url && (
                    <button className="link muted" onClick={() => api.openUrl(updateInfo.url!)} disabled={busyUpdating}>
                      {t.openInBrowser}
                    </button>
                  )}
                </div>
                {updateInfo?.available && updateInfo.installer && phase.kind === "idle" && !updateError && (
                  <span className="setting-desc">{t.updateHint}</span>
                )}
              </div>
              <h3 className="settings-sub">{t.releaseNotes}</h3>
              <ChangelogList lang={s.lang} entries={CHANGELOG} current={updateInfo?.current ?? version} />
            </div>
          )}
        </section>
      </div>
    </div>
  );
}

/** Physical, currently connected devices that are in allowlist scope and not yet listed. */
function addConnected(devices: UsbDevice[], s: Settings): AllowEntry[] {
  const now = new Date().toISOString().slice(0, 19) + "Z";
  const out = [...s.allowlist];
  for (const d of devices) {
    const isRoot = d.instanceId === d.security.rootId;
    if (!d.connected || !isRoot || d.kind === "hub" || !d.vendorId || !d.productId) continue;
    if (!d.instanceId.toUpperCase().startsWith("USB\\")) continue;
    if (s.allowlistScope === "storage" && !d.security.storage) continue;
    // Never trust a suspicious device in bulk; it can still be added on purpose from its details.
    if (d.security.reasons.some((r) => r !== "NOT_ALLOWED" && !r.startsWith("PROGRAMMABLE_BOARD"))) continue;
    if (d.security.reasons.some((r) => r.startsWith("PROGRAMMABLE_BOARD")) && d.security.keyboard) continue;
    if (out.some((e) => api.allowMatches(e, d))) continue;
    out.push({ vendorId: d.vendorId, productId: d.productId, serialNumber: d.serialNumber, name: d.name, addedAt: now });
  }
  return out;
}
