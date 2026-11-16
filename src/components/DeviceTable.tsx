import type { Lang, SortKey, UsbDevice } from "../types";
import { dict, kindLabels } from "../i18n";
import { formatDate, relative, vidPid } from "../format";
import { KindIcon } from "./Icons";

interface Props {
  lang: Lang;
  devices: UsbDevice[];
  selected: string | null;
  onSelect: (id: string) => void;
  sort: { key: SortKey; asc: boolean };
  setSort: (s: { key: SortKey; asc: boolean }) => void;
  flash: Set<string>;
}

export function DeviceTable({ lang, devices, selected, onSelect, sort, setSort, flash }: Props) {
  const t = dict[lang];
  const cols: { key: SortKey; label: string; className?: string }[] = [
    { key: "name", label: t.name, className: "col-name" },
    { key: "kind", label: t.type },
    { key: "vidpid", label: "VID:PID", className: "mono" },
    { key: "serial", label: t.serial, className: "mono" },
    { key: "drive", label: t.drive },
    { key: "lastArrival", label: t.lastArrival },
    { key: "lastRemoval", label: t.lastRemoval },
    { key: "firstInstall", label: t.firstInstall },
  ];

  const header = (c: (typeof cols)[number]) => (
    <th
      key={c.key}
      className={c.className}
      onClick={() => setSort({ key: c.key, asc: sort.key === c.key ? !sort.asc : c.key === "name" || c.key === "kind" })}
    >
      {c.label}
      {sort.key === c.key && <span className="sort">{sort.asc ? "▲" : "▼"}</span>}
    </th>
  );

  if (devices.length === 0) {
    return <div className="empty">{t.noDevices}</div>;
  }

  return (
    <div className="table-wrap">
      <table className="devices">
        <thead>
          <tr>
            <th className="col-dot" />
            {cols.map(header)}
          </tr>
        </thead>
        <tbody>
          {devices.map((d) => (
            <tr
              key={d.instanceId}
              className={`${selected === d.instanceId ? "selected" : ""} ${flash.has(d.instanceId) ? "flash" : ""} ${d.connected ? "" : "dim"}`}
              onClick={() => onSelect(d.instanceId)}
            >
              <td className="col-dot">
                <span className={`dot ${d.disabled ? "warn" : d.connected ? "on" : "off"}`} title={d.connected ? t.connected : t.history} />
              </td>
              <td className="col-name">
                <div className="name-cell">
                  <span className={`kind-badge k-${d.kind}`}>
                    <KindIcon kind={d.kind} size={15} />
                  </span>
                  <div className="name-text">
                    <span className="primary">{d.name}</span>
                    {d.description && d.description !== d.name && <span className="secondary">{d.description}</span>}
                  </div>
                </div>
              </td>
              <td>{kindLabels[lang][d.kind]}</td>
              <td className="mono">{vidPid(d) || "—"}</td>
              <td className="mono ellipsis" title={d.serialNumber ?? ""}>{d.serialNumber ?? "—"}</td>
              <td>{d.driveLetters.length ? d.driveLetters.map((l) => <span key={l} className="chip">{l}</span>) : "—"}</td>
              <td title={formatDate(d.lastArrival, lang)}>
                <div className="date">{formatDate(d.lastArrival, lang)}</div>
                <div className="secondary">{relative(d.lastArrival, lang)}</div>
              </td>
              <td>{formatDate(d.lastRemoval, lang)}</td>
              <td>{formatDate(d.firstInstall, lang)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
