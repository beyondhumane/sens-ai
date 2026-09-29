import { createStore } from "zustand/vanilla";
import { commands, events } from "../ipc/commands";
import type { BarOpened, BarProject, Copied, Front, HandOver, Workspace } from "../ipc/types";
import { blank, hearIn, idle, keepQuiet, load } from "../features/chat/store";
import type { Foot, Reply, Turn, You } from "../features/chat/turns";
import { tally, type Work } from "../features/chat/work";
import { canSend, forgetClips, hearDrops, pasteText, rereadSettings, send } from "../features/composer/store";
import { adopt, newPane } from "../features/panes/store";
import { watchVoice } from "../features/voice/store";
import { languageOf, showLanguage } from "../shared/i18n";
import { showLook } from "../shared/look";
import { store, stored } from "../shared/storage.js";
import type { Sessions } from "./choices";

export const LAST = "sens.bar.last";
const FIRST_PROVIDER = "claude";

export type Offer = "offered" | "taking" | "taken" | "gone";

export type Phase = "idle" | "typing" | "working" | "waiting" | "answer" | "done" | "error";

export interface Last {
  root: string;
  session: string;
}

export const own = newPane();

export const bar = createStore(() => ({
  projects: [] as BarProject[],
  sessions: {} as Sessions,
  filter: "",
  front: null as Front | null,
  clip: null as Copied | null,
  shot: "offered" as Offer,
  copied: "offered" as Offer,
  pinned: false,
  choosing: false,
  opened: 0,
}));

const set = bar.setState;

export const lastReply = (turns: Turn[]) => turns.findLast((turn): turn is Reply => turn.kind === "reply");

export const firstQuestion = (turns: Turn[]) => turns.find((turn): turn is You => turn.kind === "you" && turn.text.trim() !== "")?.text ?? "";

export const lastQuestion = (turns: Turn[]) => turns.findLast((turn): turn is You => turn.kind === "you" && turn.text.trim() !== "")?.text ?? "";

const asking = (reply: Reply | undefined) => Boolean(reply?.parts.some((part) => part.kind === "ask" && part.active));

export function phaseOf(turns: Turn[], busy: boolean, text: string): Phase {
  const reply = lastReply(turns);
  if (busy) {
    if (asking(reply)) return "waiting";
    const last = reply?.parts.at(-1);
    return last?.kind === "said" && last.text.trim() ? "answer" : "working";
  }
  if (text.trim()) return "typing";
  if (!reply?.closed) return "idle";
  return reply.parts.some((part) => part.kind === "fault") ? "error" : "done";
}

export function spentOf(reply: Reply | undefined) {
  if (!reply) return { reads: 0, millis: 0 };
  const work = reply.parts.filter((part): part is Work => part.kind === "step" || part.kind === "thought");
  const foot = reply.parts.findLast((part): part is Foot => part.kind === "foot");
  return { reads: tally(work).reads, millis: foot?.millis ?? 0 };
}

const conversing = () => own.chat.getState().turns.length > 0;

const held = () => own.chat.getState().busy || asking(lastReply(own.chat.getState().turns));

export function fresh(root = own.desk.getState().root, text = "") {
  blank("", own);
  idle(true, own);
  own.desk.setState({ root, text, isolate: false, worktree: null, slashes: [] });
  forgetClips(own);
  rereadSettings(own);
  const { choice } = own.desk.getState();
  if (!choice.provider) own.desk.setState({ choice: { ...choice, provider: FIRST_PROVIDER } });
  set({ choosing: false, shot: "offered", copied: "offered" });
}

const watch = () => void commands.barWatching(document.hidden ? null : own.desk.getState().session || null).catch(() => {});

async function arrive(resume: HandOver | null, projects: BarProject[]) {
  if (!resume?.root || !resume.session) return fresh(resume?.root || projects[0]?.root || own.desk.getState().root);
  if (own.desk.getState().session === resume.session) return;
  fresh(resume.root);
  await load(resume.session, own);
}

