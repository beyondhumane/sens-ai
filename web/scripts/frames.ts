import { mkdirSync } from "node:fs";
import path from "node:path";
import puppeteer from "puppeteer-core";

const chrome = process.env.CHROME ?? "C:/Program Files/Google/Chrome/Application/chrome.exe";
const url = process.env.URL ?? "http://localhost:4321/";
const [width, height] = (process.env.SIZE ?? "1440x900").split("x").map(Number);
const quiet = process.env.MOTION === "reduce";
const out = path.resolve(process.env.OUT ?? "frames");
const wait = Number(process.env.WAIT ?? 1200);
const chapters = process.env.AT === "chapters";
const entrance = process.env.AT === "entrance";
const theming = process.env.AT === "theme";
const scheme = process.env.THEME;
const listed = chapters || entrance || theming ? [] : (process.env.AT ?? "0,300,600,900,1200").split(",").map(Number);
const mobile = width < 700;

const pause = (millis: number) => new Promise((done) => setTimeout(done, millis));

const browser = await puppeteer.launch({ executablePath: chrome, headless: true });
const page = await browser.newPage();
await page.setViewport({ width, height, deviceScaleFactor: Number(process.env.DPR ?? 1), isMobile: mobile, hasTouch: mobile });
await page.emulateMediaFeatures([
  ...(quiet ? [{ name: "prefers-reduced-motion", value: "reduce" }] : []),
  ...(scheme ? [{ name: "prefers-color-scheme", value: scheme }] : []),
]);
const scripted = process.env.JS !== "off";
await page.setJavaScriptEnabled(scripted);
const errors: string[] = [];
page.on("pageerror", (error) => errors.push(String(error)));
page.on("console", (message) => {
  if (message.type() === "error") errors.push(message.text());
});
mkdirSync(out, { recursive: true });
const name = (label: string | number) =>
  path.join(out, `${width}x${height}${scheme ? `-${scheme}` : ""}${quiet ? "-reduce" : ""}${scripted ? "" : "-nojs"}-${String(label).padStart(5, "0")}.png`);

if (entrance) {
  const started = Date.now();
  await page.goto(url, { waitUntil: "domcontentloaded" });
  for (const at of [100, 350, 600, 850, 1400]) {
    await pause(Math.max(0, at - (Date.now() - started)));
    await page.screenshot({ path: name(`t${at}`) });
    console.log(name(`t${at}`));
  }
  const state = await page.evaluate(() => ({
    motion: document.documentElement.dataset.motion,
    ready: document.documentElement.dataset.ready,
    reduce: matchMedia("(prefers-reduced-motion: reduce)").matches,
    desktop: matchMedia("(min-width: 1024px) and (min-height: 640px)").matches,
    running: document.getAnimations().length,
  }));
  console.log(JSON.stringify(state));
  if (errors.length > 0) console.log(`errors:\n${errors.join("\n")}`);
  await browser.close();
  process.exit(0);
}

await page.goto(url, { waitUntil: "networkidle0" });
await pause(1500);

if (theming) {
  const top = Number(process.env.TOP ?? 0);
  await page.evaluate((y) => window.scrollTo({ top: y, behavior: "instant" }), top);
  await pause(wait);
  const moments = (process.env.MOMENTS ?? "0,120,260,420,600,800,1000,1300,1700").split(",").map(Number);
  for (const direction of ["there", "back"]) {
    const before = await page.evaluate(() => document.documentElement.dataset.theme);
    await page.click("[data-theme-toggle]");
    await page.waitForFunction(() => document.getAnimations().some((running) => String((running.effect as KeyframeEffect | null)?.pseudoElement).includes("view-transition")));
    await page.evaluate(() => document.getAnimations().forEach((running) => running.pause()));
    for (const at of moments) {
      await page.evaluate((time) => document.getAnimations().forEach((running) => (running.currentTime = time)), at);
      await page.screenshot({ path: name(`${before}-${direction}-t${at}`) });
      console.log(name(`${before}-${direction}-t${at}`));
    }
    await page.evaluate(() => document.getAnimations().forEach((running) => running.play()));
    await page.waitForFunction(() => !document.documentElement.hasAttribute("data-theming"));
    await pause(900);
  }
  if (errors.length > 0) console.log(`errors:\n${errors.join("\n")}`);
  await browser.close();
  process.exit(0);
}

const stops = chapters
  ? await page.evaluate(() =>
      [...document.querySelectorAll<HTMLElement>("[data-chapter]")]
        .filter((item) => item.getBoundingClientRect().height > 0)
        .map((item) => Math.round(item.getBoundingClientRect().top + window.scrollY - window.innerHeight * 0.4)),
    )
  : listed;

for (const top of stops) {
  await page.evaluate((y) => window.scrollTo({ top: y, behavior: "instant" }), top);
  await pause(wait);
  await page.screenshot({ path: name(top) });
  console.log(name(top));
}

if (errors.length > 0) console.log(`errors:\n${errors.join("\n")}`);
await browser.close();
