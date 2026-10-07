import { createStore } from "zustand/vanilla";
import { store, stored } from "../shared/storage.js";

const ARRANGEMENT = "sens.arrangement";

export type Tool = "files" | "changes" | "map" | "web" | "terminal" | "tasks";

export const TOOLS: Tool[] = ["files", "changes", "map", "web", "terminal", "tasks"];

export type Dock = "start" | "end" | "bottom";

export const DOCKS: Dock[] = ["start", "end", "bottom"];

export type Side = "start" | "end";

export type Docked<T> = Record<Dock, T>;

export interface Arrangement {
  rail: Side;
  railClosed: boolean;
  wide: boolean;
  calm: boolean;
  homes: Record<Tool, Dock>;
  tabs: Tool[];
  shown: Docked<Tool | null>;
  open: Docked<boolean>;
  sizes: Record<string, number>;
}

export const HOMES: Record<Tool, Dock> = { files: "end", changes: "end", map: "end", web: "end", terminal: "bottom", tasks: "end" };

const ROOMY = "(min-width: 860px)";

const isTool = (value: unknown): value is Tool => TOOLS.includes(value as Tool);
const isDock = (value: unknown): value is Dock => DOCKS.includes(value as Dock);
const docked = <T>(each: (dock: Dock) => T) => Object.fromEntries(DOCKS.map((dock) => [dock, each(dock)])) as Docked<T>;

export const tabsIn = ({ tabs, homes }: Pick<Arrangement, "tabs" | "homes">, dock: Dock) => tabs.filter((tool) => homes[tool] === dock);

export function readArrangement(value: unknown): Arrangement {
  const kept = (value && typeof value === "object" ? value : {}) as Partial<Arrangement>;
  const homes = { ...HOMES };
  for (const tool of TOOLS) if (isDock(kept.homes?.[tool])) homes[tool] = kept.homes[tool];
  const tabs = Array.isArray(kept.tabs) ? [...new Set(kept.tabs.filter(isTool))] : [];
  const shown = docked((dock) => {
    const tool = kept.shown?.[dock];
    return isTool(tool) && tabs.includes(tool) && homes[tool] === dock ? tool : (tabsIn({ tabs, homes }, dock)[0] ?? null);
  });
  const sizes = Object.fromEntries(Object.entries(kept.sizes ?? {}).filter(([, size]) => Number.isFinite(size) && size > 0));
  return {
    rail: kept.rail === "end" ? "end" : "start",
    railClosed: kept.railClosed === true,
    wide: kept.wide === true,
    calm: kept.calm === true,
    homes,
    tabs,
    shown,
    open: docked((dock) => kept.open?.[dock] === true && shown[dock] !== null),
    sizes,
  };
}

export const arrangementOf = ({ rail, railClosed, wide, calm, homes, tabs, shown, open, sizes }: Arrangement): Arrangement => ({
  rail,
  railClosed,
  wide,
  calm,
  homes,
  tabs,
  shown,
  open,
  sizes,
});

function formerArrangement() {
  const { "--tools-width": end, ...sizes } = stored("sens.sizes", {}) as Record<string, number>;
  return { railClosed: stored("sens.rail.closed", false), tabs: stored("sens.tabs", []), sizes: end ? { ...sizes, "--end-width": end } : sizes };
}

export const shell = createStore(() => ({
  ...readArrangement(stored(ARRANGEMENT, null) ?? formerArrangement()),
  narrow: false,
  treeShown: true,
  sizing: "" as "" | "x" | "y",
}));

type State = ReturnType<typeof shell.getState>;

const set = shell.setState;

let kept = JSON.stringify(arrangementOf(shell.getState()));
shell.subscribe((now) => {
  const next = JSON.stringify(arrangementOf(now));
  if (next === kept) return;
  kept = next;
  store(ARRANGEMENT, arrangementOf(now));
});

export const visibleIn = (state: Pick<Arrangement, "calm" | "open" | "shown">, dock: Dock) => (!state.calm && state.open[dock] ? state.shown[dock] : null);

export const panelShows = (tool: Tool) => {
  const state = shell.getState();
  return visibleIn(state, state.homes[tool]) === tool;
};

