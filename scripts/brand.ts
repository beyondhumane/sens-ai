import { copyFileSync, mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { heavySensPath } from "./display";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const sens = path.resolve(process.env.SENS_REPO ?? path.join(root, "..", "Sens"));
const brandSource = path.join(sens, "src", "brand");

const fromSens = async <T>(file: string): Promise<T> =>
  import(pathToFileURL(path.join(brandSource, file)).href) as Promise<T>;

interface Tokens {
  palette: Record<string, string>;
  fontSans: string;
  fontMono: string;
  radius: Record<string, number>;
  motion: Record<string, string>;
}

interface Mark {
  markSvg(options: { size?: number; micro?: boolean; id?: string; label?: string }): string;
  markSize: number;
  markRadius: number;
  cutPath: string;
  cutWidth: number;
  microCutWidth: number;
}

const WIDEN = 1.3;

function widen(d: string, factor: number): string {
  let index = 0;
  return d.replace(/-?\d*\.?\d+/g, (value) => (index++ % 2 === 0 ? String(Number((Number(value) * factor).toFixed(2))) : value));
}

const written: string[] = [];

function write(file: string, contents: string): void {
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, contents);
  written.push(path.relative(root, file).replace(/\\/g, "/"));
}

function copy(from: string, to: string): void {
  mkdirSync(path.dirname(to), { recursive: true });
  copyFileSync(from, to);
  written.push(path.relative(root, to).replace(/\\/g, "/"));
}

function tokensCss(palette: Tokens["palette"], wordmark: Box): string {
  const lines = Object.entries(palette).map(([token, value]) => `  --sens-${token}: ${value};`);
  const ratio = `  --sens-wordmark-ratio: ${Number((wordmark.height / wordmark.width).toFixed(4))};`;
  return [":root {", ...lines, ratio, "}", ""].join("\n");
}

type Box = ReturnType<typeof extentOf>;

function extentOf(d: string) {
  const numbers = (d.match(/-?\d*\.?\d+/g) ?? []).map(Number);
  const xs = numbers.filter((_, at) => at % 2 === 0);
  const ys = numbers.filter((_, at) => at % 2 === 1);
  const x = Math.min(...xs);
  const y = Math.min(...ys);
  const round = (value: number) => Math.round(value * 100) / 100;
  return { x: round(x), y: round(y), width: round(Math.max(...xs) - x), height: round(Math.max(...ys) - y) };
}

function wordmarkSvg(d: string, box: Box): string {
  return [
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${box.x} ${box.y} ${box.width} ${box.height}" aria-hidden="true" focusable="false">`,
    `<path fill="currentColor" d="${d}"/>`,
    "</svg>",
    "",
  ].join("");
}

async function main(): Promise<void> {
  const tokens = await fromSens<Tokens>("tokens.ts");
  const mark = await fromSens<Mark>("mark.ts");
  const wordmark = widen(heavySensPath, WIDEN);
  const wordmarkBox = extentOf(wordmark);

  write(path.join(root, "src", "styles", "tokens.css"), tokensCss(tokens.palette, wordmarkBox));

  write(
    path.join(root, "src", "brand", "brand.json"),
    `${JSON.stringify(
      {
        palette: tokens.palette,
        fontSans: tokens.fontSans,
        fontMono: tokens.fontMono,
        radius: tokens.radius,
        motion: tokens.motion,
        mark: {
          size: mark.markSize,
          radius: mark.markRadius,
          cut: mark.cutPath,
          cutWidth: mark.cutWidth,
          microCutWidth: mark.microCutWidth,
        },
        wordmarkBox,
      },
      null,
      2,
    )}\n`,
  );

  write(path.join(root, "src", "brand", "wordmark.svg"), wordmarkSvg(wordmark, wordmarkBox));
  write(path.join(root, "public", "brand", "mark.svg"), `${mark.markSvg({ size: 48, label: "Sens" })}\n`);
  write(path.join(root, "public", "favicon.svg"), `${mark.markSvg({ size: 32, micro: true, id: "sens-favicon", label: "Sens" })}\n`);

  const fonts = path.join(sens, "rust", "sens-app", "ui", "public", "fonts");
  copy(path.join(fonts, "space-grotesk.woff2"), path.join(root, "public", "fonts", "space-grotesk.woff2"));
  copy(path.join(fonts, "OFL.txt"), path.join(root, "public", "fonts", "space-grotesk-OFL.txt"));

  const geist = path.join(root, "node_modules", "@fontsource-variable", "geist-mono");
  copy(path.join(geist, "files", "geist-mono-latin-wght-normal.woff2"), path.join(root, "public", "fonts", "geist-mono.woff2"));
  copy(path.join(geist, "LICENSE"), path.join(root, "public", "fonts", "geist-mono-OFL.txt"));

  const theme = path.join(sens, "node_modules", "material-icon-theme");
  for (const icon of ["react_ts", "test-jsx"]) {
    copy(path.join(theme, "icons", `${icon}.svg`), path.join(root, "public", "file-icons", `${icon}.svg`));
  }
  copy(path.join(theme, "LICENSE"), path.join(root, "public", "file-icons", "LICENSE"));

  console.log(`brand · ${written.length} files from ${path.relative(root, sens) || sens}`);
  for (const file of written) console.log(`  ${file}`);
}

await main();
