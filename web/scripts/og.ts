import { mkdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import puppeteer from "puppeteer-core";
import brand from "../src/brand/brand.json" with { type: "json" };
import releases from "../src/data/releases.json" with { type: "json" };

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const out = path.join(root, "public", "og");
const chrome = process.env.CHROME ?? "C:/Program Files/Google/Chrome/Application/chrome.exe";
const WIDTH = 1200;
const HEIGHT = 630;

const color = brand.palette;
const font = (file: string) => `data:font/woff2;base64,${readFileSync(path.join(root, "public", "fonts", file)).toString("base64")}`;
const mark = readFileSync(path.join(root, "public", "brand", "mark.svg"), "utf8");
const latest = releases[0];

interface Card {
  name: string;
  label: string;
  title: string;
  line: string;
}

const cards: Card[] = [
  {
    name: "home",
    label: "Open source · MIT · Windows 10 & 11",
    title: "Less noise.<br>More sense.",
    line: "Claude Code that reuses what you have, and writes less.",
  },
  {
    name: "research",
    label: "Research · 1 October 2026",
    title: "Less code for<br>the same work.",
    line: "12% less code and 18% fewer tokens over 30 chained tasks, in our benchmark.",
  },
  {
    name: "releases",
    label: latest ? `Latest · v${latest.version}` : "Releases",
    title: "What changed<br>in Sens.",
    line: latest?.headline ?? "Every version, its notes and who made it.",
  },
];

const page = (card: Card) => `<!doctype html>
<html><head><meta charset="utf-8"><style>
@font-face { font-family: "Space Grotesk"; src: url("${font("space-grotesk.woff2")}") format("woff2"); font-weight: 300 700; }
@font-face { font-family: "Geist Mono"; src: url("${font("geist-mono.woff2")}") format("woff2"); font-weight: 100 900; }
html, body { margin: 0; width: ${WIDTH}px; height: ${HEIGHT}px; overflow: hidden; }
body { position: relative; background: ${color.paper}; color: ${color["carbon-950"]}; font-family: "Space Grotesk", sans-serif; -webkit-font-smoothing: antialiased; }
.field { position: absolute; top: 0; right: 0; width: 330px; height: 420px; background: ${color["signal-500"]}; }
.brand { position: absolute; top: 56px; left: 72px; display: flex; align-items: center; gap: 14px; font-size: 40px; font-weight: 650; letter-spacing: -0.03em; }
.brand svg { width: 52px; height: 52px; }
.label { position: absolute; top: 176px; left: 72px; font-size: 18px; font-weight: 600; letter-spacing: 0.08em; text-transform: uppercase; color: ${color["alloy-600"]}; }
h1 { position: absolute; top: 210px; left: 66px; margin: 0; font-size: 116px; line-height: 0.9; font-weight: 650; letter-spacing: -0.05em; }
p { position: absolute; left: 72px; bottom: 56px; margin: 0; max-width: 820px; font-size: 30px; line-height: 1.25; letter-spacing: -0.01em; color: ${color["alloy-600"]}; }
</style></head><body>
<div class="field"></div>
<div class="brand">${mark}<span>sens</span></div>
<div class="label">${card.label}</div>
<h1>${card.title}</h1>
<p>${card.line}</p>
</body></html>`;

mkdirSync(out, { recursive: true });
const browser = await puppeteer.launch({ executablePath: chrome, headless: true });
const tab = await browser.newPage();
await tab.setViewport({ width: WIDTH, height: HEIGHT, deviceScaleFactor: 1 });
for (const card of cards) {
  await tab.setContent(page(card), { waitUntil: "load" });
  await tab.evaluate(() => document.fonts.ready);
  await tab.screenshot({ path: path.join(out, `${card.name}.png`) });
  console.log(`og · ${card.name}.png`);
}
await browser.close();
