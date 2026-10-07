import { createStore } from "zustand/vanilla";
import { store, stored } from "../shared/storage.js";
import { arrange, arrangementOf, calmDown, DOCKS, HOMES, NO_PAIRS, pairedIn, readArrangement, shell, tabsIn, visibleIn, type Arrangement, type Dock, type Tool } from "./shell";

const SAVED = "sens.arrangements";

export const MOST_KEPT = 9;

export type Start = "conversation" | "code" | "review" | "preview" | "explore";

export const STARTS: Start[] = ["conversation", "code", "review", "preview", "explore"];

type Layout = Pick<Arrangement, "wide" | "calm" | "homes" | "tabs" | "shown" | "paired" | "open">;

function laid(docks: Partial<Record<Dock, Tool[]>>, wide = false): Layout {
  const homes = { ...HOMES };
  for (const dock of DOCKS) for (const tool of docks[dock] ?? []) homes[tool] = dock;
  const first = (dock: Dock) => docks[dock]?.[0] ?? null;
  return {
    wide,
    calm: false,
    homes,
    tabs: DOCKS.flatMap((dock) => docks[dock] ?? []),
    shown: { start: first("start"), end: first("end"), bottom: first("bottom") },
    paired: NO_PAIRS,
    open: { start: Boolean(first("start")), end: Boolean(first("end")), bottom: Boolean(first("bottom")) },
  };
}

const LAYOUTS: Record<Exclude<Start, "conversation">, Layout> = {
  code: laid({ end: ["files", "changes", "map"], bottom: ["terminal", "tasks"] }),
  review: laid({ start: ["changes"], end: ["files"], bottom: ["tasks"] }),
  preview: laid({ end: ["web", "files"], bottom: ["terminal"] }, true),
  explore: laid({ start: ["map"], end: ["files", "changes"] }),
};

export function startFrom(start: Start, from: Arrangement = arrangementOf(shell.getState())): Arrangement {
  return start === "conversation" ? { ...from, calm: true } : { ...from, ...LAYOUTS[start] };
}

export function signature(arrangement: Arrangement) {
  if (arrangement.calm) return "calm";
  return JSON.stringify([arrangement.rail, arrangement.wide, DOCKS.map((dock) => [tabsIn(arrangement, dock), visibleIn(arrangement, dock), pairedIn(arrangement, dock)])]);
}

export interface Kept {
  name: string;
  arrangement: Arrangement;
}

function readKept(): Kept[] {
  const list = stored(SAVED, []);
  if (!Array.isArray(list)) return [];
  return list
    .filter((one): one is Kept => Boolean(one) && typeof one.name === "string")
    .slice(0, MOST_KEPT)
    .map((one) => ({ name: one.name, arrangement: readArrangement(one.arrangement) }));
}

export const arrangements = createStore(() => ({
  kept: readKept(),
  before: null as Arrangement | null,
}));

function keepList(kept: Kept[]) {
  store(SAVED, kept);
  arrangements.setState({ kept });
}

export function apply(arrangement: Arrangement) {
  const now = arrangementOf(shell.getState());
  if (signature(now) === signature(arrangement) && JSON.stringify(now.sizes) === JSON.stringify(arrangement.sizes)) return;
  arrangements.setState({ before: now });
  arrange(arrangement);
}

export const applyStart = (start: Start) => apply(startFrom(start));

export function applyKept(at: number) {
  const one = arrangements.getState().kept[at];
  if (one) apply(one.arrangement);
}

export function undo() {
  const { before } = arrangements.getState();
  if (!before) return;
  arrangements.setState({ before: null });
  arrange(before);
}

export const forgetUndo = () => arrangements.setState({ before: null });

export function keepNow(name: string) {
  const { kept } = arrangements.getState();
  if (kept.length >= MOST_KEPT) return;
  keepList([...kept, { name: name.trim(), arrangement: arrangementOf(shell.getState()) }]);
}

export function forget(at: number) {
  keepList(arrangements.getState().kept.filter((_, index) => index !== at));
}

export const toggleCalm = () => calmDown(!shell.getState().calm);
