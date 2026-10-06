import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const sens = path.resolve(process.env.SENS_REPO ?? path.join(root, ".."));
const steps = path.join(sens, "bench", "results", "2026-10-01-horizonte-confirmacion", "steps.jsonl");
const figure = path.join(sens, "docs", "paper", "figuras", "2-horizonte-lineas.svg");
const canonText = path.join(sens, "rust", "sens-canon", "src", "canon.md");
const target = path.join(root, "src", "data", "research.json");

interface Step {
  condition: string;
  rep: number;
  step: number;
  tokens_in: number;
  tokens_out: number;
  accepted: boolean;
  broken: unknown[];
  shape: { lines: number; functions: number };
}

const BASE_LINES = 88;
const FIGURE_BASELINE = 328;
const FIGURE_PIXELS_PER_LINE = 0.5;
const REFERENCE_STROKE = "#363b38";

const runs = readFileSync(steps, "utf8")
  .split("\n")
  .filter(Boolean)
  .map((line) => JSON.parse(line) as Step);

const arms = { C0: "without", C2: "with" } as const;

const sequences = Object.entries(arms).flatMap(([condition, arm]) =>
  [1, 2, 3].map((rep) => {
    const ordered = runs.filter((run) => run.condition === condition && run.rep === rep).sort((a, b) => a.step - b.step);
    let spent = 0;
    return {
      arm,
      rep,
      lines: [BASE_LINES, ...ordered.map((run) => run.shape.lines)],
      tokens: [0, ...ordered.map((run) => (spent += run.tokens_in + run.tokens_out))],
      functions: ordered.at(-1)?.shape.functions ?? 0,
      accepted: ordered.filter((run) => run.accepted).length,
      broken: ordered.reduce((sum, run) => sum + run.broken.length, 0),
    };
  }),
);

const drawn = readFileSync(figure, "utf8");
const referencePath = [...drawn.matchAll(/<path d="([^"]+)"[^>]*stroke="([^"]+)"/g)].find(([, , stroke]) => stroke === REFERENCE_STROKE)?.[1];
if (!referencePath) throw new Error("the reference line is missing from the paper's figure 2");
const reference = [...referencePath.matchAll(/,([\d.]+)/g)].map(([, y]) => Math.round((FIGURE_BASELINE - Number(y)) / FIGURE_PIXELS_PER_LINE));

const canon = readFileSync(canonText, "utf8").trim();

writeFileSync(target, `${JSON.stringify({ sequences, reference, canon }, null, 2)}\n`);
console.log(`research · ${sequences.length} sequences, reference ends at ${reference.at(-1)} lines`);