export const toolsShown = (state: State = shell.getState()) => DOCKS.some((dock) => visibleIn(state, dock));

const entering: Partial<Record<Tool, () => void>> = {};
export const whenShown = (tool: Tool, enter: () => void) => void (entering[tool] = enter);

shell.subscribe((now, before) => {
  for (const dock of DOCKS) {
    const tool = visibleIn(now, dock);
    if (tool && tool !== visibleIn(before, dock)) entering[tool]?.();
  }
});

function nextIn(state: Pick<Arrangement, "tabs" | "homes">, dock: Dock, gone: Tool) {
  const list = tabsIn(state, dock);
  const rest = list.filter((tool) => tool !== gone);
  return rest[Math.min(Math.max(0, list.indexOf(gone)), rest.length - 1)] ?? null;
}

export function moveTool(tool: Tool, dock: Dock, before?: Tool | null) {
  const state = shell.getState();
  const was = panelShows(tool);
  const from = state.homes[tool];
  const rest = state.tabs.filter((one) => one !== tool);
  const at = before ? rest.indexOf(before) : -1;
  const tabs = at < 0 ? [...rest, tool] : [...rest.slice(0, at), tool, ...rest.slice(at)];
  const left = from !== dock && state.shown[from] === tool ? nextIn(state, from, tool) : state.shown[from];
  set({
    calm: false,
    tabs,
    homes: { ...state.homes, [tool]: dock },
    shown: { ...state.shown, [from]: left, [dock]: tool },
    open: { ...state.open, [from]: state.open[from] && left !== null, [dock]: true },
  });
  if (was && from === dock) entering[tool]?.();
}

export const showTool = (tool: Tool) => moveTool(tool, shell.getState().homes[tool], shell.getState().tabs.includes(tool) ? nextAfter(tool) : null);

function nextAfter(tool: Tool) {
  const { tabs } = shell.getState();
  return tabs[tabs.indexOf(tool) + 1] ?? null;
}

export function closeTab(tool: Tool) {
  const state = shell.getState();
  if (!state.tabs.includes(tool)) return;
  const dock = state.homes[tool];
  const shown = state.shown[dock] === tool ? nextIn(state, dock, tool) : state.shown[dock];
  set({ tabs: state.tabs.filter((one) => one !== tool), shown: { ...state.shown, [dock]: shown }, open: { ...state.open, [dock]: state.open[dock] && shown !== null } });
}

export const closeDock = (dock: Dock) => set(({ open }) => ({ open: { ...open, [dock]: false } }));

export const closeTools = () => set({ open: docked(() => false) });

export function hideTool(tool: Tool) {
  if (panelShows(tool)) closeDock(shell.getState().homes[tool]);
}

export function toggleDock(dock: Dock) {
  const state = shell.getState();
  if (visibleIn(state, dock)) return closeDock(dock);
  const tool = state.shown[dock] ?? tabsIn(state, dock)[0];
  if (tool) moveTool(tool, dock, nextAfter(tool));
}

export const placeRail = (rail: Side) => set({ rail });

export const spanBottom = (wide: boolean) => set({ wide });

export const calmDown = (calm: boolean) => set({ calm });

export const arrange = (arrangement: Arrangement) => set(readArrangement(arrangement));

export const railFolded = (state: Pick<State, "railClosed" | "narrow" | "calm" | "open" | "shown">) =>
  state.railClosed || (state.narrow && Boolean(visibleIn(state, "start") || visibleIn(state, "end")));

export function toggleRail() {
  const state = shell.getState();
  const railClosed = !railFolded(state);
  set({ railClosed, open: state.narrow && !railClosed ? { ...state.open, start: false, end: false } : state.open });
}

export function watchWidth() {
  const roomy = matchMedia(ROOMY);
  const hear = () => set({ narrow: !roomy.matches });
  hear();
  roomy.addEventListener("change", hear);
}

export const toggleTree = () => set(({ treeShown }) => ({ treeShown: !treeShown }));

export function keepSize(name: string, size: number) {
  set(({ sizes }) => {
    const next = { ...sizes };
    if (size) next[name] = size;
    else delete next[name];
    return { sizes: next };
  });
}

export const sizing = (on: boolean, axis: "x" | "y" = "x") => set({ sizing: on ? axis : "" });
