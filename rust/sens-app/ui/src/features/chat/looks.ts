import type { Link, ToolDetail, ToolInput, Todo } from "../../ipc/types";
import { ICONS } from "../../shared/icons.js";
import { addedRows, patchRows, type Row } from "../../shared/rows";
import { RUN_IN_TERMINAL, shellOf, type Shell } from "../../shared/syntax/shells";
import { project } from "../project/store";
import { t as sens } from "./canon.copy";
import { t } from "./step.copy";

export interface Look {
  icon: string;
  verb: string;
  target: string;
  mono?: boolean;
  link?: string;
  site?: string;
  meta?: string;
  ask: string;
  shell?: Shell;
}

export const SILENT = new Set(["ToolSearch", "AskUserQuestion", "ExitPlanMode"]);
export const EDITS = new Set(["Edit", "MultiEdit", "Write", "NotebookEdit"]);
export const SHELLS = new Set(["Bash", "PowerShell", RUN_IN_TERMINAL]);

export function relative(path: string) {
  if (!path) return "";
  const { work: root } = project.getState();
  const clean = String(path).replace(/\\/g, "/");
  const base = root.replace(/\\/g, "/").replace(/\/$/, "");
  const inside = base && clean.toLowerCase().startsWith(`${base.toLowerCase()}/`);
  return inside ? clean.slice(base.length + 1) : clean;
}

export const oneLine = (text: unknown) => String(text || "").replace(/\s+/g, " ").trim();

export const progressOf = (todos: Todo[] = []) => `${todos.filter((todo) => todo.status === "completed").length}/${todos.length}`;

function pathLook(icon: string, verb: string, ask: string, input: ToolInput): Look {
  const target = relative(input.file_path || input.notebook_path || "");
  return { icon, verb, ask, target, mono: true, link: target };
}

function mcpLook(name: string): Look {
  const [, server, tool] = name.split("__");
  return { icon: ICONS.plug, verb: server, target: tool || "", mono: true, ask: t.wantsUse(server) };
}

const shellLook = (shell: Shell, input: ToolInput): Look => ({ icon: ICONS.terminal, verb: t.run, ask: t.wantsRun, target: oneLine(input.command), mono: true, shell });

