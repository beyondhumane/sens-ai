import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import puppeteer from "puppeteer-core";

const chrome = process.env.CHROME ?? "C:/Program Files/Google/Chrome/Application/chrome.exe";
const url = process.env.URL ?? "http://localhost:4321/";
const [width, height] = (process.env.SIZE ?? "1440x900").split("x").map(Number);
const scheme = process.env.THEME ?? "light";
const axe = readFileSync(createRequire(import.meta.url).resolve("axe-core/axe.min.js"), "utf8");

const browser = await puppeteer.launch({ executablePath: chrome, headless: true });
const page = await browser.newPage();
await page.setViewport({ width, height, deviceScaleFactor: 1, isMobile: width < 700, hasTouch: width < 700 });
await page.emulateMediaFeatures([{ name: "prefers-color-scheme", value: scheme }]);
await page.goto(url, { waitUntil: "networkidle0" });
await new Promise((settled) => setTimeout(settled, 1500));
await page.evaluate(axe);
const result = (await page.evaluate(
  `axe.run(document, { runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa", "best-practice"] } })`,
)) as { violations: { id: string; impact: string; help: string; nodes: { target: string[] }[] }[]; passes: unknown[] };
await browser.close();

console.log(`${width}x${height} · ${scheme} · ${result.passes.length} rules passed · ${result.violations.length} violations`);
for (const violation of result.violations) {
  console.log(`- ${violation.impact} · ${violation.id}: ${violation.help}`);
  for (const node of violation.nodes.slice(0, 4)) console.log(`    ${node.target.join(" ")}`);
}
