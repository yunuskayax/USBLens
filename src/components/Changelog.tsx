import { useEffect } from "react";
import changelog from "../changelog.json";
import type { ChangeEntry, ChangeType, Lang } from "../types";
import { dict } from "../i18n";
import { IconX } from "./Icons";

export const CHANGELOG = changelog as ChangeEntry[];

/** "0.2.10" > "0.2.9" */
export function versionGt(a: string, b: string): boolean {
  const pa = a.replace(/^v/i, "").split(".").map((n) => parseInt(n, 10) || 0);
  const pb = b.replace(/^v/i, "").split(".").map((n) => parseInt(n, 10) || 0);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const d = (pa[i] ?? 0) - (pb[i] ?? 0);
    if (d) return d > 0;
  }
  return false;
}

function Tag({ type, lang }: { type: ChangeType; lang: Lang }) {
  const t = dict[lang];
  const label = type === "new" ? t.tagNew : type === "improved" ? t.tagImproved : t.tagFixed;
  return <span className={`chg-tag ${type}`}>{label}</span>;
}

export function ChangelogList({ lang, entries, current }: { lang: Lang; entries: ChangeEntry[]; current?: string }) {
  const t = dict[lang];
  const locale = lang === "tr" ? "tr-TR" : "en-GB";
  return (
    <div className="chg">
      {entries.map((e) => (
        <section key={e.version} className="chg-entry">
          <header>
            <b>v{e.version}</b>
            {current === e.version && <span className="chg-installed">{t.installedTag}</span>}
            <span className="setting-desc">
              {new Date(e.date + "T12:00:00").toLocaleDateString(locale, { day: "numeric", month: "long", year: "numeric" })}
            </span>
          </header>
          <ul>
            {e.items.map((i, n) => (
              <li key={n}>
                <Tag type={i.type} lang={lang} />
                <span>{i[lang]}</span>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}

const LABEL_TO_TYPE: Record<string, ChangeType> = {
  yeni: "new", new: "new", "iyileştirme": "improved", improved: "improved", "düzeltme": "fixed", fixed: "fixed",
};

/**
 * Release notes coming from GitHub (Markdown made by scripts/changelog.mjs or typed by hand).
 * Picks the section for the UI language when both are present.
 */
export function RemoteNotes({ lang, text }: { lang: Lang; text: string }) {
  let body = text;
  const split = text.split(/^###\s+What's new\s*$/im);
  if (split.length === 2) body = lang === "en" ? split[1] : split[0];
  const lines = body
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter((l) => l && !/^###\s+Yenilikler/i.test(l) && !/^\*\*İndir/i.test(l));

  return (
    <ul className="chg remote">
      {lines.map((l, n) => {
        if (l.startsWith("#")) return <li key={n} className="chg-h">{l.replace(/^#+\s*/, "")}</li>;
        const m = l.replace(/^[-*]\s+/, "").match(/^\*\*(.+?):\*\*\s*(.*)$/);
        if (m && LABEL_TO_TYPE[m[1].toLowerCase()]) {
          return (
            <li key={n}>
              <Tag type={LABEL_TO_TYPE[m[1].toLowerCase()]} lang={lang} />
              <span>{m[2]}</span>
            </li>
          );
        }
        return (
          <li key={n}>
            <span>{l.replace(/^[-*]\s+/, "").replace(/\*\*/g, "")}</span>
          </li>
        );
      })}
    </ul>
  );
}

/** Shown once after an update, or from the About dialog. */
export function WhatsNewDialog({ lang, entries, current, close }: { lang: Lang; entries: ChangeEntry[]; current: string; close: () => void }) {
  const t = dict[lang];
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && close();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [close]);
  return (
    <div className="modal-bg" onClick={close}>
      <div className="modal whatsnew" onClick={(e) => e.stopPropagation()} role="dialog" aria-modal aria-label={t.whatsNew}>
        <button className="icon-btn about-close" onClick={close} aria-label={t.close}>
          <IconX />
        </button>
        <div className="whatsnew-hero">
          <img src="/logo.png" alt="" width={48} height={48} />
          <div>
            <h2>{t.whatsNewTitle}</h2>
            <p className="secondary">
              v{current} · {t.whatsNewSub}
            </p>
          </div>
        </div>
        <div className="whatsnew-body">
          <ChangelogList lang={lang} entries={entries} current={current} />
        </div>
        <div className="modal-actions">
          <button className="btn primary" onClick={close} autoFocus>
            {t.gotIt}
          </button>
        </div>
      </div>
    </div>
  );
}