const LOOKS: Record<string, (input: ToolInput) => Look> = {
  Read: (input) => pathLook(ICONS.fileText, t.read, t.wantsRead, input),
  Edit: (input) => pathLook(ICONS.pencil, t.edit, t.wantsEdit, input),
  MultiEdit: (input) => pathLook(ICONS.pencil, t.edit, t.wantsEdit, input),
  NotebookEdit: (input) => pathLook(ICONS.pencil, t.edit, t.wantsEdit, input),
  Write: (input) => pathLook(ICONS.filePlus, t.write, t.wantsWrite, input),
  Bash: (input) => shellLook("bash", input),
  PowerShell: (input) => shellLook("powershell", input),
  Grep: (input) => ({ icon: ICONS.search, verb: t.search, ask: t.wantsSearch, target: String(input.pattern ?? ""), mono: true, meta: input.path ? t.inPath(relative(input.path)) : "" }),
  Glob: (input) => ({ icon: ICONS.files, verb: t.list, ask: t.wantsList, target: String(input.pattern ?? ""), mono: true }),
  WebFetch: (input) => ({ icon: ICONS.globe, site: input.url, verb: t.read, ask: t.wantsOpen, target: String(input.url ?? ""), mono: true }),
  WebSearch: (input) => ({ icon: ICONS.globe, verb: t.searchWeb, ask: t.wantsSearchWeb, target: String(input.query ?? "") }),
  TodoWrite: (input) => ({ icon: ICONS.listChecks, verb: t.todos, ask: t.wantsTodos, target: progressOf(input.todos) }),
  Task: (input) => ({ icon: ICONS.split, verb: t.delegate, ask: t.wantsDelegate, target: input.description || input.subagent_type || "" }),
  Agent: (input) => ({ icon: ICONS.split, verb: t.delegate, ask: t.wantsDelegate, target: input.description || input.subagent_type || "" }),
  Skill: (input) => ({ icon: ICONS.book, verb: t.useSkill, ask: t.wantsSkill, target: String(input.skill || input.command || "") }),
  mcp__sens__read_terminal: () => ({ icon: ICONS.terminal, verb: t.readTerminal, ask: t.wantsReadTerminal, target: "" }),
  [RUN_IN_TERMINAL]: (input) => shellLook(shellOf(RUN_IN_TERMINAL, String(input.command ?? "")), input),
  mcp__sens__write_terminal: (input) => ({ icon: ICONS.keyboard, verb: t.typeTerminal, ask: t.wantsTypeTerminal, target: oneLine(input.text), mono: true }),
  mcp__sens__stop_terminal: (input) => ({ icon: ICONS.stopSquare, verb: t.stopTerminal, ask: t.wantsStopTerminal, target: String(input.terminal ?? "") }),
  mcp__sens__list_terminals: () => ({ icon: ICONS.terminal, verb: t.listTerminals, ask: t.wantsReadTerminal, target: "" }),
  mcp__sens__open_file: (input) => {
    const path = relative(String(input.path ?? ""));
    return { icon: ICONS.fileText, verb: t.showInSens, ask: t.wantsUse("Sens"), target: input.line ? `${path}:${input.line}` : path, mono: true, link: path };
  },
  mcp__sens__show_pane: (input) => ({ icon: ICONS.panelOpen, verb: t.openPane, ask: t.wantsUse("Sens"), target: String(input.pane ?? "") }),
  mcp__sens__close_pane: () => ({ icon: ICONS.panelClose, verb: t.closePane, ask: t.wantsUse("Sens"), target: "" }),
  mcp__sens__open_terminal_tab: () => ({ icon: ICONS.terminal, verb: t.openTerminal, ask: t.wantsUse("Sens"), target: "" }),
  mcp__sens__get_layout: () => ({ icon: ICONS.monitor, verb: t.lookLayout, ask: t.wantsUse("Sens"), target: "" }),
  mcp__sens__screenshot_app: () => ({ icon: ICONS.scan, verb: t.lookSens, ask: t.wantsUse("Sens"), target: "" }),
  mcp__sens__notify: (input) => ({ icon: ICONS.message, verb: t.notify, ask: t.wantsUse("Sens"), target: oneLine(input.body) }),
  mcp__sens__navigate: (input) => ({ icon: ICONS.globe, site: String(input.url ?? ""), verb: t.openPage, ask: t.wantsUse("Sens"), target: String(input.url ?? ""), mono: true }),
  mcp__sens__read_page: () => ({ icon: ICONS.globe, verb: t.readPage, ask: t.wantsUse("Sens"), target: "" }),
  mcp__sens__find: (input) => ({ icon: ICONS.search, verb: t.findOnPage, ask: t.wantsUse("Sens"), target: String(input.query ?? "") }),
  mcp__sens__page_text: () => ({ icon: ICONS.fileText, verb: t.readPageText, ask: t.wantsUse("Sens"), target: "" }),
  mcp__sens__click: (input) => ({ icon: ICONS.arrow, verb: t.clickPage, ask: t.wantsClick, target: String(input.ref ?? (input.x !== undefined ? `${input.x}, ${input.y}` : "")), mono: true }),
  mcp__sens__type_text: (input) => ({ icon: ICONS.keyboard, verb: t.typePage, ask: t.wantsTypePage, target: oneLine(input.text), mono: true }),
  mcp__sens__press_key: (input) => ({ icon: ICONS.keyboard, verb: t.pressKey, ask: t.wantsPressKey, target: String(input.key ?? ""), mono: true }),
  mcp__sens__scroll: (input) => ({ icon: ICONS.upDown, verb: t.scrollPage, ask: t.wantsUse("Sens"), target: String(input.ref ?? input.screens ?? "") }),
  mcp__sens__screenshot_page: () => ({ icon: ICONS.scan, verb: t.lookPage, ask: t.wantsUse("Sens"), target: "" }),
  mcp__sens__eval_js: (input) => ({ icon: ICONS.code, verb: t.runScript, ask: t.wantsRunScript, target: oneLine(input.expression), mono: true }),
  mcp__sens__console_logs: () => ({ icon: ICONS.terminal, verb: t.readConsole, ask: t.wantsUse("Sens"), target: "" }),
  mcp__sens__network_requests: (input) => ({ icon: ICONS.activity, verb: t.readNetwork, ask: t.wantsUse("Sens"), target: String(input.pattern ?? "") }),
  mcp__sens__resize_browser: (input) => ({ icon: ICONS.monitor, verb: t.pageWidth, ask: t.wantsUse("Sens"), target: input.width ? `${input.width} px` : "" }),
  "sens.dependency": (input) => ({ icon: ICONS.shieldAlert, verb: sens.sens, ask: sens.dependencyTitle, target: String(input.key ?? "").replace(/^R3:/, ""), mono: true }),
  "sens.tests": (input) => ({ icon: ICONS.shieldAlert, verb: sens.sens, ask: sens.testsTitle, target: relative(String(input.file ?? "")), mono: true, link: relative(String(input.file ?? "")) }),
  "sens.canon": (input) => ({ icon: ICONS.shieldAlert, verb: sens.sens, ask: sens.canonTitle, target: relative(String(input.file ?? "")), mono: true }),
};

export function describe(name: string, input: ToolInput = {}): Look {
  if (LOOKS[name]) return LOOKS[name](input);
  if (name.startsWith("mcp__")) return mcpLook(name);
  return { icon: ICONS.wrench, verb: name, ask: t.wantsUse(name), target: "" };
}

export function statusOf(name: string, input: ToolInput) {
  const look = describe(name, input);
  return [look.verb, look.target].filter(Boolean).join(" · ");
}

export function hostOf(url: string) {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

export const consulting = (links: Link[]) => (links.length === 1 ? t.consulting(hostOf(links[0].url)) : t.reviewing(links.length));

export interface Edit {
  path: string;
  rows: Row[];
  added: number[];
  plus: number;
  minus: number;
}

export function editOf(name: string, input: ToolInput, detail: ToolDetail | null): Edit | null {
  const path = relative(input.file_path || detail?.filePath || "");
  const patch = Array.isArray(detail?.structuredPatch) ? detail.structuredPatch : [];
  if (EDITS.has(name) && patch.length) return { path, ...patchRows(patch) };
  if (name === "Write" && detail?.type === "create") {
    const rows = addedRows(detail.content ?? input.content ?? "");
    return { path, rows, added: rows.map((row) => row.num!), plus: rows.length, minus: 0 };
  }
  return null;
}

export const hitsOf = (output: string) =>
  String(output || "")
    .split("\n")
    .map((line) => line.trimEnd())
    .filter((line) => line && !/^Found \d+ /.test(line) && !/^No (files|matches) found/.test(line));

export const searchSummary = (output: string) =>
  String(output || "")
    .replace(/^Web search results for query:.*\n+/, "")
    .replace(/^Links: \[.*\]\n*/m, "")
    .trim();
