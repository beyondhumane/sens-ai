import { holds, isUnity } from "../project/folders";

export async function interviewDue(root: string): Promise<boolean> {
  const [unity, designed] = await Promise.all([isUnity(root), holds(root, "", "Design")]);
  return unity && !designed;
}
