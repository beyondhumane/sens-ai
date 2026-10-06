import { project } from "../features/project/store";
import { commands, events } from "../ipc/commands";
import type { Acting } from "../ipc/types";

type Act = (input: never) => string | Promise<string>;

const acts = new Map<string, Act>();

const ELSEWHERE = "The person is looking at another session, so Sens left their screen as it was.";

export function onScreen(session: string) {
  if (project.getState().session !== session) throw new Error(ELSEWHERE);
}

export const answers = <Input>(act: string, run: (input: Input) => string | Promise<string>) => void acts.set(act, run as Act);

const reasonOf = (reason: unknown) => (reason instanceof Error ? reason.message : String(reason));

export async function perform({ ask, act, input }: Acting) {
  const run = acts.get(act);
  try {
    if (!run) throw new Error(`Sens has no "${act}" in this window.`);
    await commands.actAnswer(ask, true, await run(input as never));
  } catch (reason) {
    await commands.actAnswer(ask, false, reasonOf(reason)).catch(() => {});
  }
}

export const hearActs = () => events.act(perform);
