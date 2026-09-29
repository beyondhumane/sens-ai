import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";
import { useStore } from "zustand";
import { Icon } from "../shared/Icon";
import { ICONS } from "../shared/icons.js";
import { stem } from "../shared/format.js";
import { choicesOf, keyOf, titleOf, type Choice } from "./choices";
import { t } from "./copy";
import { bar, choose, filterChoices, firstQuestion, own, resumeSession, toggleChoosing } from "./store";

const backToField = () => document.getElementById("task")?.focus();

export function SessionChip() {
  const root = useStore(own.desk, (s) => s.root);
  const session = useStore(own.desk, (s) => s.session);
  const asked = useStore(own.chat, (s) => firstQuestion(s.turns));
  const projects = useStore(bar, (s) => s.projects);
  const sessions = useStore(bar, (s) => s.sessions);
  const choosing = useStore(bar, (s) => s.choosing);
  if (!root) return <span className="bar-project none">{t.noProject}</span>;
  const name = projects.find((one) => one.root === root)?.name ?? stem(root);
  const title = titleOf(sessions, root, session) || asked || t.newSession;
  return (
    <button
      type="button"
      className="bar-project"
      aria-haspopup="listbox"
      aria-expanded={choosing}
      aria-controls="bar-choices"
      title={t.where(name, title)}
      aria-label={t.where(name, title)}
      onClick={toggleChoosing}
    >
      <Icon svg={ICONS.folderSmall} />
      <span className="bar-project-name">{name}</span>
      <span className="bar-project-session">{title}</span>
      <Icon svg={ICONS.caret} />
    </button>
  );
}

function groupOf(choice: Choice, filtering: boolean) {
  if (filtering) return choice.kind === "session" ? t.sessions : t.projects;
  return choice.kind === "project" ? t.otherProjects : choice.name;
}

function pick(choice: Choice) {
  if (choice.kind === "session") void resumeSession(choice.root, choice.id);
  else choose(choice.root);
  backToField();
}

export function Picker() {
  const choosing = useStore(bar, (s) => s.choosing);
  const projects = useStore(bar, (s) => s.projects);
  const sessions = useStore(bar, (s) => s.sessions);
  const filter = useStore(bar, (s) => s.filter);
  const root = useStore(own.desk, (s) => s.root);
  const session = useStore(own.desk, (s) => s.session);
  const search = useRef<HTMLInputElement>(null);
  const list = useRef<HTMLDivElement>(null);
  const choices = useMemo(() => choicesOf(projects, sessions, root, filter), [projects, sessions, root, filter]);
  const current = choices.findIndex((choice) => (choice.kind === "session" ? choice.id === session : choice.kind === "new" && !session));
  const [active, setActive] = useState(0);

  useEffect(() => {
    if (!choosing) return;
    setActive(Math.max(current, 0));
    search.current?.focus();
  }, [choosing]);

  useEffect(() => setActive(0), [filter]);

  useEffect(() => {
    list.current?.querySelector('[data-active="true"]')?.scrollIntoView({ block: "nearest" });
  }, [active]);

  if (!choosing) return null;

  function keyDown(event: KeyboardEvent<HTMLInputElement>) {
    const step = ({ ArrowDown: 1, ArrowUp: -1 } as Record<string, number>)[event.key];
    if (step && choices.length) {
      event.preventDefault();
      setActive((at) => (at + step + choices.length) % choices.length);
    } else if (event.key === "Enter" && choices[active]) {
      event.preventDefault();
      pick(choices[active]);
    }
  }

  const filtering = Boolean(filter.trim());
  const at = choices[active];
  return (
    <div className="bar-picker">
      <input
        ref={search}
        className="bar-picker-search"
        role="combobox"
        aria-expanded="true"
        aria-controls="bar-choices"
        aria-activedescendant={at ? `bar-choice-${keyOf(at)}` : undefined}
        aria-label={t.search}
        placeholder={t.search}
        autoComplete="off"
        spellCheck={false}
        value={filter}
        onChange={(event) => filterChoices(event.target.value)}
        onKeyDown={keyDown}
      />
      <div className="bar-projects" id="bar-choices" role="listbox" aria-label={t.choices} ref={list}>
        {!choices.length && <p className="bar-picker-none">{t.nothingFound}</p>}
        {choices.map((choice, index) => {
          const group = groupOf(choice, filtering);
          const heading = index === 0 || groupOf(choices[index - 1], filtering) !== group;
          const chosen = index === current;
          return (
            <div key={keyOf(choice)} className="bar-choice-group" role="presentation">
              {heading && (
                <span className="bar-choice-heading" role="presentation">
                  {group}
                </span>
              )}
              <button
                type="button"
                role="option"
                id={`bar-choice-${keyOf(choice)}`}
                className="bar-project-row"
                tabIndex={-1}
                aria-selected={chosen}
                data-active={index === active ? "true" : undefined}
                onPointerMove={() => setActive(index)}
                onClick={() => pick(choice)}
              >
                <Icon svg={choice.kind === "session" ? ICONS.messageSmall : choice.kind === "new" ? ICONS.messagePlusSmall : ICONS.folderSmall} />
                <span className="project-name">{choice.kind === "session" ? choice.title : choice.kind === "new" ? t.newSession : choice.name}</span>
                <span className="mono">{choice.kind === "session" && filtering ? choice.name : choice.kind === "project" ? t.newSessionThere : ""}</span>
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
}
