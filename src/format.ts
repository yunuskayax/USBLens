import type { Lang, UsbDevice } from "./types";

export function formatDate(iso: string | null, lang: Lang): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "—";
  return d.toLocaleString(lang === "tr" ? "tr-TR" : "en-GB", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function relative(iso: string | null, lang: Lang): string {
  if (!iso) return "";
  const diff = (Date.now() - new Date(iso).getTime()) / 1000;
  const rtf = new Intl.RelativeTimeFormat(lang, { numeric: "auto" });
  const steps: [number, Intl.RelativeTimeFormatUnit][] = [
    [60, "second"],
    [3600, "minute"],
    [86400, "hour"],
    [86400 * 30, "day"],
    [86400 * 365, "month"],
  ];
  let unit: Intl.RelativeTimeFormatUnit = "year";
  let value = diff / (86400 * 365);
  let prev = 1;
  for (const [limit, u] of steps) {
    if (Math.abs(diff) < limit) {
      unit = u;
      value = diff / prev;
      break;
    }
    prev = limit;
  }
  return rtf.format(-Math.round(value), unit);
}

export function vidPid(d: UsbDevice): string {
  return d.vendorId && d.productId ? `${d.vendorId}:${d.productId}` : "";
}

/** Local calendar day, YYYY-MM-DD. */
export function dayKey(iso: string): string {
  return new Date(iso).toLocaleDateString("sv-SE");
}
