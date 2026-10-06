import { viewer, showFile } from "../features/files/view";
import { panes } from "../features/panes/store";
import { project } from "../features/project/store";
import { consoles, openConsole } from "../features/terminal/store";
import { answers } from "./acts";
import { closeTools, shell, showTool, type Tool } from "./shell";

interface Here {
  session: string;
}

const ELSEWHERE = "The person is looking at another session, so Sens left their screen as it was.";

function here(session: string) {
  if (project.getState().session !== session) throw new Error(ELSEWHERE);
}

async function openFileAt({ session, path, line }: Here & { path: string; line: number | null }) {
  here(session);
  await showFile(path, line ?? 0);
  const { body } = viewer.getState();
  if (body.kind === "fault") throw new Error(body.fault);
  return line ? `Showing ${path} at line ${line}.` : `Showing ${path}.`;
}

function showPane({ session, pane }: Here & { pane: Tool }) {
  here(session);
  showTool(pane);
  return `The ${pane} pane is open.`;
}

function closePane({ session }: Here) {
  here(session);
  closeTools();
  return "The side pane is closed.";
}

async function openTerminalTab({ session, root }: Here & { root: string }) {
  here(session);
  showTool("terminal");
  await openConsole(root);
  return `Terminal ${consoles.getState().shown} is open for the person.`;
}

export function layoutOf({ session }: Here) {
  const { session: seen, work } = project.getState();
  const { toolsOpen, tool, treeShown } = shell.getState();
  const { title } = viewer.getState();
  const open = consoles.getState().open;
  const yours = open.filter((one) => one.title).length;
  const chats = panes.getState().open.length;
  return [
    seen === session ? "The person is looking at this session." : `The person is looking at another session${work ? `, in ${work}` : ""}.`,
    toolsOpen ? `Side pane: ${tool}${tool === "files" ? `${title ? `, showing ${title}` : ", no file open"}${treeShown ? ", with the tree" : ""}` : ""}.` : "The side pane is closed.",
    `Terminals: ${open.length} open${yours ? `, ${yours} started by Claude` : ""}.`,
    chats > 1 ? `${chats} chats side by side.` : "One chat on screen.",
  ].join("\n");
}

export function answerSurface() {
  answers("open_file", openFileAt);
  answers("show_pane", showPane);
  answers("close_pane", closePane);
  answers("open_terminal_tab", openTerminalTab);
  answers("get_layout", layoutOf);
}
