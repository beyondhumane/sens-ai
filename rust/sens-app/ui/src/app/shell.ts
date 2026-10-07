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
  paired: Docked<Tool | null>;
  open: Docked<boolean>;
  sizes: Record<string, number>;
}

export const HOMES: Record<Tool, Dock> = { files: "end", changes: "end", map: "end", web: "end", terminal: "bottom", tasks: "end" };

export const NO_PAIRS: Docked<Tool | null> = { start: null, end: null, bottom: null };

const ROOMY = "(min-width: 860px)";

const isTool = (value: unknown): value is Tool => TOOLS.includes(value as Tool);
const isDock = (value: unknown): value is Dock => DOCKS.includes(value as Dock);
const docked = <T>(each: (dock: Dock) => T) => Object.fromEntries(DOCKS.map((dock) => [dock, each(dock)])) as Docked<T>;

export const tabsIn = ({ tabs, homes }: Pick<Arrangement, "tabs" | "homes">, dock: Dock) => tabs.filter((tool) => homes[tool] === dock);

export const mainTabsIn = (state: Pick<Arrangement, "tabs" | "homes" | "paired">, dock: Dock) => tabsIn(state, dock).filter((tool) => tool !== state.paired[dock]);

export function readArrangement(value: unknown): Arrangement {
  const kept = (value && typeof value === "object" ? value : {}) as Partial<Arrangement>;
  const homes = { ...HOMES };
  for (const tool of TOOLS) if (isDock(kept.homes?.[tool])) homes[tool] = kept.homes[tool];
  const tabs = Array.isArray(kept.tabs) ? [...new Set(kept.tabs.filter(isTool))] : [];
  const fits = (tool: unknown, dock: Dock): tool is Tool => isTool(tool) && tabs.includes(tool) && homes[tool] === dock;
  const shown = docked((dock) => {
    const tool = kept.shown?.[dock];
    return fits(tool, dock) ? tool : (tabsIn({ tabs, homes }, dock).find((one) => one !== kept.paired?.[dock]) ?? null);
  });
  const paired = docked((dock) => {
    const tool = kept.paired?.[dock];
    return fits(tool, dock) && shown[dock] !== null && tool !== shown[dock] ? tool : null;
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
    paired,
    open: docked((dock) => kept.open?.[dock] === true && shown[dock] !== null),
    sizes,
  };
}

export const arrangementOf = ({ rail, railClosed, wide, calm, homes, tabs, shown, paired, open, sizes }: Arrangement): Arrangement => ({
  rail,
  railClosed,
  wide,
  calm,
  homes,
  tabs,
  shown,
  paired,
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

type Visible = Pick<Arrangement, "calm" | "open" | "shown">;

export const visibleIn = (state: Visible, dock: Dock) => (!state.calm && state.open[dock] ? state.shown[dock] : null);

export const pairedIn = (state: Visible & Pick<Arrangement, "paired">, dock: Dock) => (visibleIn(state, dock) ? state.paired[dock] : null);

export const onScreen = (state: Visible & Pick<Arrangement, "paired" | "homes">, tool: Tool) =>
  visibleIn(state, state.homes[tool]) === tool || pairedIn(state, state.homes[tool]) === tool;

export const panelShows = (tool: Tool) => onScreen(shell.getState(), tool);

export const toolsShown = (state: State = shell.getState()) => DOCKS.some((dock) => visibleIn(state, dock));

const entering: Partial<Record<Tool, () => void>> = {};
export const whenShown = (tool: Tool, enter: () => void) => void (entering[tool] = enter);

shell.subscribe((now, before) => {
  for (const tool of TOOLS) if (onScreen(now, tool) && !onScreen(before, tool)) entering[tool]?.();
});

function nextIn(state: Arrangement, dock: Dock, gone: Tool) {
  const list = mainTabsIn(state, dock);
  const rest = list.filter((tool) => tool !== gone);
  return rest[Math.min(Math.max(0, list.indexOf(gone)), rest.length - 1)] ?? null;
}

function without(state: Arrangement, dock: Dock, gone: Tool) {
  const pair = state.paired[dock] === gone ? null : state.paired[dock];
  const next = state.shown[dock] === gone ? nextIn(state, dock, gone) : state.shown[dock];
  const shown = next ?? pair;
  return { shown, paired: shown === pair ? null : pair, open: state.open[dock] && shown !== null };
}

function placed(state: Arrangement, tool: Tool, dock: Dock, tabs: Tool[], shown: Tool, paired: Tool | null) {
  const from = state.homes[tool];
  const left = from !== dock ? without(state, from, tool) : null;
  set({
    calm: false,
    tabs,
    homes: { ...state.homes, [tool]: dock },
    shown: { ...state.shown, ...(left && { [from]: left.shown }), [dock]: shown },
    paired: { ...state.paired, ...(left && { [from]: left.paired }), [dock]: paired },
    open: { ...state.open, ...(left && { [from]: left.open }), [dock]: true },
  });
}

export function moveTool(tool: Tool, dock: Dock, before?: Tool | null) {
  const state = shell.getState();
  const was = panelShows(tool) && state.paired[dock] !== tool;
  const rest = state.tabs.filter((one) => one !== tool);
  const at = before ? rest.indexOf(before) : -1;
  const tabs = at < 0 ? [...rest, tool] : [...rest.slice(0, at), tool, ...rest.slice(at)];
  placed(state, tool, dock, tabs, tool, state.paired[dock] === tool ? null : state.paired[dock]);
  if (was && state.homes[tool] === dock) entering[tool]?.();
}

export function pairTool(tool: Tool, dock: Dock) {
  const state = shell.getState();
  const partner = state.shown[dock] && state.shown[dock] !== tool ? state.shown[dock] : mainTabsIn(state, dock).find((one) => one !== tool);
  if (!partner) return moveTool(tool, dock);
  const stays = state.tabs.includes(tool) && state.homes[tool] === dock;
  placed(state, tool, dock, stays ? state.tabs : [...state.tabs.filter((one) => one !== tool), tool], partner, tool);
}

export function showTool(tool: Tool) {
  const state = shell.getState();
  const dock = state.homes[tool];
  if (state.paired[dock] === tool) return set({ calm: false, open: { ...state.open, [dock]: true } });
  moveTool(tool, dock, state.tabs.includes(tool) ? nextAfter(tool) : null);
}

function nextAfter(tool: Tool) {
  const { tabs } = shell.getState();
  return tabs[tabs.indexOf(tool) + 1] ?? null;
}

export function closeTab(tool: Tool) {
  const state = shell.getState();
  if (!state.tabs.includes(tool)) return;
  const dock = state.homes[tool];
  const left = without(state, dock, tool);
  set({
    tabs: state.tabs.filter((one) => one !== tool),
    shown: { ...state.shown, [dock]: left.shown },
    paired: { ...state.paired, [dock]: left.paired },
    open: { ...state.open, [dock]: left.open },
  });
}

export function splitDock(dock: Dock) {
  const state = shell.getState();
  const list = mainTabsIn(state, dock);
  const shown = state.shown[dock];
  const partner = list[list.indexOf(shown!) + 1] ?? list.find((tool) => tool !== shown);
  if (partner) pairTool(partner, dock);
}

export const unpair = (dock: Dock) => set(({ paired }) => ({ paired: { ...paired, [dock]: null } }));

export function widenPaired(dock: Dock) {
  const { shown, paired } = shell.getState();
  if (paired[dock]) set({ shown: { ...shown, [dock]: paired[dock] }, paired: { ...paired, [dock]: null } });
}

export const closeDock = (dock: Dock) => set(({ open }) => ({ open: { ...open, [dock]: false } }));

export const closeTools = () => set({ open: docked(() => false) });

export function hideTool(tool: Tool) {
  const state = shell.getState();
  const dock = state.homes[tool];
  if (pairedIn(state, dock) === tool) return unpair(dock);
  if (panelShows(tool)) closeDock(dock);
}

export function toggleDock(dock: Dock) {
  const state = shell.getState();
  if (visibleIn(state, dock)) return closeDock(dock);
  const tool = state.shown[dock] ?? tabsIn(state, dock)[0];
  if (tool) set({ calm: false, shown: { ...state.shown, [dock]: tool }, open: { ...state.open, [dock]: true } });
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
