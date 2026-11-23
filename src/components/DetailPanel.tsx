import { useState } from "react";
import type { Lang, UsbDevice } from "../types";
import { dict, kindLabels } from "../i18n";
import { formatDate } from "../format";
import { IconCopy, IconEject, IconPower, IconShieldCheck, IconTrash, IconX, KindIcon } from "./Icons";
import { RiskBox } from "./Security";
import { brandOf, deviceLabel, detailsText, modelOf, nominalCapacity, usableCapacity } from "../label";

interface Props {
  lang: Lang;
  device: UsbDevice;
  busy: boolean;
  onClose: () => void;
  onToggleEnabled: (d: UsbDevice) => void;
  onEject: (d: UsbDevice) => void;
  onUninstall: (d: UsbDevice) => void;
  /** Root (physical) device used for allowlist identity; null when unknown. */
  root: UsbDevice | null;
  isAllowlisted: boolean;
  onToggleAllow: (root: UsbDevice) => void;
  copyTemplate: string;
}

export function DetailPanel({ lang, device: d, busy, onClose, onToggleEnabled, onEject, onUninstall, root, isAllowlisted, onToggleAllow, copyTemplate }: Props) {
  const t = dict[lang];
  const [copied, setCopied] = useState<string | null>(null);

  const copy = async (key: string, value: string) => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(key);
      setTimeout(() => setCopied(null), 1200);
    } catch {
      /* clipboard unavailable */
    }
  };

  const nominal = nominalCapacity(d.capacityBytes, lang);
  const usable = usableCapacity(d.capacityBytes, lang);
  const capacityText = nominal ? (usable && usable !== nominal ? `${nominal} (${usable} ${t.usable})` : nominal) : null;
  const isStorage = d.kind === "storage";
  const label = deviceLabel(d, copyTemplate, lang);

  const rows: [string, string | null | undefined, boolean?][] = [
    [t.status, d.disabled ? t.disabledF : d.connected ? t.connected : t.history],
    [t.type, kindLabels[lang][d.kind]],
    [t.brand, brandOf(d)],
    [t.model, modelOf(d)],
    [t.capacity, capacityText],
    [t.interfaces, [d.security.keyboard && t.keyboardCap, d.security.storage && t.storageCap].filter(Boolean).join(", ")],
    [t.allowlist, d.security.allowed == null ? null : d.security.allowed ? t.allowedTag : t.notAllowedTag],
    ["VID", d.vendorId],
    ["PID", d.productId],
    [t.serial, d.serialNumber, true],
    [t.drive, d.driveLetters.join(", ")],
    [t.lastArrival, formatDate(d.lastArrival, lang)],
    [t.lastRemoval, formatDate(d.lastRemoval, lang)],
    [t.firstInstall, formatDate(d.firstInstall, lang)],
    [t.description, d.description],
    [t.className, d.className],
    [t.service, d.service],
    [t.manufacturer, d.manufacturer],
    [t.driverVersion, d.driverVersion],
    [t.location, d.location],
    [t.problemCode, d.problemCode != null ? String(d.problemCode) : null],
    [t.parent, d.parentId, true],
    [t.instanceId, d.instanceId, true],
  ];

  return (
    <aside className="detail">
      <header className="detail-head">
        <span className={`kind-badge big k-${d.kind}`}>
          <KindIcon kind={d.kind} size={22} />
        </span>
        <div className="grow">
          <h2>{d.name}</h2>
          <span className={`pill ${d.disabled ? "warn" : d.connected ? "on" : "off"}`}>
            {d.disabled ? t.disabledF : d.connected ? t.connected : t.history}
          </span>
        </div>
        <button className="icon-btn" onClick={onClose} aria-label="close">
          <IconX />
        </button>
      </header>

      <RiskBox lang={lang} security={d.security} />

      <div className="label-box">
        <div className="label-head">
          <span className="side-title">{isStorage ? t.copyLabel : ""}</span>
          <button
            className="link muted"
            onClick={() => copy("__all", `${isStorage ? `${label}\n\n` : ""}${detailsText(rows.map(([k, v]) => [k, v ?? null]))}`)}
          >
            {copied === "__all" ? t.copied : t.copyAll}
          </button>
        </div>
        {isStorage && (
          <button className="label-text" onClick={() => copy("__label", label)} title={t.copy}>
            <span>{label}</span>
            {copied === "__label" ? <b className="copied-pill">{t.copied}</b> : <IconCopy size={15} />}
          </button>
        )}
      </div>

      <div className="actions">
        <button className="btn" disabled={busy} onClick={() => onToggleEnabled(d)}>
          <IconPower size={15} /> {d.disabled ? t.enable : t.disable}
        </button>
        <button className="btn" disabled={busy || !d.connected} onClick={() => onEject(d)}>
          <IconEject size={15} /> {t.eject}
        </button>
        <button className="btn danger" disabled={busy} onClick={() => onUninstall(d)}>
          <IconTrash size={15} /> {t.uninstall}
        </button>
        {root && root.vendorId && root.productId && (
          <button className={`btn ${isAllowlisted ? "" : "ok"}`} onClick={() => onToggleAllow(root)}>
            <IconShieldCheck size={15} /> {isAllowlisted ? t.removeFromAllowlist : t.addToAllowlist}
          </button>
        )}
      </div>

      {d.children.length > 0 && (
        <div className="children">
          <div className="side-title">{t.components}</div>
          {d.children.map((c) => (
            <div key={c.instanceId} className="child-row" title={c.instanceId}>
              <span className={`kind-badge k-${c.kind}`}>
                <KindIcon kind={c.kind} size={14} />
              </span>
              <div className="grow">
                <div>{c.name}</div>
                <div className="secondary mono">{c.className}{c.service && ` · ${c.service}`}</div>
              </div>
            </div>
          ))}
        </div>
      )}

      <dl className="props">
        {rows
          .filter(([, v]) => v != null && v !== "" && v !== "—")
          .map(([k, v, copyable]) => (
            <div key={k} className="prop">
              <dt>{k}</dt>
              <dd className={copyable ? "mono" : ""}>
                <span>{v}</span>
                {copyable && (
                  <button className="copy" title={t.copy} onClick={() => copy(k, String(v))}>
                    {copied === k ? t.copied : <IconCopy size={13} />}
                  </button>
                )}
              </dd>
            </div>
          ))}
      </dl>
    </aside>
  );
}
