import { useEffect, useRef, useState, type FormEvent, type KeyboardEvent } from "react";
import { useStore } from "zustand";
import { showView } from "../../app/session";
import { commands, events } from "../../ipc/commands";
import { looks } from "../../shared/copy";
import { Icon } from "../../shared/Icon";
import { ICONS } from "../../shared/icons.js";
import { LanguagePicker } from "../../shared/LanguagePicker";
import { accentName, look, modeName, type Look } from "../../shared/look";
import { AccentPicker, ModePicker } from "../../shared/LookPicker";
import { Mark } from "../../shared/Mark";
import { chooseLook } from "../look/store";
import { useLanguageChoice } from "../look/useLanguageChoice";
import { setNotices } from "../notify/store";
import { profile, saveProfileName } from "../profile/store";
import { checkUpdates, setAutomatic, updateState, updates } from "../updates/store";
import { openUpdate } from "../updates/UpdatePanel";
import { chooseMicrophone, loadMicrophones, percentOf, prepareVoice, voice } from "../voice/store";
import { openWelcome, type Step } from "../welcome/store";
import { t } from "./copy";
import { ProvidersSection } from "./ProvidersSection";
import { settingsSheet } from "./sheet";
import { SECTIONS, closeSettings, enterSettings, settings, settingsClosed, showSection, type Section } from "./store";

export function SettingsDialog() {
  const open = useStore(settingsSheet, (s) => s.open);
  const box = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const shown = box.current;
    if (!shown) return;
    if (open && !shown.open) shown.showModal();
    if (!open && shown.open) shown.close();
  }, [open]);

  return (
    <dialog
      className="settings"
      id="settings"
      aria-labelledby="settings-title"
      ref={box}
      onClose={settingsClosed}
      onClick={(event) => event.target === box.current && closeSettings()}
    >
      {open && <Settings />}
    </dialog>
  );
}

export function Settings() {
  const section = useStore(settings, (s) => s.section);
  const visits = useStore(settings, (s) => s.visits);
  return (
    <div className="settings-frame">
      <header className="settings-head">
        <div className="settings-top">
          <div>
            <h2 className="label" id="settings-title">
              {t.title}
            </h2>
            <p>{t.lead}</p>
          </div>
          <span className="settings-keys" aria-hidden="true">
            <kbd>Esc</kbd>
          </span>
          <button className="icon-btn" id="settings-close" title={t.closeHint} aria-label={t.closeSettings} onClick={closeSettings}>
            <Icon svg={ICONS.dismiss} />
          </button>
        </div>
        <SectionTabs at={section} />
      </header>
      <div className="settings-pane" id="settings-pane" role="tabpanel" aria-labelledby={`settings-tab-${section}`}>
        {section === "general" && <GeneralSection key={visits} />}
        {section === "look" && <LookSection key={visits} />}
        {section === "language" && <LanguageSection key={visits} />}
        {section === "providers" && <ProvidersSection key={visits} />}
      </div>
    </div>
  );
}

function SectionTabs({ at }: { at: Section }) {
  const bar = useRef<HTMLDivElement>(null);

  function go(section: Section) {
    showSection(section);
    enterSettings();
  }

  function onKeyDown(event: KeyboardEvent) {
    const from = SECTIONS.indexOf(at);
    const to = ({ ArrowRight: from + 1, ArrowLeft: from - 1, Home: 0, End: SECTIONS.length - 1 } as Record<string, number>)[event.key];
    if (to === undefined) return;
    event.preventDefault();
    const next = SECTIONS[(to + SECTIONS.length) % SECTIONS.length];
    go(next);
    bar.current?.querySelector<HTMLElement>(`[data-tab="${next}"]`)?.focus();
  }

  return (
    <div className="tabs settings-tabs" id="settings-tabs" role="tablist" aria-label={t.sections} ref={bar} onKeyDown={onKeyDown}>
      {SECTIONS.map((id) => (
        <button
          key={id}
          className="tab"
          role="tab"
          id={`settings-tab-${id}`}
          data-tab={id}
          aria-controls="settings-pane"
          aria-selected={id === at}
          tabIndex={id === at ? 0 : -1}
          onClick={() => go(id)}
        >
          {t.section[id]}
        </button>
      ))}
    </div>
  );
}

function LookSection() {
  const chosen = useStore(look, (s) => s.chosen);
  const [fault, setFault] = useState("");

  async function choose(next: Look) {
    setFault("");
    try {
      await chooseLook(next);
    } catch (reason) {
      setFault(String(reason));
    }
  }

  return (
    <>
      <div className="view-focus look-now">
        <div>
          <span className="label">{t.now}</span>
          <p className="tally" aria-live="polite">{`${modeName(chosen.mode)} · ${accentName(chosen.accent)}`}</p>
          <p className="note">{t.accentNote}</p>
        </div>
        <Mark key={chosen.accent} className="look-mark" />
      </div>
      <div className="pair">
        <span className="label">{looks.mode}</span>
        <ModePicker chosen={chosen.mode} pick={(next) => choose({ ...chosen, mode: next })} />
        <p className="note">{t.systemNote}</p>
      </div>
      <div className="pair">
        <span className="label">{looks.color}</span>
        <AccentPicker chosen={chosen.accent} pick={(next) => choose({ ...chosen, accent: next })} />
      </div>
      <p className="note fault" role="alert" hidden={!fault}>
        {fault}
      </p>
    </>
  );
}

