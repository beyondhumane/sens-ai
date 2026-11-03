import { createStore } from "zustand/vanilla";
import { commands } from "../../ipc/commands";
import type { Entry } from "../../ipc/types";
import { project } from "../project/store";

export const files = createStore(() => ({
  unfolded: new Set<string>(),
  folders: new Map<string, Entry[]>(),
  fault: "",
  loads: 0,
}));

const set = files.setState;
const home = () => project.getState().work;

let generation = 0;
const pending = new Set<string>();

export function loadFolder(path: string) {
  if (files.getState().folders.has(path) || pending.has(path)) return;
  const mine = generation;
  pending.add(path);
  commands
    .folder(home(), path)
    .then(
      (entries) => mine === generation && set(({ folders }) => ({ folders: new Map(folders).set(path, entries), fault: "" })),
      (reason) => mine === generation && set({ fault: String(reason) }),
    )
    .finally(() => mine === generation && pending.delete(path));
}

async function readShown(root: string, path: string, into: Map<string, Entry[]>) {
  const entries = await commands.folder(root, path);
  into.set(path, entries);
  const { unfolded } = files.getState();
  await Promise.all(entries.filter((entry) => entry.dir && unfolded.has(entry.path)).map((entry) => readShown(root, entry.path, into)));
}

export async function loadFiles() {
  const mine = ++generation;
  pending.clear();
  const root = home();
  if (!root) return set({ folders: new Map(), fault: "" });
  set(({ loads }) => ({ loads: loads + 1 }));
  const folders = new Map<string, Entry[]>();
  try {
    await readShown(root, "", folders);
  } catch (reason) {
    if (mine === generation) set({ fault: String(reason) });
    return;
  }
  if (mine === generation) set({ folders, fault: "" });
}

export function forgetTree() {
  generation++;
  pending.clear();
  set({ unfolded: new Set(), folders: new Map(), fault: "" });
}

export function toggleFolder(path: string) {
  set(({ unfolded }) => {
    const next = new Set(unfolded);
    if (!next.delete(path)) next.add(path);
    return { unfolded: next };
  });
}

export function revealFile(path: string) {
  set(({ unfolded }) => {
    const parts = path.split("/");
    const next = new Set(unfolded);
    for (let at = 1; at < parts.length; at++) next.add(parts.slice(0, at).join("/"));
    return next.size === unfolded.size ? {} : { unfolded: next };
  });
}