export async function opened({ look, language, front, pinned, resume }: BarOpened) {
  showLanguage(languageOf(language));
  showLook(look);
  set(({ opened }) => ({ front, pinned, clip: null, opened: opened + 1 }));
  const [projects, spaces] = await Promise.all([commands.barProjects().catch((): BarProject[] => []), commands.workspaces().catch((): Workspace[] => [])]);
  set({ projects, sessions: Object.fromEntries(spaces.map((space) => [space.root, space.sessions])) });
  if (!held()) await arrive(resume, projects);
  watch();
  const context = await commands.barContext().catch(() => null);
  set({ clip: context?.clip ?? null });
}

export async function ask() {
  const { text, root, session } = own.desk.getState();
  if (own.chat.getState().busy || !canSend(text, own)) return;
  own.desk.setState({ text: "" });
  if (!session) commands.remember(root).catch(() => {});
  await send(text, own);
  const sent = own.desk.getState().session;
  if (sent) store(LAST, { root, session: sent } satisfies Last);
}

export async function resumeLast() {
  const last = stored(LAST, null) as Last | null;
  if (!last?.root || !last.session || conversing() || own.chat.getState().busy) return false;
  own.desk.setState({ root: last.root });
  await load(last.session, own);
  return true;
}

export async function handOver() {
  const { root, session, text } = own.desk.getState();
  await commands.barHandOver({ root, session, text });
  fresh(root);
}

export function hide() {
  set({ choosing: false });
  void commands.barHide().catch(() => {});
  void commands.barWatching(null).catch(() => {});
}

export function pin() {
  const pinned = !bar.getState().pinned;
  set({ pinned });
  void commands.barPin(pinned).catch(() => {});
}

export function choose(root: string) {
  set({ choosing: false });
  if (root === own.desk.getState().root && !conversing()) return;
  fresh(root, own.desk.getState().text);
}

export async function resumeSession(root: string, id: string) {
  set({ choosing: false });
  if (own.desk.getState().session === id) return;
  fresh(root, own.desk.getState().text);
  await load(id, own);
}

export const filterChoices = (filter: string) => set({ filter });

export function cycle(step: number) {
  const { projects } = bar.getState();
  if (!projects.length) return;
  const at = projects.findIndex((one) => one.root === own.desk.getState().root);
  choose(projects[(at + step + projects.length) % projects.length].root);
}

export const toggleChoosing = () => set(({ choosing }) => ({ choosing: !choosing, filter: "" }));

export async function takeClip() {
  if (bar.getState().copied !== "offered") return;
  set({ copied: "taking" });
  const text = await commands.barClip().catch(() => null);
  if (!text) return set({ clip: null, copied: "gone" });
  await pasteText(text, own);
  set({ copied: own.desk.getState().attached.some((file) => file.kind === "text") ? "taken" : "offered" });
}

export async function takeShot() {
  const { front, shot } = bar.getState();
  if (!front || shot !== "offered") return;
  set({ shot: "taking" });
  const taken = await commands.barShot().catch(() => null);
  if (!taken) return set({ shot: "gone" });
  const picture = {
    name: front.app,
    bytes: Math.floor((taken.data.length * 3) / 4),
    mediaType: taken.mediaType,
    url: `data:${taken.mediaType};base64,${taken.data}`,
    width: taken.width,
    height: taken.height,
  };
  own.desk.setState(({ pasted }) => ({ pasted: [...pasted, picture] }));
  set({ shot: "taken" });
}

export function dropShot(url: string) {
  own.desk.setState(({ pasted }) => ({ pasted: pasted.filter((one) => one.url !== url) }));
  set({ shot: "offered" });
}

export function dropText(path: string) {
  own.desk.setState(({ attached }) => ({ attached: attached.filter((one) => one.path !== path) }));
  set(({ clip }) => ({ copied: clip ? "offered" : "gone" }));
}

export function boot() {
  adopt(own);
  keepQuiet();
  fresh();
  events.chat((from, event) => {
    if (event.kind === "limits" || event.kind === "lockedOut") return;
    if (from && from === own.desk.getState().session) hearIn(own, from, event);
  });
  events.barOpen((given) => void opened(given));
  own.desk.subscribe((now, was) => now.session !== was.session && watch());
  document.addEventListener("visibilitychange", watch);
  hearDrops();
  watchVoice().catch(() => {});
}
