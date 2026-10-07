import { Fragment, useEffect, useMemo, useState, type MouseEvent, type ReactNode } from "react";
import { code, shared } from "../copy";
import { FileIcon } from "../FileIcon";
import { Icon } from "../Icon";
import { ICONS } from "../icons.js";
import { openOutside } from "../outside";
import { isPullUrl, openPlace, placeOf, placesOpen, pullsIn, type Place } from "../references";
import { colored, coloredRuns } from "../syntax/colored";
import { useCode } from "../syntax/code";
import { languageNamed, titleOf } from "../syntax/languages";
import { grammarOf, sessionOf, type SessionLine, type Shell } from "../syntax/shells";
import { parse, type Block, type Inline, type List } from "./parse";

export interface Fade {
  stamps: { from: number; time: number }[];
  now: number;
}

interface Cursor {
  at: number;
  fade?: Fade;
}

export function Markdown({ text, fade, className }: { text: string; fade?: Fade; className?: string }) {
  const blocks = useMemo(() => parse(text), [text]);
  const cursor: Cursor = { at: 0, fade };
  return <div className={className ? `prose ${className}` : "prose"}>{blocks.map((block, at) => <Fragment key={at}>{renderBlock(block, cursor)}</Fragment>)}</div>;
}

function renderBlock(block: Block, cursor: Cursor): ReactNode {
  switch (block.kind) {
    case "p":
      return <p>{spell(block.inline, cursor)}</p>;
    case "h": {
      const Heading = `h${block.level}` as "h1";
      return <Heading>{spell(block.inline, cursor)}</Heading>;
    }
    case "quote":
      return <blockquote>{spell(block.inline, cursor)}</blockquote>;
    case "hr":
      return <hr />;
    case "code":
      cursor.at += block.text.length;
      return <CodeBlock text={block.text} language={block.language} />;
    case "table":
      return (
        <div className="table">
          <table>
            <thead>
              <tr>
                {block.head.map((cell, at) => (
                  <th key={at}>{spell(cell, cursor)}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {block.rows.map((row, at) => (
                <tr key={at}>
                  {row.map((cell, column) => (
                    <td key={column}>{spell(cell, cursor)}</td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
    case "list":
      return renderList(block, cursor);
  }
}

function renderList(list: List, cursor: Cursor) {
  const items = list.items.map((item, at) => <li key={at}>{spell(item, cursor)}</li>);
  return list.ordered ? <ol start={list.start}>{items}</ol> : <ul>{items}</ul>;
}

function spell(nodes: (Inline | List)[], cursor: Cursor) {
  return nodes.map((node, at) => <Fragment key={at}>{renderInline(node, cursor)}</Fragment>);
}

function renderInline(node: Inline | List, cursor: Cursor): ReactNode {
  switch (node.kind) {
    case "text":
      return pullsIn(node.text).map((piece, at) =>
        typeof piece === "string" ? (
          <Fragment key={at}>{faded(piece, cursor)}</Fragment>
        ) : (
          <a key={at} className="ref-pull" href={piece.url} title={code.openOnline(piece.ref)} onClick={(event) => outside(event, piece.url)}>
            <Icon svg={ICONS.pullRequest} />
            {faded(piece.ref, cursor)}
          </a>
        ),
      );
    case "code": {
      const place = placeOf(node.text);
      return place ? <PlaceRef place={place}>{faded(node.text, cursor)}</PlaceRef> : <code>{faded(node.text, cursor)}</code>;
    }
    case "strong":
      return <strong>{spell(node.children, cursor)}</strong>;
    case "em":
      return <em>{spell(node.children, cursor)}</em>;
    case "link":
      return (
        <a href={node.url} title={node.url} className={isPullUrl(node.url) ? "ref-pull" : undefined} onClick={(event) => outside(event, node.url)}>
          {isPullUrl(node.url) && <Icon svg={ICONS.pullRequest} />}
          {spell(node.children, cursor)}
        </a>
      );
    case "list":
      return renderList(node, cursor);
  }
}

function outside(event: MouseEvent, url: string) {
  event.preventDefault();
  openOutside(url);
}

function PlaceRef({ place, children }: { place: Place; children: ReactNode }) {
  const face = (
    <>
      <FileIcon path={place.path} />
      <span>{children}</span>
    </>
  );
  if (!placesOpen()) return <code className="ref-file">{face}</code>;
  return (
    <button type="button" className="ref-file" title={code.openPlace(place.path)} onClick={() => openPlace(place)}>
      {face}
    </button>
  );
}

function faded(text: string, cursor: Cursor): ReactNode {
  const start = cursor.at;
  const end = start + text.length;
  cursor.at = end;
  const stamps = cursor.fade?.stamps ?? [];
  if (!stamps.length || end <= stamps[0].from) return text;
  const bounds = [start, ...stamps.map((stamp) => stamp.from).filter((at) => at > start && at < end), end];
  return bounds.slice(0, -1).map((from, at) => {
    const piece = text.slice(from - start, bounds[at + 1] - start);
    const stamp = stamps.findLast((one) => one.from <= from);
    if (!stamp) return piece;
    return (
      <span key={at} className="fresh" style={{ animationDelay: `-${Math.round(cursor.fade!.now - stamp.time)}ms` }}>
        {piece}
      </span>
    );
  });
}

const CODE_FOLD = 30;

export function CodeBlock({ text, language = "" }: { text: string; language?: string }) {
  const session = useMemo(() => sessionOf(text, language), [text, language]);
  const grammar = languageNamed(language);
  const painted = useCode(session ? "" : text, grammar);
  const lines = text.split("\n").length;
  const [unfolded, setUnfolded] = useState(false);
  const folded = lines > CODE_FOLD && !unfolded;

  return (
    <div className="codeblock" data-session={session ? "true" : undefined} data-folded={lines > CODE_FOLD ? String(folded) : undefined}>
      <div className="codeblock-head">
        <span>{labelOf(language, Boolean(session))}</span>
        <CopyButton text={text} />
      </div>
      <pre>
        <code>{session ? <Session lines={session} /> : painted ? colored(painted) : text}</code>
      </pre>
      {folded && (
        <button className="unfold" type="button" onClick={() => setUnfolded(true)}>
          {code.allLines(lines)}
        </button>
      )}
    </div>
  );
}

const PLAIN = /^(?:text|txt|plain|plaintext)$/i;

function labelOf(language: string, session: boolean) {
  if (session || languageNamed(language) === "shellsession") return code.console;
  if (!language.trim()) return code.code;
  if (PLAIN.test(language.trim())) return code.text;
  return titleOf(language) ?? language;
}

function Session({ lines }: { lines: SessionLine[] }) {
  const typed = (shell: Shell) =>
    lines
      .filter((line) => line.shell === shell)
      .map((line) => line.text)
      .join("\n");
  const painted = {
    bash: useCode(typed("bash"), grammarOf("bash")),
    powershell: useCode(typed("powershell"), grammarOf("powershell")),
    cmd: useCode(typed("cmd"), grammarOf("cmd")),
  };
  const seen: Record<Shell, number> = { bash: 0, powershell: 0, cmd: 0 };
  return lines.map((line, at) => {
    const own = line.shell && painted[line.shell];
    const runs = line.shell && own ? own.lines[seen[line.shell]] : undefined;
    if (line.shell) seen[line.shell] += 1;
    return (
      <Fragment key={at}>
        {at > 0 && "\n"}
        {line.prompt && <span className="prompt">{line.prompt}</span>}
        {line.shell ? <span className="command">{runs && own ? coloredRuns(runs, own.looks) : line.text}</span> : line.text}
      </Fragment>
    );
  });
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  const [fault, setFault] = useState("");

  useEffect(() => {
    if (!copied) return;
    const back = setTimeout(() => setCopied(false), 1500);
    return () => clearTimeout(back);
  }, [copied]);

  async function copy() {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
    } catch (reason) {
      setFault(String(reason));
    }
  }

  return (
    <button className="copy" type="button" title={fault || (copied ? shared.copied : shared.copy)} aria-label={shared.copy} onClick={copy}>
      <Icon svg={copied ? ICONS.check : ICONS.copy} />
    </button>
  );
}
