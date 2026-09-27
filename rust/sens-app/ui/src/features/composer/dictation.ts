import { useEffect, useRef, useState, type RefObject } from "react";
import { commands, events } from "../../ipc/commands";
import type { DictationCause, DictationHeard, DictationRefusal } from "../../ipc/types";
import { localeNow } from "../../shared/i18n";
import { warn } from "../chat/store";
import { focused, type Pane } from "../panes/store";
import { profile } from "../profile/store";
import { t } from "./copy";

type Blocked = Exclude<DictationCause, "other"> | null;

const refusalOf = (error: unknown): DictationRefusal =>
  typeof error === "object" && error !== null && "cause" in error ? (error as DictationRefusal) : { cause: "other", message: String(error) };

const joined = (before: string, phrases: string[], guess: string) => [before, ...phrases, guess.trim()].filter(Boolean).join(" ");

export async function setWake(on: boolean) {
  await commands.setWake(on);
  profile.setState(({ person }) => ({ person: { ...person, wake: on } }));
}

export function armWake() {
  if (profile.getState().person.wake) commands.setWake(true).catch((reason) => warn(String(reason)));
}

export function useDictation(pane: Pane, text: string, setText: (text: string) => void, field: RefObject<HTMLTextAreaElement | null>) {
  const [listening, setListening] = useState(false);
  const [blocked, setBlocked] = useState<Blocked>(null);
  const live = useRef<{ stop: (keep: boolean) => void } | null>(null);
  const starter = useRef<(handsFree: boolean) => Promise<void>>(async () => {});

  useEffect(() => () => live.current?.stop(false), []);

  useEffect(() => {
    const heard = events.dictation((what) => {
      if (focused() !== pane) return;
      if (what.kind === "slept") refuse(what.refusal);
      if (what.kind === "woke" && !live.current) void starter.current(true);
    });
    return () => void heard.then((unlisten) => unlisten());
  }, [pane]);

  function refuse({ cause, message }: DictationRefusal) {
    if (cause !== "other") setBlocked(cause);
    warn(cause === "speech" ? t.speechOff : cause === "microphone" ? t.microphoneOff : cause === "unsupported" ? t.noDictation : message, pane);
  }

  async function start(handsFree: boolean) {
    const before = text.trim();
    const phrases: string[] = [];
    const early: DictationHeard[] = [];
    let id: number | null = null;
    let muted = false;
    let stopping = false;
    let unlisten = () => {};

    const finish = (refusal: DictationRefusal | null) => {
      unlisten();
      live.current = null;
      setListening(false);
      if (refusal) refuse(refusal);
      if (!muted) field.current?.focus();
    };
    const hear = (heard: DictationHeard) => {
      if (heard.kind === "woke" || heard.kind === "slept") return;
      if (id === null) return void early.push(heard);
      if (heard.id !== id) return;
      if (heard.kind === "ended") return finish(heard.refusal);
      if (muted) return;
      if (heard.kind === "phrase") phrases.push(heard.text.trim());
      setText(joined(before, phrases, heard.kind === "guess" ? heard.text : ""));
    };

    setListening(true);
    live.current = {
      stop: (keep) => {
        muted ||= !keep;
        stopping = true;
        if (id !== null) void commands.dictationStop();
      },
    };
    unlisten = await events.dictation(hear);
    try {
      id = await commands.dictationStart(localeNow(), handsFree);
    } catch (error) {
      return finish(refusalOf(error));
    }
    if (stopping) void commands.dictationStop();
    for (const heard of early.splice(0)) hear(heard);
  }
  starter.current = start;

  function toggle() {
    if (blocked === "speech" || blocked === "microphone") {
      setBlocked(null);
      commands.dictationSettings(blocked).catch((reason) => warn(String(reason), pane));
      return;
    }
    if (live.current) return live.current.stop(true);
    void start(false);
  }

  const label = listening
    ? t.stopDictating
    : blocked === "speech"
      ? t.openSpeech
      : blocked === "microphone"
        ? t.openMicrophone
        : blocked === "unsupported"
          ? t.noDictation
          : t.dictate;

  return { able: blocked !== "unsupported", listening, label, toggle, hush: () => live.current?.stop(false) };
}
