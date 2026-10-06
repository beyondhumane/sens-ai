import { createStore } from "zustand/vanilla";
import { panelShows } from "../../app/shell";
import { commands } from "../../ipc/commands";
import type { ProjectMap, Reach } from "../../ipc/types";
import { store, stored } from "../../shared/storage.js";
import { project } from "../project/store";

const MODE = "sens.map.mode";

export type MapMode = "list" | "graph";

export const atlas = createStore(() => ({
  map: null as ProjectMap | null,
  fault: "",
  reach: null as Reach | null,
  unfolded: new Set<string>(),
  mode: (stored(MODE, "list") === "graph" ? "graph" : "list") as MapMode,
  picked: "",
}));

const set = atlas.setState;
const home = () => project.getState().work;

let lap = 0;

export async function loadMap() {
  const mine = ++lap;
  const work = home();
  if (!work) return set({ map: null, fault: "" });
  let map;
  try {
    map = await commands.canonMap(work);
  } catch (reason) {
    if (mine === lap) set({ fault: String(reason) });
    return;
  }
  if (mine !== lap || work !== home()) return;
  set({ map, fault: "" });
  const { reach } = atlas.getState();
  if (reach) await inspect(reach.file);
}

export async function inspect(file: string) {
  const work = home();
  if (!work) return;
  let reach;
  try {
    reach = await commands.canonReach(work, file);
  } catch (reason) {
    return set({ fault: String(reason) });
  }
  if (work === home()) set({ reach });
}

export const leaveReach = () => set({ reach: null });

export function unfoldRegion(name: string, open: boolean) {
  set(({ unfolded }) => {
    if (unfolded.has(name) === open) return {};
    const next = new Set(unfolded);
    if (open) next.add(name);
    else next.delete(name);
    return { unfolded: next };
  });
}

export function showMapAs(mode: MapMode) {
  store(MODE, mode);
  set({ mode });
}

export function pickArea(name: string) {
  const picked = atlas.getState().picked === name ? "" : name;
  set({ picked });
  if (picked) unfoldRegion(picked, true);
}

export function refreshMapAfterTurn() {
  if (panelShows("map")) loadMap();
}

project.subscribe((now, before) => {
  if (now.work === before.work) return;
  lap += 1;
  set({ map: null, fault: "", reach: null, unfolded: new Set(), picked: "" });
  if (panelShows("map")) loadMap();
});
