import { useMemo, useState } from "react";
import type { Lang, LogEvent } from "../types";
import { dict, reasonText } from "../i18n";
import { dayKey } from "../format";
import { IconArrowIn, IconArrowOut, IconDownload, IconShieldAlert, IconTrash, KindIcon } from "./Icons";
import { levelLabel } from "./Security";

type Filter = "all" | "arrival" | "removal" | "alerts";

interface Props {
  lang: Lang;
  events: LogEvent[];
  enabled: boolean;
  query: string;
  onSelect: (instanceId: string) => void;
  onExport: (format: "csv" | "json") => void;
  onClear: () => void;
}

export function EventsView({ lang, events, enabled, query, onSelect, onExport, onClear }: Props) {
  const t = dict[lang];
  const [filter, setFilter] = useState<Filter>("all");
  const locale = lang === "tr" ? "tr-TR" : "en-GB";

  const groups = useMemo(() => {
    const q = query.trim().toLocaleLowerCase(lang);
    const list = events.filter((e) => {
      if (filter === "arrival" && e.action !== "arrival") return false;
      if (filter === "removal" && e.action !== "removal") return false;
      if (filter === "alerts" && !e.alerts.length) return false;
      if (!q) return true;
      const hay = [e.name, e.serialNumber, e.vendorId && `${e.vendorId}:${e.productId}`, e.user, ...e.driveLetters]
        .filter(Boolean)
        .join(" ")
        .toLocaleLowerCase(lang);
      return q.split(/\s+/).every((p) => hay.includes(p));
    });
    const map = new Map<string, LogEvent[]>();
    for (const e of list) {
      const k = dayKey(e.at);
      if (!map.has(k)) map.set(k, []);
      map.get(k)!.push(e);
    }
    return [...map.entries()];
  }, [events, filter, query, lang]);

  const alertCount = events.filter((e) => e.alerts.length).length;
  const filters: { key: Filter; label: string; count?: number }[] = [
    { key: "all", label: t.all, count: events.length },
    { key: "arrival", label: t.plugEvent },
    { key: "removal", label: t.unplugEvent },
    { key: "alerts", label: t.onlyAlerts, count: alertCount },
  ];

  return (
    <div className="events">
      <div className="events-bar">
        <div className="segmented small">
          {filters.map((f) => (
            <button key={f.key} className={filter === f.key ? "on" : ""} onClick={() => setFilter(f.key)}>
              {f.label}
              {f.count != null && <span className={`seg-count ${f.key === "alerts" && f.count ? "danger" : ""}`}>{f.count}</span>}
            </button>
          ))}
        </div>
        <span className="grow" />
        <button className="btn" onClick={() => onExport("csv")} disabled={!events.length}>
          <IconDownload size={15} /> CSV
        </button>
        <button className="btn" onClick={() => onExport("json")} disabled={!events.length}>
          <IconDownload size={15} /> JSON
        </button>
        <button className="btn danger" onClick={onClear} disabled={!events.length}>
          <IconTrash size={15} /> {t.clearLog}
        </button>
      </div>

      {!enabled && <div className="events-note">{t.eventLogOff}</div>}
      {groups.length === 0 ? (
        <div className="empty small">{events.length ? t.noDevices : t.noEvents}</div>
      ) : (
        groups.map(([day, evs]) => (
          <section key={day} className="tl-day">
            <h3>
              {new Date(day + "T12:00:00").toLocaleDateString(locale, { weekday: "long", day: "numeric", month: "long", year: "numeric" })}
            </h3>
            <ol className="ev-list">
              {evs.map((e, i) => (
                <li key={i} className={`ev-row ${e.level}`} onClick={() => e.instanceId && onSelect(e.instanceId)}>
                  <time>{new Date(e.at).toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit", second: "2-digit" })}</time>
                  <span className={`ev-action ${e.action}`} title={e.action === "arrival" ? t.plugEvent : t.unplugEvent}>
                    {e.action === "arrival" ? <IconArrowIn size={14} /> : <IconArrowOut size={14} />}
                  </span>
                  {e.kind && (
                    <span className={`kind-badge k-${e.kind}`}>
                      <KindIcon kind={e.kind} size={14} />
                    </span>
                  )}
                  <div className="grow ev-main">
                    <div>
                      <b>{e.name}</b>
                      <span className="secondary">
                        {" "}
                        · {e.action === "arrival" ? t.plugEvent : t.unplugEvent}
                        {e.vendorId && ` · ${e.vendorId}:${e.productId}`}
                        {e.serialNumber && ` · ${e.serialNumber}`}
                        {e.driveLetters.length > 0 && ` · ${e.driveLetters.join(" ")}`}
                      </span>
                    </div>
                    {e.alerts.length > 0 && (
                      <div className={`ev-alerts ${e.level}`}>
                        <IconShieldAlert size={13} />
                        <span>
                          <b>{levelLabel(lang, e.level)}:</b> {e.alerts.map((a) => reasonText(lang, a)).join(" · ")}
                        </span>
                      </div>
                    )}
                  </div>
                  {e.blocked && <span className="tag blocked">{t.blocked}</span>}
                  {e.blockError && <span className="tag removed" title={e.blockError}>{t.blockFailed}</span>}
                  {e.user && <span className="ev-user secondary">{e.user}</span>}
                </li>
              ))}
            </ol>
          </section>
        ))
      )}
    </div>
  );
}
