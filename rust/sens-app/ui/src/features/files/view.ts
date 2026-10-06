import { createStore } from "zustand/vanilla";
import { commands } from "../../ipc/commands";
import type { Opened } from "../../ipc/types";
import { showTool } from "../../app/shell";
import { MARKDOWN, PAGE } from "../../shared/format.js";
import { project } from "../project/store";
import { showSite } from "../web/store";
import { revealFile } from "./store";

export type Mode = "source" | "view";

export type Body = Opened | { kind: "fault"; fault: string };

const NOTHING: Body = { kind: "text", text: "" };

export const viewer = createStore(() => ({
  title: "",
  body: NOTHING as Body,
  home: "",
  opened: "",
  mode: "source" as Mode,
  shown: 0,
  line: 0,
}));

const set = viewer.setState;

export const viewOf = (title: string) => (MARKDOWN.test(title) ? "reading" : PAGE.test(title) ? "site" : "");

export const textOf = (body: Body) => (body.kind === "text" ? body.text : "");

let reads = 0;

function show(title: string, body: Body, home: string, opened: string, line = 0) {
  reads++;
  const mode: Mode = body.kind === "text" && viewOf(title) === "reading" && !line ? "view" : "source";
  set(({ shown }) => ({ title, body, home, opened, mode, line, shown: shown + 1 }));
}

export function present(title: string, text: string, home: string, opened = "") {
  show(title, { kind: "text", text }, home, opened);
}

export async function openFile(path: string, line = 0) {
  const { work: root } = project.getState();
  const { opened, mode, body: before } = viewer.getState();
  const mine = ++reads;
  revealFile(path);
  const body: Body = await commands.openFile(root, path).catch((reason) => ({ kind: "fault" as const, fault: String(reason) }));
  if (mine !== reads) return;
  show(path, body, root, path, line);
  if (opened === path && mode === "source" && before.kind === "text") set({ mode: "source" });
}

export function showFile(path: string, line = 0) {
  showTool("files");
  return openFile(path, line);
}

export function setMode(mode: Mode) {
  const { title, home } = viewer.getState();
  if (mode === "view" && viewOf(title) === "site") return void showSite(title, home);
  set({ mode });
}

export function forgetViewer() {
  reads++;
  set(({ shown }) => ({ title: "", body: NOTHING, home: "", opened: "", mode: "source", line: 0, shown: shown + 1 }));
}
