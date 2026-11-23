import type { Lang, UsbDevice } from "./types";
import { kindLabels } from "./i18n";
import { vidPid } from "./format";

// ---------------------------------------------------------------- capacity

/** Sizes printed on packaging (decimal GB). Flash drives report ~93–100 % of this. */
const NOMINAL_GB = [
  1, 2, 4, 8, 16, 32, 60, 64, 120, 128, 240, 250, 256, 320, 480, 500, 512, 640, 750, 960, 1000, 1500, 2000, 3000, 4000,
  5000, 6000, 8000, 10000, 12000, 14000, 16000, 18000, 20000, 22000, 24000,
];

function gbText(gb: number, lang: Lang): string {
  const n = new Intl.NumberFormat(lang === "tr" ? "tr-TR" : "en-US", { maximumFractionDigits: 1 });
  return gb >= 1000 ? `${n.format(gb / 1000)} TB` : `${n.format(gb)} GB`;
}

/** 61 530 000 000 bytes -> "64 GB" (the size on the box). */
export function nominalCapacity(bytes: number | null, lang: Lang): string | null {
  if (!bytes || bytes <= 0) return null;
  const gb = bytes / 1e9;
  let best: number | null = null;
  for (const n of NOMINAL_GB) {
    const ratio = gb / n;
    if (ratio > 1.005 || ratio < 0.9) continue;
    if (best == null || Math.abs(1 - ratio) < Math.abs(1 - gb / best)) best = n;
  }
  return best != null ? gbText(best, lang) : gbText(Math.round(gb * 10) / 10, lang);
}

/** Usable size as Windows shows it (binary units): "57,3 GB". */
export function usableCapacity(bytes: number | null, lang: Lang): string | null {
  if (!bytes || bytes <= 0) return null;
  const gib = bytes / 1024 ** 3;
  const n = new Intl.NumberFormat(lang === "tr" ? "tr-TR" : "en-US", { maximumFractionDigits: 1 });
  return gib >= 1024 ? `${n.format(gib / 1024)} TB` : `${n.format(gib)} GB`;
}

// ---------------------------------------------------------------- brand

/** Common USB storage vendors by VID (used when the disk does not report a usable brand). */
const VENDOR_BY_VID: Record<string, string> = {
  "0781": "SanDisk",
  "0951": "Kingston",
  "8564": "Transcend",
  "054C": "Sony",
  "0930": "Toshiba",
  "18A5": "Verbatim",
  "125F": "ADATA",
  "0BC2": "Seagate",
  "1058": "WD",
  "04E8": "Samsung",
  "05DC": "Lexar",
  "154B": "PNY",
  "0EA0": "Ours",
  "1005": "Apacer",
  "0718": "Imation",
  "13FE": "Kingston",
  "0480": "Toshiba",
  "059F": "LaCie",
  "04B4": "Cypress",
  "0411": "Buffalo",
  "1B1C": "Corsair",
  "2009": "iStorage",
};

/** Names that only say "some USB disk". */
const GENERIC = /^(generic|usb|mass|storage|flash|disk|usb disk|usb flash|usb flash disk|usb mass storage|general|ut\d*|\(standard.*\))$/i;

/** Proper case for brands that disks report in capitals. */
const PRETTY: Record<string, string> = {
  SANDISK: "SanDisk", KINGSTON: "Kingston", TOSHIBA: "Toshiba", SAMSUNG: "Samsung", TRANSCEND: "Transcend",
  VERBATIM: "Verbatim", LEXAR: "Lexar", SEAGATE: "Seagate", SONY: "Sony", APACER: "Apacer", CORSAIR: "Corsair",
  INTEGRAL: "Integral", PHILIPS: "Philips", HIKVISION: "Hikvision", NETAC: "Netac", TEAM: "Team", LACIE: "LaCie",
  "WESTERN DIGITAL": "WD", WDC: "WD", PATRIOT: "Patriot", EMTEC: "Emtec", INTENSO: "Intenso", ISTORAGE: "iStorage",
};

