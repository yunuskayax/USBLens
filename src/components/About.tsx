import { useEffect, useState } from "react";
import type { Lang } from "../types";
import { dict } from "../i18n";
import { WEBSITE, appVersion, openUrl } from "../api";
import { IconExternal, IconX } from "./Icons";

export function AboutDialog({ lang, close }: { lang: Lang; close: () => void }) {
  const t = dict[lang];
  const [version, setVersion] = useState("…");

  useEffect(() => {
    appVersion().then(setVersion).catch(() => setVersion("0.2.1"));
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && close();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [close]);

  const site = WEBSITE.replace(/^https?:\/\//, "");

  return (
    <div className="modal-bg" onClick={close}>
      <div className="modal about" onClick={(e) => e.stopPropagation()} role="dialog" aria-modal aria-label={t.about}>
        <button className="icon-btn about-close" onClick={close} aria-label={t.close}>
          <IconX />
        </button>

        <div className="about-hero">
          <img src="/logo.png" alt="" width={72} height={72} />
          <h2>UsbLens</h2>
          <p className="secondary">{t.aboutTagline}</p>
        </div>

        <p className="about-body">{t.aboutBody}</p>

        <dl className="about-list">
          <div><dt>{t.version}</dt><dd>{version}</dd></div>
          <div><dt>{t.developer}</dt><dd>Yunus Kaya</dd></div>
          <div>
            <dt>{t.website}</dt>
            <dd>
              <button className="link" onClick={() => openUrl(WEBSITE)}>
                {site} <IconExternal size={13} />
              </button>
            </dd>
          </div>
          <div><dt>{t.license}</dt><dd>MIT</dd></div>
          <div><dt>{t.builtWith}</dt><dd>Tauri · Rust · React</dd></div>
        </dl>

        <div className="modal-actions">
          <button className="btn primary" onClick={() => openUrl(WEBSITE)}>
            <IconExternal size={15} /> {site}
          </button>
          <button className="btn" onClick={close} autoFocus>
            {t.close}
          </button>
        </div>

        <p className="about-foot secondary">© {new Date().getFullYear()} Yunus Kaya</p>
      </div>
    </div>
  );
}
