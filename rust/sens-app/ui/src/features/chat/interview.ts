import { commands } from "../../ipc/commands";

async function holds(root: string, folder: string, name: string): Promise<boolean> {
  try {
    const entries = await commands.folder(root, folder);
    return entries.some((entry) => entry.name.toLowerCase() === name.toLowerCase());
  } catch {
    return false;
  }
}

export async function interviewDue(root: string): Promise<boolean> {
  const [unity, designed] = await Promise.all([holds(root, "ProjectSettings", "ProjectVersion.txt"), holds(root, "", "Design")]);
  return unity && !designed;
}
