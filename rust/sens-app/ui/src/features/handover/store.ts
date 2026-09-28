import { draft, resume, toChat } from "../../app/session";
import { events } from "../../ipc/commands";
import type { HandOver } from "../../ipc/types";
import { addToMessage } from "../composer/store";

async function arrive({ root, session }: HandOver) {
  if (session) await resume(root, session);
  else if (root) await draft(root);
  else toChat();
}

async function takeOver(hand: HandOver) {
  await arrive(hand);
  if (hand.text) addToMessage(hand.text);
  else document.getElementById("task")?.focus();
}

export const hearHandOver = () => events.barHandOver((hand) => void takeOver(hand));