function pretty(s: string): string {
  const t = s.trim();
  return PRETTY[t.toUpperCase()] ?? t;
}

export function brandOf(d: UsbDevice): string | null {
  const v = d.vendorName?.trim();
  if (v && !GENERIC.test(v)) return pretty(v);
  if (d.vendorId && VENDOR_BY_VID[d.vendorId.toUpperCase()]) return VENDOR_BY_VID[d.vendorId.toUpperCase()];
  // "Kingston DataTraveler 3.0" -> "Kingston"
  const first = (d.description || d.name).split(/\s+/)[0];
  if (first && first.length > 2 && !GENERIC.test(first) && /[a-z]/i.test(first)) {
    const known = Object.values(VENDOR_BY_VID).concat(Object.values(PRETTY));
    const hit = known.find((k) => k.toLowerCase() === first.toLowerCase());
    if (hit) return hit;
  }
  return null;
}

export function modelOf(d: UsbDevice): string | null {
  const p = d.productName?.trim();
  if (p && !GENERIC.test(p)) return p;
  return null;
}

// ---------------------------------------------------------------- label

export function typeText(d: UsbDevice, lang: Lang): string {
  if (d.kind !== "storage") return kindLabels[lang][d.kind];
  if (d.removable === false) return lang === "tr" ? "Harici Disk" : "External Drive";
  return lang === "tr" ? "USB Bellek" : "USB Flash Drive";
}

/** Values for every template token, in Turkish and English spelling. */
export function tokenValues(d: UsbDevice, lang: Lang): Record<string, string> {
  const brand = brandOf(d) ?? "";
  const model = modelOf(d) ?? "";
  const capacity = nominalCapacity(d.capacityBytes, lang) ?? "";
  const type = typeText(d, lang);
  const serial = d.serialNumber ?? "";
  const drive = d.driveLetters.join(" ");
  const vp = vidPid(d);
  return {
    marka: brand, brand,
    model,
    kapasite: capacity, capacity,
    "tür": type, tur: type, type,
    seri: serial, serial,
    "sürücü": drive, surucu: drive, drive,
    vidpid: vp,
    ad: d.name, name: d.name,
  };
}

export const TEMPLATE_TOKENS = ["{marka}", "{model}", "{kapasite}", "{tür}", "{seri}", "{sürücü}", "{vidpid}", "{ad}"];

/**
 * "{marka} {kapasite} {tür} (S.No: {seri})" -> "Kingston 64 GB USB Bellek (S.No: 2009…)".
 * A bracketed part whose tokens are all empty is dropped, so a missing serial
 * does not leave "(S.No: )" behind.
 */
export function deviceLabel(d: UsbDevice, template: string, lang: Lang): string {
  const values = tokenValues(d, lang);
  const sub = (s: string) => s.replace(/\{([^{}]+)\}/g, (_, k: string) => values[k.trim().toLowerCase()] ?? `{${k}}`);
  const hasValue = (s: string) => {
    const tokens = [...s.matchAll(/\{([^{}]+)\}/g)].map((m) => values[m[1].trim().toLowerCase()] ?? "x");
    return tokens.length === 0 || tokens.some((v) => v !== "");
  };
  const withoutEmptyGroups = template.replace(/\(([^()]*)\)|\[([^[\]]*)\]/g, (whole) => (hasValue(whole) ? whole : ""));
  return sub(withoutEmptyGroups)
    .replace(/\s{2,}/g, " ")
    .replace(/\s+([,.;:)\]])/g, "$1")
    .trim();
}

/** Every property as "Name: value" lines, for "Copy all". */
export function detailsText(rows: [string, string | null | undefined][]): string {
  return rows
    .filter(([, v]) => v != null && v !== "" && v !== "—")
    .map(([k, v]) => `${k}: ${v}`)
    .join("\n");
}
