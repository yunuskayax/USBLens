import { useMemo } from "react";
import type { Lang, UsbDevice } from "../types";
import { dict } from "../i18n";
import { dayKey } from "../format";
import { KindIcon } from "./Icons";

type Ev = { at: string; type: "installed" | "arrived" | "removed"; d: UsbDevice };

interface Props {
  lang: Lang;
  devices: UsbDevice[];
  onSelect: (id: string) => void;
}

export function Timeline({ lang, devices, onSelect }: Props) {
  const t = dict[lang];
  const groups = useMemo(() => {
    const events: Ev[] = [];
    for (const d of devices) {
      if (d.firstInstall) events.push({ at: d.firstInstall, type: "installed", d });
      // Skip the arrival if it is the same moment as the first install.
      if (d.lastArrival && d.lastArrival !== d.firstInstall) events.push({ at: d.lastArrival, type: "arrived", d });
      if (d.lastRemoval) events.push({ at: d.lastRemoval, type: "removed", d });
    }
    events.sort((a, b) => b.at.localeCompare(a.at));
    const map = new Map<string, Ev[]>();
    for (const e of events) {
      const k = dayKey(e.at);
      if (!map.has(k)) map.set(k, []);
      map.get(k)!.push(e);
    }
    return [...map.entries()];
  }, [devices]);

  if (groups.length === 0) return <div className="empty">{t.noDevices}</div>;

  const locale = lang === "tr" ? "tr-TR" : "en-GB";
  const label = { installed: t.installed, arrived: t.arrived, removed: t.removed };

  return (
    <div className="timeline">
      {groups.map(([day, evs]) => (
        <section key={day} className="tl-day">
          <h3>
            {new Date(day + "T12:00:00").toLocaleDateString(locale, {
              weekday: "long",
              day: "numeric",
              month: "long",
              year: "numeric",
            })}
          </h3>
          <ol>
            {evs.map((e, i) => (
              <li key={i} className={`tl-ev ${e.type}`} onClick={() => onSelect(e.d.instanceId)}>
                <time>{new Date(e.at).toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit" })}</time>
                <span className="tl-mark" />
                <span className={`kind-badge k-${e.d.kind}`}>
                  <KindIcon kind={e.d.kind} size={14} />
                </span>
                <span className="grow">
                  <b>{e.d.name}</b>
                  {e.d.serialNumber && <span className="secondary mono"> · {e.d.serialNumber}</span>}
                </span>
                <span className={`tag ${e.type}`}>{label[e.type]}</span>
              </li>
            ))}
          </ol>
        </section>
      ))}
    </div>
  );
}
