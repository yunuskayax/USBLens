import type { DeviceKind, Lang, StatusFilter, UsbDevice } from "../types";
import { dict, kindLabels } from "../i18n";
import { KindIcon } from "./Icons";

const KINDS: DeviceKind[] = ["storage", "phone", "hid", "audio", "video", "printer", "network", "bluetooth", "serial", "hub", "other"];

interface Props {
  lang: Lang;
  devices: UsbDevice[];
  status: StatusFilter;
  setStatus: (s: StatusFilter) => void;
  kinds: Set<DeviceKind>;
  toggleKind: (k: DeviceKind) => void;
  hideHubs: boolean;
  setHideHubs: (v: boolean) => void;
}

export function Sidebar({ lang, devices, status, setStatus, kinds, toggleKind, hideHubs, setHideHubs }: Props) {
  const t = dict[lang];
  const pool = hideHubs ? devices.filter((d) => d.kind !== "hub") : devices;
  const statuses: { key: StatusFilter; label: string; count: number; dot?: string }[] = [
    { key: "all", label: t.all, count: pool.length },
    { key: "connected", label: t.connected, count: pool.filter((d) => d.connected).length, dot: "on" },
    { key: "history", label: t.history, count: pool.filter((d) => !d.connected).length, dot: "off" },
    { key: "disabled", label: t.disabledF, count: pool.filter((d) => d.disabled).length, dot: "warn" },
  ];
  const kindCounts = KINDS.map((k) => ({ k, n: pool.filter((d) => d.kind === k).length })).filter(
    (x) => x.n > 0 || kinds.has(x.k),
  );

  return (
    <aside className="sidebar">
      <div className="side-title">{t.status}</div>
      {statuses.map((s) => (
        <button key={s.key} className={`side-item ${status === s.key ? "active" : ""}`} onClick={() => setStatus(s.key)}>
          {s.dot ? <span className={`dot ${s.dot}`} /> : <span className="dot all" />}
          <span className="grow">{s.label}</span>
          <span className="count">{s.count}</span>
        </button>
      ))}

      <div className="side-title">{t.types}</div>
      {kindCounts.map(({ k, n }) => (
        <button key={k} className={`side-item ${kinds.has(k) ? "active" : ""}`} onClick={() => toggleKind(k)}>
          <KindIcon kind={k} size={15} />
          <span className="grow">{kindLabels[lang][k]}</span>
          <span className="count">{n}</span>
        </button>
      ))}

      <label className="side-check">
        <input type="checkbox" checked={hideHubs} onChange={(e) => setHideHubs(e.target.checked)} />
        {t.hideHubs}
      </label>
    </aside>
  );
}
