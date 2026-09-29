import { createStore } from "zustand/vanilla";
import { commands, events } from "../../ipc/commands";
import type { Microphone } from "../../ipc/types";
import { profile } from "../profile/store";

export const voice = createStore(() => ({
  ready: false,
  fetching: false,
  done: 0,
  total: 0,
  fault: "",
  microphones: [] as Microphone[],
  chosen: null as string | null,
}));

export const percentOf = (done: number, total: number) => (total ? Math.min(100, Math.floor((done * 100) / total)) : 0);

export async function watchVoice() {
  await events.voice((heard) => {
    if (heard.kind === "fetching") voice.setState({ fetching: true, done: heard.done, total: heard.total, fault: "" });
    if (heard.kind === "ready") voice.setState({ ready: true, fetching: false, fault: "" });
    if (heard.kind === "unfetched") voice.setState({ fetching: false, fault: heard.message });
  });
  const model = await commands.voiceModel();
  voice.setState({ ready: model.ready, fetching: model.fetching, total: model.bytes });
}

export function prepareVoice() {
  voice.setState({ fault: "" });
  return commands.voicePrepare();
}

export async function loadMicrophones() {
  const [microphones, chosen] = await Promise.all([commands.voiceMicrophones(), commands.voiceMicrophone()]);
  voice.setState({ microphones, chosen });
}

export async function chooseMicrophone(chosen: string | null) {
  await commands.voiceChoose(chosen);
  voice.setState({ chosen });
}

export async function setWake(on: boolean) {
  const refused = await commands.setWake(on);
  profile.setState(({ person }) => ({ person: { ...person, wake: on } }));
  if (refused) throw refused;
}
