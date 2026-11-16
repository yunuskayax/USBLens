import type { DeviceKind, Lang, StatusFilter, UsbDevice } from "../types";
import { dict, kindLabels } from "../i18n";
import { IconBan, IconChevron, IconClock, IconHub, IconLayers, IconPlug, KindIcon } from "./Icons";

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
  collapsed: boolean;
  setCollapsed: (v: boolean) => void;
}

export function Sidebar({ lang, devices, status, setStatus, kinds, toggleKind, hideHubs, setHideHubs, collapsed, setCollapsed }: Props) {
  const t = dict[lang];
  const pool = hideHubs ? devices.filter((d) => d.kind !== "hub") : devices;
  const statuses: { key: StatusFilter; label: string; count: number; icon: React.ReactNode; tone: string }[] = [
    { key: "all", label: t.all, count: pool.length, icon: <IconLayers size={16} />, tone: "all" },
    { key: "connected", label: t.connected, count: pool.filter((d) => d.connected).length, icon: <IconPlug size={16} />, tone: "on" },
    { key: "history", label: t.history, count: pool.filter((d) => !d.connected).length, icon: <IconClock size={16} />, tone: "off" },
    { key: "disabled", label: t.disabledF, count: pool.filter((d) => d.disabled).length, icon: <IconBan size={16} />, tone: "warn" },
  ];
  const kindCounts = KINDS.map((k) => ({ k, n: pool.filter((d) => d.kind === k).length })).filter(
    (x) => x.n > 0 || kinds.has(x.k),
  );
  const toggleLabel = collapsed ? t.expandSidebar : t.collapseSidebar;

  return (
    <aside className={`sidebar ${collapsed ? "collapsed" : ""}`}>
      <button className="side-toggle" onClick={() => setCollapsed(!collapsed)} title={`${toggleLabel} (Ctrl+B)`} aria-label={toggleLabel} aria-expanded={!collapsed}>
        <IconChevron size={16} />
        {!collapsed && <span>{toggleLabel}</span>}
      </button>

      <div className="side-title">{collapsed ? <span className="side-rule" /> : t.status}</div>
      {statuses.map((s) => (
        <button
          key={s.key}
          className={`side-item ${status === s.key ? "active" : ""}`}
          onClick={() => setStatus(s.key)}
          title={collapsed ? `${s.label} (${s.count})` : undefined}
        >
          <span className={`side-icon tone-${s.tone}`}>{s.icon}</span>
          <span className="grow label">{s.label}</span>
          <span className="count">{s.count}</span>
        </button>
      ))}

      <div className="side-title">{collapsed ? <span className="side-rule" /> : t.types}</div>
      {kindCounts.map(({ k, n }) => (
        <button
          key={k}
          className={`side-item ${kinds.has(k) ? "active" : ""}`}
          onClick={() => toggleKind(k)}
          title={collapsed ? `${kindLabels[lang][k]} (${n})` : undefined}
        >
          <span className="side-icon">
            <KindIcon kind={k} size={16} />
          </span>
          <span className="grow label">{kindLabels[lang][k]}</span>
          <span className="count">{n}</span>
        </button>
      ))}

      {collapsed ? (
        <button
          className={`side-item hub-toggle ${hideHubs ? "" : "active"}`}
          onClick={() => setHideHubs(!hideHubs)}
          title={hideHubs ? t.showHubs : t.hideHubs}
        >
          <span className="side-icon">
            <IconHub size={16} />
          </span>
        </button>
      ) : (
        <label className="side-check">
          <input type="checkbox" checked={hideHubs} onChange={(e) => setHideHubs(e.target.checked)} />
          {t.hideHubs}
        </label>
      )}
    </aside>
  );
}
