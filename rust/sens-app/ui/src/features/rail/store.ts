import { createStore } from "zustand/vanilla";
import { commands } from "../../ipc/commands";
import type { SessionSummary, Workspace } from "../../ipc/types";
import { shared } from "../../shared/copy";
import { store, stored } from "../../shared/storage.js";
import { paneOf } from "../panes/store";

const FOLDED = "sens.rail.folded";

export type Activity = "working" | "waiting" | "done";

export const rail = createStore(() => ({
  spaces: null as Workspace[] | null,
  folded: new Set<string>([].concat(stored(FOLDED, []))),
  fault: "",
  activity: new Map<string, Activity>(),
}));

const set = rail.setState;

const URGENCY: Activity[] = ["waiting", "working", "done"];

export const activityOf = (ids: string[], activity: Map<string, Activity>) =>
  URGENCY.find((one) => ids.some((id) => activity.get(id) === one));

export function noteActivity(id: string, now: Activity | null) {
  set((state) => {
    const { activity } = state;
    if ((activity.get(id) ?? null) === now) return state;
    const next = new Map(activity);
    if (now) next.set(id, now);
    else next.delete(id);
    return { activity: next };
  });
}

let owed = "";
const naming = new Set<string>();

let lap = 0;

export async function loadRail() {
  const mine = ++lap;
  let read;
  try {
    read = { spaces: await commands.workspaces(), fault: owed };
  } catch (reason) {
    read = { fault: owed || String(reason) };
  }
  if (mine !== lap) return;
  set(read);
  owed = "";
}

export function titleOf(root: string, session: string) {
  const named = rail.getState().spaces?.find((space) => space.root === root)?.sessions.find((one) => one.id === session)?.title;
  return named || shared.newSession;
}

export const oweRail = (reason: unknown) => void (owed = String(reason));
export const failRail = (reason: unknown) => set({ fault: String(reason) });

export function fold(root: string, shut: boolean) {
  set(({ folded }) => {
    const next = new Set(folded);
    if (shut) next.add(root);
    else next.delete(root);
    store(FOLDED, [...next]);
    return { folded: next };
  });
}

function homeOf(id: string) {
  const shown = paneOf(id);
  if (shown) return shown.desk.getState().root;
  return rail.getState().spaces?.find((space) => space.sessions.some((one) => one.id === id))?.root;
}

export async function nameSession(id: string) {
  const home = homeOf(id);
  if (!home || naming.has(id)) return;
  naming.add(id);
  const title = await commands.titleSession(home, id).catch(() => null);
  naming.delete(id);
  if (title) await loadRail();
}

async function change(work: () => Promise<unknown>) {
  await work().catch(oweRail);
  await loadRail();
}

export const renameSession = (home: string, id: string, title: string) => change(() => commands.renameSession(home, id, title));

export const archiveSession = (home: string, summary: SessionSummary) =>
  change(() => commands.archiveSession(home, summary.id, !summary.archived));

export async function deleteSession(home: string, id: string) {
  try {
    await commands.deleteSession(home, id);
  } catch (reason) {
    oweRail(reason);
    await loadRail();
    return null;
  }
  await loadRail();
  return paneOf(id) ?? null;
}
