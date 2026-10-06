export interface Segment {
  text: string;
  tone?: "green" | "red" | "ghost";
}

export type Line = Segment[];

interface Base {
  id: string;
  seconds: number;
}

export interface Thought extends Base {
  kind: "thought";
  title: string;
}

export interface Read extends Base {
  kind: "read";
  path: string;
  meta: string;
}

export interface Search extends Base {
  kind: "search";
  query: string;
  meta: string;
}

export interface Run extends Base {
  kind: "run";
  command: string;
  output: Line[];
}

export interface Edit extends Base {
  kind: "edit";
  path: string;
  plus: number;
  minus: number;
}

export interface Found {
  signature: string;
  place: string;
  uses: string;
}

export interface Finding {
  rule: string;
  place: string;
  existing: { place: string; code: string };
}

export interface Sens extends Base {
  kind: "sens";
  tone: "found" | "stop" | "pass";
  said: string;
  found?: Found;
  finding?: Finding;
}

export type Step = Thought | Read | Search | Run | Edit | Sens;

export type Mark = " " | "+" | "−";

export interface Row {
  num: number;
  mark: Mark;
  src: string;
}

export interface Change {
  id: string;
  state: "M" | "A";
  name: string;
  dir: string;
  icon: string;
  plus: number;
  minus: number;
  rows: Row[];
}

const passed = (files: number, tests: number): Line[] => [
  [{ text: " Test Files  " }, { text: `${files} passed`, tone: "green" }, { text: ` (${files})`, tone: "ghost" }],
  [{ text: "      Tests  " }, { text: `${tests} passed`, tone: "green" }, { text: ` (${tests})`, tone: "ghost" }],
];

const suite = (header: number, user: number): Line[] => [
  [{ text: " ✓", tone: "green" }, { text: " src/header.test.tsx " }, { text: `(${header} tests)`, tone: "ghost" }],
  [{ text: " ✓", tone: "green" }, { text: " src/user.test.ts " }, { text: `(${user} tests)`, tone: "ghost" }],
  [],
  ...passed(2, header + user),
];

const greetingFor = [
  'export function greetingFor(name: string, greeting = "Welcome") {',
  "  return `${greeting}, ${name}`;",
  "}",
].join("\n");

export const steps: Step[] = [
  {
    id: "sens-found",
    kind: "sens",
    tone: "found",
    said: "already has greetingFor",
    found: { signature: 'greetingFor(name: string, greeting = "Welcome")', place: "src/greeting.ts:1", uses: "used 3 times" },
    seconds: 0,
  },
  { id: "thought-read", kind: "thought", title: "Reading how the header is built", seconds: 0.6 },
  { id: "read", kind: "read", path: "src/header.tsx", meta: "13 lines", seconds: 0.2 },
  { id: "search", kind: "search", query: "<h1>", meta: "1 result", seconds: 0.3 },
  { id: "thought-tests", kind: "thought", title: "Checking the tests before changing anything", seconds: 0.8 },
  { id: "run-before", kind: "run", command: "npm test", output: suite(2, 8), seconds: 1.4 },
  { id: "thought-prop", kind: "thought", title: "Adding the prop with its default", seconds: 0.7 },
  {
    id: "sens-stop",
    kind: "sens",
    tone: "stop",
    said: "stopped a write",
    finding: { rule: "Copies existing code", place: "src/header.tsx:7", existing: { place: "src/greeting.ts:1", code: greetingFor } },
    seconds: 0,
  },
  { id: "thought-reuse", kind: "thought", title: "Using greetingFor instead", seconds: 0.4 },
  { id: "edit-header", kind: "edit", path: "src/header.tsx", plus: 4, minus: 2, seconds: 0.3 },
  { id: "edit-test", kind: "edit", path: "src/header.test.tsx", plus: 9, minus: 0, seconds: 0.4 },
  { id: "run-after", kind: "run", command: "npm test", output: suite(4, 8), seconds: 1.5 },
  { id: "sens-pass", kind: "sens", tone: "pass", said: "approved the changes", seconds: 0 },
];

