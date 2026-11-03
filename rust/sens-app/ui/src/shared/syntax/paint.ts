import type { Look, Painted } from "./tokenize";

export type { Look, Painted } from "./tokenize";
export type Runs = [string, number][];

const LONGEST_TEXT = 1_000_000;
const KEPT_TEXT = 4_000_000;

const kept = new Map<string, Painted | null>();
let keptText = 0;

interface Job {
  key: string;
  text: string;
  language: string;
  wanted: () => boolean;
  done: (painted: Painted | null) => void;
}
const waiting: Job[] = [];
let turning = false;
let tokenizer: ((text: string, language: string) => Promise<Painted | null>) | null = null;

const keyOf = (text: string, language: string) => `${language}\n${text}`;

export function paintedNow(text: string, language: string | null) {
  return (language && kept.get(keyOf(text, language))) || null;
}

export function paint(text: string, language: string | null, wanted = () => true): Promise<Painted | null> {
  if (!language || text.length > LONGEST_TEXT) return Promise.resolve(null);
  const key = keyOf(text, language);
  if (kept.has(key)) return Promise.resolve(kept.get(key) ?? null);
  return new Promise((done) => {
    waiting.push({ key, text, language, wanted, done });
    if (turning) return;
    turning = true;
    setTimeout(turn);
  });
}

async function turn() {
  let job = waiting.shift();
  while (job && !job.wanted()) {
    job.done(null);
    job = waiting.shift();
  }
  if (!job) {
    turning = false;
    return;
  }
  const painted = kept.has(job.key) ? (kept.get(job.key) ?? null) : await tokenize(job.text, job.language);
  keep(job.key, painted);
  job.done(painted);
  turn();
}

function keep(key: string, painted: Painted | null) {
  if (kept.has(key)) return;
  kept.set(key, painted);
  keptText += key.length;
  for (const [old] of kept) {
    if (keptText <= KEPT_TEXT) break;
    kept.delete(old);
    keptText -= old.length;
  }
}

function tokenize(text: string, language: string) {
  tokenizer ??= import.meta.env.MODE === "test" ? inPage : inWorker();
  return tokenizer(text, language);
}

const inPage = (text: string, language: string) =>
  import("./tokenize").then((module) => module.tokenize(text, language)).catch(() => null);

function inWorker() {
  const worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
  const answers = new Map<number, (painted: Painted | null) => void>();
  let asked = 0;
  worker.addEventListener("message", ({ data }: MessageEvent<{ id: number; painted: Painted | null }>) => {
    answers.get(data.id)?.(data.painted);
    answers.delete(data.id);
  });
  worker.addEventListener("error", () => {
    tokenizer = async () => null;
    for (const answer of answers.values()) answer(null);
    answers.clear();
  });
  return (text: string, language: string) =>
    new Promise<Painted | null>((answer) => {
      answers.set(++asked, answer);
      worker.postMessage({ id: asked, text, language });
    });
}

export interface Row {
  kind: string;
  text: string;
}

export async function paintRows(rows: Row[], language: string | null, wanted = () => true) {
  const before = rows.flatMap((row, at) => (row.kind === "" || row.kind === "del" ? [at] : []));
  const after = rows.flatMap((row, at) => (row.kind === "" || row.kind === "add" ? [at] : []));
  const removes = rows.some((row) => row.kind === "del");
  const [old, fresh] = await Promise.all([
    removes ? paint(before.map((at) => rows[at].text).join("\n"), language, wanted) : null,
    paint(after.map((at) => rows[at].text).join("\n"), language, wanted),
  ]);
  const out: ({ runs: Runs; looks: Look[] } | null)[] = rows.map(() => null);
  const fill = (side: number[], painted: Painted | null, kinds: string[]) =>
    side.forEach((at, line) => {
      if (painted?.lines[line] && kinds.includes(rows[at].kind)) out[at] = { runs: painted.lines[line], looks: painted.looks };
    });
  fill(before, old, ["del"]);
  fill(after, fresh, ["", "add"]);
  return out;
}
