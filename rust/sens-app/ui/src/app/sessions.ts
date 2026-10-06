import { send, switched, effortLevels } from "../features/composer/store";
import { choose, chosenCard, models, offeredBy, sameModel } from "../features/models/store";
import { paneOf, split, workOf, type Pane } from "../features/panes/store";
import { project } from "../features/project/store";
import { loadRail } from "../features/rail/store";
import { commands } from "../ipc/commands";
import { answers, onScreen } from "./acts";
import { closePane, draftBeside, openBeside } from "./session";

interface From {
  session: string;
}

interface Target {
  target: string;
}

function shown(target: string) {
  const pane = paneOf(target);
  if (!pane) throw new Error("That session is not on screen; open it with open_session first.");
  return pane;
}

async function opened(from: string, root: string, target: string) {
  const pane = paneOf(target);
  if (pane) return pane;
  onScreen(from);
  await openBeside(root, target);
  return shown(target);
}

async function message(pane: Pane, text: string) {
  await send(text, pane);
  const id = pane.desk.getState().session;
  if (!id || !pane.chat.getState().busy) throw new Error("Sens could not send it: the session has no model to answer with, or is busy.");
  return id;
}

async function openSession({ session, target, root }: From & Target & { root: string }) {
  await opened(session, root, target);
  return "The session is open beside this one.";
}

async function newSession({ session, root, prompt }: From & { root: string; prompt: string }) {
  onScreen(session);
  const pane = await draftBeside(root);
  if (!prompt) return "A new, empty session is open beside this one for the person.";
  const id = await message(pane, prompt);
  return `Session ${id} started beside this one and is working on it.`;
}

async function sendToSession({ session, target, root, text }: From & Target & { root: string; text: string }) {
  const pane = await opened(session, root, target);
  await message(pane, text);
  return "Sent; that session is working on it.";
}

async function stopSession({ target }: Target) {
  await commands.chatStop(target);
  return "That session was told to stop.";
}

async function refreshSessions() {
  await loadRail();
  return "";
}

async function closeSessionPane({ target }: Target) {
  const pane = shown(target);
  if (!split()) throw new Error("Only one chat is on screen; there is nothing to close.");
  await closePane(pane);
  return "That half of the window is closed.";
}

function cardFor(model: string) {
  const wanted = model.trim().toLowerCase();
  for (const provider of models.getState().catalog) {
    const card = offeredBy(provider).find((one) => sameModel(one.id, model) || one.label.toLowerCase() === wanted);
    if (card) return { provider, card };
  }
  throw new Error(`The model picker offers no ${model}.`);
}

function setSessionModel({ target, model, effort, thinking }: Target & { model?: string | null; effort?: string | null; thinking?: boolean | null }) {
  const pane = shown(target);
  if (model) {
    const { provider, card } = cardFor(model);
    choose(provider.id, card.id, pane);
  }
  const card = chosenCard(pane);
  if (effort) {
    if (!effortLevels(card).includes(effort)) throw new Error(`${card?.label ?? "This model"} has no ${effort} effort; it offers ${effortLevels(card).join(", ") || "none"}.`);
    pane.desk.setState({ effort });
  }
  if (typeof thinking === "boolean") pane.desk.setState({ thinking });
  const { effort: now, thinking: thinks } = pane.desk.getState();
  return `Next messages use ${card?.label ?? "the chosen model"}${effort ? `, ${now} effort` : ""}${typeof thinking === "boolean" ? `, thinking ${thinks ? "on" : "off"}` : ""}.`;
}

async function branchSwitched({ session }: From & { branch: string }) {
  const pane = paneOf(session);
  if (!pane) return "";
  const repo = await commands.repo(workOf(pane));
  if (repo) await switched(workOf(pane), repo, pane);
  return "";
}

export function answerSessions() {
  answers("open_session", openSession);
  answers("new_session", newSession);
  answers("send_to_session", sendToSession);
  answers("stop_session", stopSession);
  answers("refresh_sessions", refreshSessions);
  answers("close_session_pane", closeSessionPane);
  answers("set_session_model", setSessionModel);
  answers("branch_switched", branchSwitched);
}
