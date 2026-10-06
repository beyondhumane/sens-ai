import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import puppeteer from "puppeteer-core";
import { home } from "../src/i18n/home";
import { htmlLang, locales } from "../src/i18n/locales";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const chrome = process.env.CHROME ?? "C:/Program Files/Google/Chrome/Application/chrome.exe";
const target = path.join(root, "src", "data", "headline.json");
const SIZE = 100;
const SAFETY = 1.03;

const font = `data:font/woff2;base64,${readFileSync(path.join(root, "public", "fonts", "space-grotesk.woff2")).toString("base64")}`;
const stacks: Record<string, string> = {
  ja: `"Space Grotesk", "Hiragino Sans", "Yu Gothic UI", "Yu Gothic", Meiryo, sans-serif`,
  zh: `"Space Grotesk", "PingFang SC", "Microsoft YaHei", "Noto Sans SC", sans-serif`,
};
const tracking = (locale: string) => (locale === "ja" || locale === "zh" ? "0" : "-0.05em");

const browser = await puppeteer.launch({ executablePath: chrome, headless: true });
const page = await browser.newPage();
const measured: Record<string, { line: number; word: number }> = {};

for (const locale of locales) {
  const lines = home(locale).lines;
  await page.setContent(`<!doctype html><html lang="${htmlLang[locale]}"><head><style>
    @font-face { font-family: "Space Grotesk"; src: url("${font}") format("woff2"); font-weight: 300 700; }
    span { font-family: ${stacks[locale] ?? `"Space Grotesk", sans-serif`}; font-size: ${SIZE}px; font-weight: 650; letter-spacing: ${tracking(locale)}; white-space: nowrap; }
  </style></head><body></body></html>`);
  await page.evaluate(() => document.fonts.ready);
  const widths = await page.evaluate(
    (texts: string[]) =>
      texts.map((text) => {
        const span = document.createElement("span");
        span.textContent = text;
        document.body.append(span);
        const width = span.getBoundingClientRect().width;
        span.remove();
        return width;
      }),
    [...lines.map((words) => words.join(" ")), ...lines.flat()],
  );
  const lineWidths = widths.slice(0, lines.length);
  const wordWidths = widths.slice(lines.length);
  measured[locale] = {
    line: Number(((Math.max(...lineWidths) / SIZE) * SAFETY).toFixed(3)),
    word: Number(((Math.max(...wordWidths) / SIZE) * SAFETY).toFixed(3)),
  };
}

await browser.close();
writeFileSync(target, `${JSON.stringify(measured, null, 2)}\n`);
console.log(Object.entries(measured).map(([locale, ems]) => `${locale} ${ems.line}em/${ems.word}em`).join(" · "));
