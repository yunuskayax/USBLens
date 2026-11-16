import type { Lang, RiskLevel, Security } from "../types";
import { dict, reasonText } from "../i18n";
import { IconShieldAlert, IconShieldCheck } from "./Icons";

export function levelLabel(lang: Lang, level: RiskLevel): string {
  const t = dict[lang];
  return level === "high" ? t.riskHigh : level === "medium" ? t.riskMedium : level === "info" ? t.riskInfo : "";
}

/** Small shield shown next to a device name. */
export function RiskBadge({ lang, security }: { lang: Lang; security: Security }) {
  if (security.level === "none") {
    if (security.allowed === true) {
      return (
        <span className="risk-badge ok" title={dict[lang].allowedTag}>
          <IconShieldCheck size={14} />
        </span>
      );
    }
    return null;
  }
  const title = security.reasons.map((r) => "• " + reasonText(lang, r)).join("\n");
  return (
    <span className={`risk-badge ${security.level}`} title={`${levelLabel(lang, security.level)}\n${title}`}>
      <IconShieldAlert size={14} />
    </span>
  );
}

/** Coloured box with the reasons, used in the detail panel. */
export function RiskBox({ lang, security }: { lang: Lang; security: Security }) {
  if (security.level === "none") return null;
  return (
    <div className={`risk-box ${security.level}`}>
      <div className="risk-head">
        <IconShieldAlert size={16} />
        <b>{levelLabel(lang, security.level)}</b>
      </div>
      <ul>
        {security.reasons.map((r) => (
          <li key={r}>{reasonText(lang, r)}</li>
        ))}
      </ul>
    </div>
  );
}
