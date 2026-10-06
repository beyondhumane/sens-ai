import puppeteer from "puppeteer-core";

const chrome = process.env.CHROME ?? "C:/Program Files/Google/Chrome/Application/chrome.exe";
const url = process.env.URL ?? "http://localhost:4321/";
const [width, height] = (process.env.SIZE ?? "1440x900").split("x").map(Number);
const slowdown = Number(process.env.CPU ?? 4);
const theming = process.env.MODE === "theme";
const pause = (millis: number) => new Promise((done) => setTimeout(done, millis));

const watch = `(() => {
  const record = { frames: [], long: [], shifts: 0 };
  let last = performance.now();
  const tick = (now) => {
    record.frames.push(now - last);
    last = now;
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
  new PerformanceObserver((list) => {
    for (const entry of list.getEntries()) record.long.push(entry.duration);
  }).observe({ type: "long-animation-frame", buffered: false });
  new PerformanceObserver((list) => {
    for (const entry of list.getEntries()) if (!entry.hadRecentInput) record.shifts += entry.value;
  }).observe({ type: "layout-shift", buffered: true });
  window.__record = record;
})()`;

const browser = await puppeteer.launch({ executablePath: chrome, headless: true });
const page = await browser.newPage();
await page.setViewport({ width, height, deviceScaleFactor: 1 });
await page.goto(url, { waitUntil: "networkidle0" });
await pause(1200);
await page.emulateCPUThrottling(slowdown);
await page.evaluate(watch);
if (theming) {
  for (let switched = 0; switched < 4; switched += 1) {
    await page.click("[data-theme-toggle]");
    await pause(1500);
  }
} else {
  await page.mouse.move(width / 2, height / 2);
  const total = Number(await page.evaluate("document.documentElement.scrollHeight - innerHeight"));
  for (let travelled = 0; travelled < total; travelled += 120) {
    await page.mouse.wheel({ deltaY: 120 });
    await pause(90);
  }
  await pause(1500);
}
const record = (await page.evaluate("window.__record")) as { frames: number[]; long: number[]; shifts: number };
await browser.close();

const frames = record.frames.slice(1).sort((a, b) => a - b);
const at = (share: number) => frames[Math.min(frames.length - 1, Math.floor(frames.length * share))];
const slow = frames.filter((frame) => frame > 1000 / 30).length;
console.log(
  JSON.stringify(
    {
      viewport: `${width}x${height}`,
      cpu: `${slowdown}x slower`,
      frames: frames.length,
      medianMs: Number(at(0.5).toFixed(1)),
      p95Ms: Number(at(0.95).toFixed(1)),
      over33ms: `${((slow / frames.length) * 100).toFixed(1)}%`,
      longAnimationFrames: record.long.length,
      longestMs: Math.round(Math.max(0, ...record.long)),
      cumulativeLayoutShift: Number(record.shifts.toFixed(4)),
    },
    null,
    2,
  ),
);
