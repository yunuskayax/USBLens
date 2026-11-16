#!/usr/bin/env node
// Single source of truth for release notes: src/changelog.json
//
//   node scripts/changelog.mjs notes 0.2.4   -> Markdown for one version (GitHub Release body)
//   node scripts/changelog.mjs md            -> writes CHANGELOG.md
//   node scripts/changelog.mjs check 0.2.4   -> fails if the version has no entry

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const log = JSON.parse(readFileSync(join(root, "src/changelog.json"), "utf8"));

const label = {
  tr: { new: "Yeni", improved: "İyileştirme", fixed: "Düzeltme" },
  en: { new: "New", improved: "Improved", fixed: "Fixed" },
};

const find = (v) => log.find((e) => e.version === String(v).replace(/^v/i, ""));

function section(entry, lang) {
  return entry.items.map((i) => `- **${label[lang][i.type]}:** ${i[lang]}`).join("\n");
}

function notes(entry) {
  return [
    "### Yenilikler",
    section(entry, "tr"),
    "",
    "### What's new",
    section(entry, "en"),
    "",
    "**İndir / Download:** `UsbLens-setup.exe` · https://usblens.com",
  ].join("\n");
}

const [cmd, arg] = process.argv.slice(2);

if (cmd === "notes") {
  const entry = find(arg);
  if (!entry) process.exit(0); // no entry: print nothing, caller keeps its own text
  process.stdout.write(notes(entry) + "\n");
} else if (cmd === "check") {
  if (!find(arg)) {
    console.error(`src/changelog.json has no entry for ${arg}`);
    process.exit(1);
  }
} else if (cmd === "md") {
  const md = [
    "# Değişiklik günlüğü / Changelog",
    "",
    "Bu dosya `src/changelog.json` dosyasından üretilir (`node scripts/changelog.mjs md`).",
    "",
    ...log.flatMap((e) => [`## ${e.version} — ${e.date}`, "", section(e, "tr"), "", "<details><summary>English</summary>", "", section(e, "en"), "", "</details>", ""]),
  ].join("\n");
  writeFileSync(join(root, "CHANGELOG.md"), md);
  console.log("CHANGELOG.md written");
} else {
  console.error("usage: changelog.mjs notes <version> | md | check <version>");
  process.exit(2);
}
