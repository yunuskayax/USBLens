import { useState } from "react";
import type { Lang, UsbDevice } from "../types";
import { dict, kindLabels } from "../i18n";
import { formatDate } from "../format";
import { IconCopy, IconEject, IconPower, IconTrash, IconX, KindIcon } from "./Icons";

interface Props {
  lang: Lang;
  device: UsbDevice;
  busy: boolean;
  onClose: () => void;
  onToggleEnabled: (d: UsbDevice) => void;
  onEject: (d: UsbDevice) => void;
  onUninstall: (d: UsbDevice) => void;
}

export function DetailPanel({ lang, device: d, busy, onClose, onToggleEnabled, onEject, onUninstall }: Props) {
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

  const rows: [string, string | null | undefined, boolean?][] = [
    [t.status, d.disabled ? t.disabledF : d.connected ? t.connected : t.history],
    [t.type, kindLabels[lang][d.kind]],
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
      </div>

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
