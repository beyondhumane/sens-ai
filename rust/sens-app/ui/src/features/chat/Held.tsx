import { memo, useEffect, useState } from "react";
import { commands } from "../../ipc/commands";
import { Icon } from "../../shared/Icon";
import { ICONS } from "../../shared/icons.js";
import { sendPlain } from "../composer/store";
import { usePane } from "../panes/context";
import { workOf } from "../panes/store";
import { t } from "./canon.copy";
import { FindingRow } from "./SensStep";
import type { Held as HeldPart } from "./turns";

type Standing = "checking" | "open" | "busy" | "settled";

export const UNJUDGED = "unjudged";

export const Held = memo(function Held({ part }: { part: HeldPart }) {
  const pane = usePane();
  const [standing, setStanding] = useState<Standing>("checking");
  const [note, setNote] = useState("");
  const [fault, setFault] = useState("");
  const unjudged = part.findings.some((finding) => finding.key === UNJUDGED);
  const shown = part.findings.filter((finding) => finding.key !== UNJUDGED);

  useEffect(() => {
    let live = true;
    commands.canonHeld(workOf(pane)).then(
      (held) => {
        if (!live) return;
        setStanding(held ? "open" : "settled");
        if (!held) setNote(t.resolved);
      },
      () => live && setStanding("open"),
    );
    return () => {
      live = false;
    };
  }, [pane]);

  async function act(action: () => Promise<string>) {
    setStanding("busy");
    setFault("");
    try {
      setNote(await action());
      setStanding("settled");
    } catch (reason) {
      setFault(String(reason));
      setStanding("open");
    }
  }

  const work = () => workOf(pane);
  const fix = () =>
    act(async () => {
      const asked = await commands.canonFix(work());
      if (asked) await sendPlain(asked, pane);
      return t.fixing;
    });
  const retry = () =>
    act(async () => {
      await commands.canonRetry(pane.desk.getState().session);
      return t.retrying;
    });
  const undo = () =>
    act(async () => {
      const { restored, skipped } = await commands.canonUndo(work());
      return [t.undone(restored.length), skipped.length ? t.kept(skipped.length) : ""].filter(Boolean).join(" · ");
    });
  const accept = () =>
    act(async () => {
      await commands.canonAccept(work());
      return t.accepted;
    });

  const choices: [string, boolean, () => void][] = [
    unjudged ? [t.retry, true, retry] : [t.fix, true, fix],
    [t.undo, false, undo],
    [t.accept, false, accept],
  ];
  const open = standing === "open" || standing === "busy";

  return (
    <div className="ask held" data-state={open ? "waiting" : "settled"} role="group" aria-label={unjudged ? t.unjudgedTitle : t.heldTitle}>
      <div className="ask-head">
        <span className="ask-icon">
          <Icon svg={ICONS.shieldAlert} />
        </span>
        <span className="ask-title">{unjudged ? t.unjudgedTitle : t.heldTitle}</span>
      </div>
      <div className="ask-body">
        {open && <p className="ask-note">{unjudged ? t.unjudgedLead : t.heldLead}</p>}
        {shown.map((finding) => (
          <FindingRow key={finding.key} finding={finding} />
        ))}
        {unjudged && part.findings.filter((finding) => finding.key === UNJUDGED).map((finding) => <p key={finding.key} className="sens-why">{finding.message}</p>)}
      </div>
      <div className="ask-actions" hidden={!open}>
        {open &&
          choices.map(([label, primary, run]) => (
            <button key={label} type="button" className={primary ? "primary" : "quiet"} disabled={standing === "busy"} onClick={run}>
              {label}
            </button>
          ))}
      </div>
      {note && <p className="ask-note">{note}</p>}
      {fault && <p className="ask-note fault">{fault}</p>}
    </div>
  );
});