function LanguageSection() {
  const { current, fault, choose } = useLanguageChoice();

  return (
    <>
      <div className="pair">
        <span className="label">{looks.language}</span>
        <LanguagePicker chosen={current} pick={choose} />
        <p className="note">{t.languageNote}</p>
      </div>
      <p className="note fault" role="alert" hidden={!fault}>
        {fault}
      </p>
    </>
  );
}

function GeneralSection() {
  const [name, setName] = useState(() => profile.getState().person.name);
  const [saving, setSaving] = useState(false);
  const [said, setSaid] = useState({ text: "", failed: false });

  async function save(event: FormEvent) {
    event.preventDefault();
    setSaving(true);
    setSaid(({ text }) => ({ text, failed: false }));
    try {
      await saveProfileName(name);
      setSaid({ text: t.saved, failed: false });
    } catch (reason) {
      setSaid({ text: String(reason), failed: true });
    }
    setSaving(false);
  }

  return (
    <>
      <div className="pair">
        <label className="label" htmlFor="settings-name">
          {t.name}
        </label>
        <form className="settings-row" onSubmit={save}>
          <input
            className="field"
            id="settings-name"
            value={name}
            onChange={(event) => setName(event.target.value)}
            autoComplete="off"
            spellCheck={false}
          />
          <button className="primary" disabled={saving}>
            {t.save}
          </button>
        </form>
        <p className={said.failed ? "note fault" : "note"} role="status">
          {said.text}
        </p>
      </div>
      <UpdatesBlock />
      <NoticesBlock />
      <VoiceBlock />
      <WelcomeBlock />
    </>
  );
}

function welcomeAgain(step?: Step) {
  closeSettings();
  openWelcome(step);
}

function WelcomeBlock() {
  return (
    <div className="pair">
      <span className="label">{t.welcome}</span>
      <p className="note">{t.welcomeNote}</p>
      <div className="settings-row">
        <button className="quiet" onClick={() => welcomeAgain()}>
          {t.welcomeAgain}
        </button>
        <button className="quiet" onClick={() => welcomeAgain("import")}>
          {t.importClaudeCode}
        </button>
      </div>
    </div>
  );
}

function UpdatesBlock() {
  const known = useStore(updates);
  const { current, latest, installable, checking, fault } = known;
  const state = updateState(known);
  const version = [current && `Sens ${current}`, !installable && t.devBuild].filter(Boolean).join(" · ");
  return (
    <div className="pair">
      <span className="label">{t.updates}</span>
      <p className="note">{version}</p>
      <p className={fault && !checking ? "note fault" : "note"} role="status" hidden={!state}>
        {state}
      </p>
      <div className="settings-row">
        <button className="primary" hidden={!latest} onClick={(event) => openUpdate(event.currentTarget)}>
          {latest ? t.viewVersion(latest.version) : ""}
        </button>
        <button className="quiet" disabled={checking} onClick={() => checkUpdates(true)}>
          {t.checkUpdates}
        </button>
        <button
          className="quiet"
          onClick={() => {
            closeSettings();
            showView("news");
          }}
        >
          {t.whatsNew}
        </button>
      </div>
      <UpdateSwitch />
    </div>
  );
}

function NoticesBlock() {
  const on = useStore(profile, (s) => s.person.notify !== false);
  const [fault, setFault] = useState("");

  async function flip() {
    setFault("");
    try {
      await setNotices(!on);
    } catch (reason) {
      setFault(String(reason));
    }
  }

  return (
    <div className="pair">
      <span className="label">{t.notices}</span>
      <div className="settings-switch">
        <button className="switch" id="settings-notify" role="switch" aria-checked={on} onClick={flip} />
        <label htmlFor="settings-notify">{t.noticesSwitch}</label>
      </div>
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

function UpdateSwitch() {
  const [on, setOn] = useState(() => profile.getState().person.checkUpdates !== false);
  const [fault, setFault] = useState("");

  async function flip() {
    const next = !on;
    setOn(next);
    setFault("");
    try {
      await setAutomatic(next);
    } catch (reason) {
      setOn(!next);
      setFault(String(reason));
    }
  }

  return (
    <>
      <div className="settings-switch">
        <button className="switch" id="settings-update-check" role="switch" aria-checked={on} onClick={flip} />
        <label htmlFor="settings-update-check">{t.checkAtStart}</label>
      </div>
      <p className="note fault" role="alert" hidden={!fault}>
        {fault}
      </p>
    </>
  );
}
