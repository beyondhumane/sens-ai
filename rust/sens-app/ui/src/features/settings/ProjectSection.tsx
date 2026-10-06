import { useEffect, useState } from "react";
import { useStore } from "zustand";
import { commands } from "../../ipc/commands";
import type { Avoided, Excepted, ProjectRules } from "../../ipc/types";
import { when } from "../../shared/format.js";
import { t as sens } from "../chat/canon.copy";
import { project } from "../project/store";
import { t } from "./project.copy";
import { Switch } from "./Switch";

const MONTH = 30 * 24 * 60 * 60 * 1000;

export function avoidedLines(avoided: Avoided) {
  return [
    avoided.copies && t.copies(avoided.copies),
    avoided.dependencies && t.dependencies(avoided.dependencies),
    avoided.comments && t.comments(avoided.comments),
    avoided.protected && t.protectedFiles(avoided.protected),
    avoided.tests && t.tests(avoided.tests),
    avoided.orphans && t.orphans(avoided.orphans),
    avoided.cycles && t.cycles(avoided.cycles),
    avoided.judgment && t.judgment(avoided.judgment),
    avoided.held && t.held(avoided.held),
    avoided.reviews && t.reviews(avoided.reviews, `$${avoided.reviewerCost.toFixed(2)}`),
  ].filter((line): line is string => Boolean(line));
}

export function ProjectSection() {
  const work = useStore(project, (state) => state.work);
  const [rules, setRules] = useState<ProjectRules | null>(null);
  const [excepted, setExcepted] = useState<Excepted[]>([]);
  const [avoided, setAvoided] = useState<Avoided | null>(null);
  const [fault, setFault] = useState("");

  useEffect(() => {
    if (!work) return;
    commands.canonRules(work).then(setRules, (reason) => setFault(String(reason)));
    commands.canonExceptions(work).then(setExcepted, () => {});
    commands.canonAvoided(work, Date.now() - MONTH).then(setAvoided, () => {});
  }, [work]);

  if (!work) return <p className="note">{t.noProject}</p>;

  async function strict(noComments: boolean) {
    const chosen = { ...(rules ?? { noComments: false }), noComments };
    await commands.canonSetRules(work, chosen);
    setRules(chosen);
  }

  async function retract(key: string) {
    setFault("");
    try {
      await commands.canonRetract(work, key);
      setExcepted((known) => known.filter((one) => one.key !== key));
    } catch (reason) {
      setFault(String(reason));
    }
  }

  const lines = avoided ? avoidedLines(avoided) : [];

  return (
    <>
      <div className="pair">
        <span className="label">{t.rules}</span>
        <Switch id="settings-no-comments" on={rules?.noComments ?? false} save={strict}>
          {t.noComments}
        </Switch>
        <p className="note">{t.noCommentsNote}</p>
      </div>
      <div className="pair">
        <span className="label">{t.exceptions}</span>
        <p className="note">{t.exceptionsNote}</p>
        {excepted.length === 0 ? (
          <p className="note">{t.noExceptions}</p>
        ) : (
          <ul className="settings-list">
            {excepted.map((one) => (
              <li key={one.key}>
                <span className="settings-list-main">{one.rule ? (sens.rules[one.rule] ?? one.rule) : one.key}</span>
                <span className="settings-list-path mono">{one.file}</span>
                <span className="settings-list-when">{one.since ? when(one.since) : ""}</span>
                <button className="quiet" onClick={() => void retract(one.key)}>
                  {t.retract}
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
      <div className="pair">
        <span className="label">{t.avoided}</span>
        {lines.length === 0 ? (
          <p className="note">{t.nothingYet}</p>
        ) : (
          <ul className="settings-facts">
            {lines.map((line) => (
              <li key={line}>{line}</li>
            ))}
          </ul>
        )}
      </div>
      <p className="note fault" role="alert" hidden={!fault}>
        {fault}
      </p>
    </>
  );
}
