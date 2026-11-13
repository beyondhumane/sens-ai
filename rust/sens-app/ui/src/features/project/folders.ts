import { commands } from "../../ipc/commands";

export async function holds(root: string, folder: string, name: string): Promise<boolean> {
  try {
    const entries = await commands.folder(root, folder);
    return entries.some((entry) => entry.name.toLowerCase() === name.toLowerCase());
  } catch {
    return false;
  }
}

export function isUnity(root: string): Promise<boolean> {
  return holds(root, "ProjectSettings", "ProjectVersion.txt");
}
