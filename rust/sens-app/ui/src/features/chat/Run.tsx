import { memo, useState } from "react";
import { seconds } from "../../shared/format.js";
import { Icon } from "../../shared/Icon";
import { ICONS } from "../../shared/icons.js";
import { t as said } from "./canon.copy";
import { SensStep } from "./SensStep";
import { Step } from "./Step";
import { Thought } from "./Thought";
import { t } from "./thread.copy";
import { sameWork, tally, type Tally, type Work } from "./work";

export const Run = memo(
  function Run({ parts, folded }: { parts: Work[]; folded: boolean }) {
    const [opened, setOpened] = useState(false);
    const told = folded ? tally(parts) : null;
    return (
      <details
        className="run"
        open={!folded || opened}
        data-folded={String(folded)}
        data-failed={told?.failed ? "true" : undefined}
        onToggle={(event) => folded && setOpened(event.currentTarget.open)}
      >
        <summary className="run-head" hidden={!told}>
          {told && <Head told={told} />}
        </summary>
        <div className="run-steps">
          {parts.map((part) => {
            if (part.kind === "thought") return <Thought key={part.key} part={part} />;
            if (part.kind === "sens") return <SensStep key={part.key} part={part} />;
            return <Step key={part.key} part={part} />;
          })}
        </div>
      </details>
    );
  },
  (was, now) => was.folded === now.folded && sameWork(was.parts, now.parts),
);

function Head({ told }: { told: Tally }) {
  const counts = [
    told.commands && t.commands(told.commands),
    told.reads && t.reads(told.reads),
    told.edits && t.edits(told.edits),
    told.searches && t.searches(told.searches),
    told.others && t.others(told.others),
  ].filter(Boolean);
  return (
    <>
      <span className="run-icon">
        <Icon svg={ICONS.shut} />
      </span>
      <span className="run-verb">{t.worked}</span>
      <span className="run-tally">{counts.join(" · ")}</span>
      {told.failed > 0 && <span className="run-failed">{t.failed(told.failed)}</span>}
      <SensTally told={told} />
      <span className="run-time">{told.took === null ? "" : seconds(told.took)}</span>
      <span className="run-state" />
    </>
  );
}

function SensTally({ told }: { told: Tally }) {
  const quiet = [told.stops > 0 && said.stops(told.stops)].filter(Boolean);
  const lit = [told.reused > 0 && said.reused(told.reused), told.approved && said.approved].filter(Boolean);
  if (!quiet.length && !lit.length) return null;
  return (
    <span className="run-sens">
      {said.tally("")}
      {quiet.join(" · ")}
      {quiet.length > 0 && lit.length > 0 && " · "}
      {lit.length > 0 && <span className="run-sens-lit">{lit.join(" · ")}</span>}
    </span>
  );
}
