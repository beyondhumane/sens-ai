import { useEffect, useRef, useState, type RefObject } from "react";
import { useStore } from "zustand";
import { commands, events } from "../../ipc/commands";
import type { VoiceHeard, VoiceRefusal } from "../../ipc/types";
import { languageNow } from "../../shared/i18n";
import { warn } from "../chat/store";
import type { Pane } from "../panes/store";
import { percentOf, prepareVoice, voice } from "../voice/store";
import { t } from "./copy";

type Phase = "idle" | "listening" | "finishing";

const refusalOf = (error: unknown): VoiceRefusal =>
  typeof error === "object" && error !== null && "cause" in error ? (error as VoiceRefusal) : { cause: "other", message: String(error) };

const joined = (before: string, phrases: string[], guess = "") => [before, ...phrases, guess.trim()].filter(Boolean).join(" ");

export function useDictation(pane: Pane, text: string, setText: (text: string) => void, field: RefObject<HTMLTextAreaElement | null>) {
  const [phase, setPhase] = useState<Phase>("idle");
  const [level, setLevel] = useState(0);
  const model = useStore(voice);
  const live = useRef<{ stop: (keep: boolean) => void } | null>(null);

  useEffect(() => () => live.current?.stop(false), []);

  function refuse({ cause, message }: VoiceRefusal) {
    warn(cause === "microphone" ? t.microphoneRefused(message) : message, pane);
    if (cause === "model") void prepareVoice();
  }

  async function start() {
    const before = text.trim();
    const phrases: string[] = [];
    const early: VoiceHeard[] = [];
    let id: number | null = null;
    let muted = false;
    let stopping = false;
    let unlisten = () => {};

    const finish = (refusal: VoiceRefusal | null) => {
      unlisten();
      live.current = null;
      setPhase("idle");
      setLevel(0);
      if (refusal) refuse(refusal);
      if (!muted) field.current?.focus();
    };
    const hear = (heard: VoiceHeard) => {
      if (!("id" in heard)) return;
      if (id === null) return void early.push(heard);
      if (heard.id !== id) return;
      if (heard.kind === "ended") return finish(heard.refusal);
      if (heard.kind === "level") return setLevel(heard.level);
      if (muted) return;
      if (heard.kind === "guess") return setText(joined(before, phrases, heard.text));
      phrases.push(heard.text.trim());
      setText(joined(before, phrases));
    };

    setPhase("listening");
    live.current = {
      stop: (keep) => {
        muted ||= !keep;
        stopping = true;
        setPhase("finishing");
        setLevel(0);
        if (id !== null) void commands.voiceStop();
      },
    };
    unlisten = await events.voice(hear);
    try {
      id = await commands.voiceStart(languageNow());
    } catch (error) {
      return finish(refusalOf(error));
    }
    if (stopping) void commands.voiceStop();
    for (const heard of early.splice(0)) hear(heard);
  }

  function toggle() {
    if (phase === "finishing") return;
    if (live.current) return live.current.stop(true);
    if (!model.ready) {
      if (!model.fetching) void prepareVoice();
      warn(model.fetching ? t.voiceFetching(percentOf(model.done, model.total)) : t.voiceFetchingStarts, pane);
      return;
    }
    void start();
  }

  const label =
    phase === "listening"
      ? t.stopDictating
      : phase === "finishing"
        ? t.transcribing
        : !model.ready && model.fetching
          ? t.voiceFetching(percentOf(model.done, model.total))
          : t.dictate;

  return { phase, level, label, toggle, hush: () => live.current?.stop(false) };
}