export const changes: Change[] = [
  {
    id: "header",
    state: "M",
    name: "header.tsx",
    dir: "src",
    icon: "react_ts",
    plus: 4,
    minus: 2,
    rows: [
      { num: 1, mark: "+", src: 'import { greetingFor } from "./greeting";' },
      { num: 2, mark: " ", src: 'import type { User } from "./user";' },
      { num: 3, mark: " ", src: "" },
      { num: 4, mark: " ", src: "interface Props {" },
      { num: 5, mark: " ", src: "  user: User;" },
      { num: 6, mark: "+", src: "  greeting?: string;" },
      { num: 7, mark: " ", src: "}" },
      { num: 8, mark: " ", src: "" },
      { num: 7, mark: "−", src: "export function Header({ user }: Props) {" },
      { num: 9, mark: "+", src: "export function Header({ user, greeting }: Props) {" },
      { num: 10, mark: " ", src: "  return (" },
      { num: 11, mark: " ", src: '    <header className="header">' },
      { num: 10, mark: "−", src: "      <h1>{user.name}</h1>" },
      { num: 12, mark: "+", src: "      <h1>{greetingFor(user.name, greeting)}</h1>" },
      { num: 13, mark: " ", src: "    </header>" },
      { num: 14, mark: " ", src: "  );" },
      { num: 15, mark: " ", src: "}" },
    ],
  },
  {
    id: "test",
    state: "M",
    name: "header.test.tsx",
    dir: "src",
    icon: "test-jsx",
    plus: 9,
    minus: 0,
    rows: [],
  },
];

export const session = {
  project: "orbit",
  branch: "main",
  request: "The header should greet people by name, with a greeting that defaults to Welcome. Add a test for it.",
  answer: {
    lead: ["Done. The header greets people by name now, and falls back to ", "Welcome", " when no greeting is given:"],
    code: { label: "TSX", source: '<Header user={alex} />\n<Header user={alex} greeting="Good morning" />' },
    bullets: [
      [{ code: "src/header.tsx" }, { text: " takes an optional " }, { code: "greeting" }, { text: " prop and reuses " }, { code: "greetingFor" }],
      [{ code: "src/header.test.tsx" }, { text: " covers both cases, and the suite passes: " }, { strong: "12 tests" }],
    ],
  },
  foot: { took: "7.6 s", tokens: "2.1k" },
  knobs: { model: "Opus 5.5 (1M)", mode: "Ask", context: 31, effort: "Medium" },
} as const;

const count = (amount: number, one: string, many: string) => (amount === 1 ? `1 ${one}` : `${amount} ${many}`);

export function sensTallyOf(list: Step[]): { quiet: string; lit: string } {
  const stops = list.filter((step) => step.kind === "sens" && step.tone === "stop").length;
  const reused = list.filter((step) => step.kind === "sens" && step.tone === "found").length;
  const approved = list.some((step) => step.kind === "sens" && step.tone === "pass");
  return {
    quiet: stops ? count(stops, "stop", "stops") : "",
    lit: [reused && `${reused} reused`, approved && "approved"].filter(Boolean).join(" · "),
  };
}

export function tallyOf(list: Step[]): string {
  const of = (kind: Step["kind"]) => list.filter((step) => step.kind === kind).length;
  return [
    count(of("run"), "command", "commands"),
    count(of("read"), "file read", "files read"),
    count(of("edit"), "edit", "edits"),
    count(of("search"), "search", "searches"),
  ].join(" · ");
}

export const secondsOf = (list: Step[]): string => `${list.reduce((sum, step) => sum + step.seconds, 0).toFixed(1)} s`;

export const totalsOf = (list: Change[]) => ({
  files: list.length,
  plus: list.reduce((sum, change) => sum + change.plus, 0),
  minus: list.reduce((sum, change) => sum + change.minus, 0),
});
