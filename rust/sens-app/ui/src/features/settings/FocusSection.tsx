import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { useStore } from "zustand";
import { commands, events } from "../../ipc/commands";
import type { Shortcut, ShortcutKeys } from "../../ipc/types";
import { chooseMicrophone, loadMicrophones, percentOf, prepareVoice, voice } from "../voice/store";
import { t } from "./copy";
import { DEFAULT_SHORTCUT, onlyModifiers, pressedKeys, sameKeys } from "./shortcut";

export function FocusSection() {
  return (
    <>
      <ShortcutBlock />
      <VoiceBlock />
    </>
  );
}

function ShortcutBlock() {
  const [shortcut, setShortcut] = useState<Shortcut | null>(null);
  const [capturing, setCapturing] = useState(false);
  const [fault, setFault] = useState("");
  const paused = useRef(false);

  useEffect(() => {
    commands.shortcutState().then(setShortcut, () => {});
    return () => {
      if (paused.current) void commands.shortcutPause(false);
    };
  }, []);

  function pause(on: boolean) {
    paused.current = on;
    setCapturing(on);
    return commands.shortcutPause(on);
  }

  async function capture() {
    setFault("");
    if (capturing) return void pause(false);
    await pause(true);
  }

  async function choose(keys: ShortcutKeys) {
    paused.current = false;
    setCapturing(false);
    setFault("");
    try {
      setShortcut(await commands.shortcutSet(keys));
    } catch (reason) {
      setFault(String(reason));
    }
  }

  function heard(event: KeyboardEvent) {
    if (!capturing) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.key === "Escape") return void pause(false);
    if (onlyModifiers(event.nativeEvent)) return;
    const keys = pressedKeys(event.nativeEvent);
    if (!keys) return setFault(t.shortcutUnusable);
    void choose(keys);
  }

  if (!shortcut) return null;
  return (
    <div className="pair">
      <span className="label">{t.focusMode}</span>
      <p className="note">{t.focusNote}</p>
      <div className="settings-switch">
        <span>{t.shortcut}</span>
        <span className="settings-keys keycaps" aria-live="polite">
          {shortcut.named.split("+").map((key) => (
            <kbd key={key}>{key}</kbd>
          ))}
        </span>
      </div>
      <div className="settings-row">
        <button
          className="quiet"
          id="settings-shortcut"
          aria-pressed={capturing}
          onClick={() => void capture()}
          onKeyDown={heard}
          onBlur={() => capturing && void pause(false)}
        >
          {capturing ? t.shortcutPress : t.shortcutChange}
        </button>
        <button className="quiet" hidden={sameKeys(shortcut.keys, DEFAULT_SHORTCUT)} onClick={() => void choose(DEFAULT_SHORTCUT)}>
          {t.shortcutReset}
        </button>
      </div>
      <p className="note fault" role="status" hidden={!shortcut.taken}>
        {t.shortcutTaken}
      </p>
      <p className="note fault" role="alert" hidden={!fault}>
        {fault}
      </p>
    </div>
  );
}

const MEBIBYTE = 1024 * 1024;

function VoiceBlock() {
  const known = useStore(voice);
  const [testing, setTesting] = useState<number | null>(null);
  const [level, setLevel] = useState(0);
  const [fault, setFault] = useState("");
  const listening = useRef<number | null>(null);

  useEffect(() => {
    loadMicrophones().catch((reason) => setFault(String(reason)));
    return () => {
      if (listening.current !== null) void commands.voiceStop();
    };
  }, []);

  useEffect(() => {
    listening.current = testing;
    if (testing === null) return;
    const heard = events.voice((what) => {
      if (!("id" in what) || what.id !== testing) return;
      if (what.kind === "level") setLevel(what.level);
      if (what.kind === "ended") {
        setTesting(null);
        setLevel(0);
        if (what.refusal) setFault(what.refusal.message);
      }
    });
    return () => void heard.then((unlisten) => unlisten());
  }, [testing]);

  async function test() {
    setFault("");
    if (testing !== null) return void commands.voiceStop();
    try {
      setTesting(await commands.voiceTest());
    } catch (reason) {
      setFault(typeof reason === "object" && reason !== null && "message" in reason ? String(reason.message) : String(reason));
    }
  }

  async function pick(chosen: string) {
    setFault("");
    try {
      await chooseMicrophone(chosen || null);
    } catch (reason) {
      setFault(String(reason));
    }
  }

  const fallback = known.microphones.find((one) => one.default)?.name ?? "";
  const missing = known.chosen !== null && !known.microphones.some((one) => one.name === known.chosen);
  const model = known.ready
    ? t.modelReady(Math.round(known.total / MEBIBYTE))
    : known.fetching
      ? t.modelFetching(percentOf(known.done, known.total))
      : known.fault || t.modelMissing;

  return (
    <div className="pair">
      <span className="label">{t.voice}</span>
      <div className="settings-row">
        <select className="field" id="settings-microphone" aria-label={t.microphone} value={known.chosen ?? ""} onChange={(event) => void pick(event.target.value)}>
          <option value="">{t.windowsDefault(fallback)}</option>
          {known.microphones.map((one) => (
            <option key={one.name} value={one.name}>
              {one.name}
            </option>
          ))}
          {missing && <option value={known.chosen ?? ""}>{known.chosen}</option>}
        </select>
        <button className="quiet" id="settings-microphone-test" aria-pressed={testing !== null} onClick={() => void test()}>
          {testing !== null ? t.stopTest : t.testMicrophone}
        </button>
      </div>
      <div className="voice-level" role="meter" aria-label={t.level} aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(level * 100)} hidden={testing === null}>
        <span style={{ width: `${Math.round(level * 100)}%` }} />
      </div>
      <p className="note">{t.voiceNote}</p>
      <div className="settings-row">
        <p className={known.fault && !known.ready ? "note fault" : "note"} role="status">
          {model}
        </p>
        <button className="quiet" hidden={known.ready || known.fetching} onClick={() => void prepareVoice()}>
          {t.modelFetch}
        </button>
      </div>
      <p className="note fault" role="alert" hidden={!fault}>
        {fault}
      </p>
    </div>
  );
}
