import { viewer, showFile } from "../features/files/view";
import { panes } from "../features/panes/store";
import { project } from "../features/project/store";
import { consoles, openConsole } from "../features/terminal/store";
import { aim, goBack, goForward, pickWidth, reloadSite, web } from "../features/web/store";
import { answers } from "./acts";
import { closeTools, shell, showTool, type Tool } from "./shell";

interface Here {
  session: string;
}

const LOADING_PATIENCE = 15_000;
const MOVES: Record<string, () => unknown> = { back: goBack, forward: goForward, reload: reloadSite };

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

function present({ session }: Here) {
  here(session);
  return "";
}

function loaded() {
  return new Promise<boolean>((done) => {
    let started = false;
    const timer = setTimeout(() => {
      stop();
      done(false);
    }, LOADING_PATIENCE);
    const stop = web.subscribe(({ loading }) => {
      if (loading) started = true;
      else if (started) {
        stop();
        clearTimeout(timer);
        done(true);
      }
    });
  });
}

async function browse({ session, url }: Here & { url: string }) {
  here(session);
  const finished = loaded();
  await (MOVES[url] ?? (() => aim(url)))();
  showTool("web");
  const done = await finished;
  const { url: now, title } = web.getState();
  if (!now) throw new Error(`Sens's browser could not open ${url}.`);
  return `${done ? "Loaded" : "Still loading"} ${now}${title ? ` · ${title}` : ""}`;
}

function browserWidth({ session, width }: Here & { width: number }) {
  here(session);
  pickWidth(width);
  showTool("web");
  return width ? `The page is drawn ${width} px wide.` : "The page is drawn at the pane's own width.";
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
  answers("present", present);
  answers("browse", browse);
  answers("browser_width", browserWidth);
}
