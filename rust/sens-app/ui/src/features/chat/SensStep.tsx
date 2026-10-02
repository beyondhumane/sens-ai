import { memo } from "react";
import type { Finding, Suggested } from "../../ipc/types";
import { Icon } from "../../shared/Icon";
import { ICONS } from "../../shared/icons.js";
import { CodeBlock } from "../../shared/markdown/Markdown";
import { languageOf } from "../../shared/syntax/languages";
import { showFile } from "../files/view";
import { t } from "./canon.copy";
import type { Sens } from "./turns";
import { stopping } from "./work";

type Tone = "found" | "stop" | "note" | "pass" | "quiet";

const SHOWN_NAMES = 3;

export function lookOf(part: Sens): { icon: string; said: string; tone: Tone } {
  if (part.stage === "anticipated") return { icon: ICONS.search, said: t.found(part.suggestions.slice(0, SHOWN_NAMES).map((found) => found.name).join(", ")), tone: "found" };
  if (part.stage === "passed") return { icon: ICONS.shieldCheck, said: t.passed, tone: "pass" };
  if (part.stage === "pending") return { icon: ICONS.shield, said: t.pending, tone: "quiet" };
  if (part.stage === "detached") return { icon: ICONS.shieldOff, said: t.detached, tone: "stop" };
  if (part.stage === "reviewed") return { icon: ICONS.shield, said: t.reviewed, tone: "quiet" };
  if (stopping(part)) return { icon: ICONS.shieldAlert, said: t.stopped[part.stage] ?? t.stopped.blocked, tone: "stop" };
  return { icon: ICONS.shield, said: t.noted(part.findings.length), tone: "note" };
}

export const SensStep = memo(function SensStep({ part }: { part: Sens }) {
  const look = lookOf(part);
  const empty = part.findings.length === 0 && part.suggestions.length === 0;
  return (
    <details className={empty ? "step sens empty" : "step sens"} data-tone={look.tone}>
      <summary onClick={(event) => empty && event.preventDefault()}>
        <span className="step-icon">
          <Icon svg={look.icon} />
        </span>
        <span className="step-verb">{t.sens}</span>
        <span className="step-target">{look.said}</span>
        <span className="step-meta">{part.stage === "reviewed" && part.findings.length > 0 ? t.noted(part.findings.length) : ""}</span>
        <span className="step-state" />
      </summary>
      <div className="step-body">
        {part.suggestions.map((found) => (
          <Offered key={`${found.file}:${found.line}:${found.name}`} found={found} />
        ))}
        {part.findings.map((finding) => (
          <FindingRow key={finding.key} finding={finding} />
        ))}
      </div>
    </details>
  );
});

function Place({ file, line }: { file: string; line: number }) {
  return (
    <button type="button" className="sens-place mono" title={file} onClick={() => showFile(file)}>
      {line ? `${file}:${line}` : file}
    </button>
  );
}

function Offered({ found }: { found: Suggested }) {
  return (
    <div className="sens-found">
      <code className="sens-signature">{found.signature.trim()}</code>
      <Place file={found.file} line={found.line} />
      <span className="sens-uses">{t.uses(found.uses)}</span>
    </div>
  );
}

export function FindingRow({ finding }: { finding: Finding }) {
  const judged = finding.rule.startsWith("S");
  return (
    <div className="sens-finding" data-severity={finding.severity}>
      <p className="sens-rule">
        <span>{t.rules[finding.rule] ?? finding.rule}</span>
        {finding.file && <Place file={finding.file} line={finding.line} />}
      </p>
      {judged && finding.message && <p className="sens-why">{finding.message}</p>}
      {finding.target && (
        <div className="sens-target">
          <p className="sens-rule">
            <span>{t.existing}</span>
            <Place file={finding.target.file} line={finding.target.line} />
          </p>
          <CodeBlock text={finding.target.excerpt} language={languageOf(finding.target.file) ?? ""} />
        </div>
      )}
    </div>
  );
}
