import { createStore } from "zustand/vanilla";
import { store, stored } from "../shared/storage.js";

const RAIL_CLOSED = "sens.rail.closed";
const SIZES = "sens.sizes";
const TABS = "sens.tabs";

export type Tool = "files" | "changes" | "map" | "web" | "terminal" | "tasks";

export const TOOLS: Tool[] = ["files", "changes", "map", "web", "terminal", "tasks"];

const ROOMY = "(min-width: 860px)";

const keptTabs = (): Tool[] => {
  const kept = stored(TABS, []);
  return Array.isArray(kept) ? TOOLS.filter((tool) => kept.includes(tool)) : [];
};

export const shell = createStore(() => ({
  railClosed: stored(RAIL_CLOSED, false) === true,
  narrow: false,
  toolsOpen: false,
  tool: "files" as Tool,
  tabs: keptTabs(),
  treeShown: true,
  sizes: stored(SIZES, {}) as Record<string, number>,
  sizing: false,
}));

const set = shell.setState;

export const panelShows = (tool: Tool) => {
  const { toolsOpen, tool: shown } = shell.getState();
  return toolsOpen && shown === tool;
};

const entering: Partial<Record<Tool, () => void>> = {};
export const whenShown = (tool: Tool, enter: () => void) => void (entering[tool] = enter);

function keepTabs(tabs: Tool[]) {
  store(TABS, tabs);
  return tabs;
}

export function showTool(tool: Tool) {
  const { tabs } = shell.getState();
  set({ toolsOpen: true, tool, tabs: tabs.includes(tool) ? tabs : keepTabs([...tabs, tool]) });
  entering[tool]?.();
}

export const closeTools = () => set({ toolsOpen: false });

export function closeTab(tool: Tool) {
  const { tabs, tool: shown, toolsOpen } = shell.getState();
  const at = tabs.indexOf(tool);
  if (at < 0) return;
  const left = keepTabs(tabs.filter((one) => one !== tool));
  if (!left.length) return set({ tabs: left, toolsOpen: false });
  if (shown !== tool) return set({ tabs: left });
  const next = left[Math.min(at, left.length - 1)];
  set({ tabs: left, tool: next });
  if (toolsOpen) entering[next]?.();
}

type Layout = { railClosed: boolean; narrow: boolean; toolsOpen: boolean };

export const railFolded = ({ railClosed, narrow, toolsOpen }: Layout) => railClosed || (narrow && toolsOpen);

export function toggleRail() {
  const state = shell.getState();
  const railClosed = !railFolded(state);
  store(RAIL_CLOSED, railClosed);
  set({ railClosed, toolsOpen: state.toolsOpen && (railClosed || !state.narrow) });
}

export function watchWidth() {
  const roomy = matchMedia(ROOMY);
  const hear = () => set({ narrow: !roomy.matches });
  hear();
  roomy.addEventListener("change", hear);
}

export const toggleTree = () => set(({ treeShown }) => ({ treeShown: !treeShown }));

export function keepSize(name: string, width: number) {
  set(({ sizes }) => {
    const next = { ...sizes };
    if (width) next[name] = width;
    else delete next[name];
    store(SIZES, next);
    return { sizes: next };
  });
}

export const sizing = (on: boolean) => set({ sizing: on });
