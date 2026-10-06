import { memo, useLayoutEffect, useMemo, useRef, useState } from "react";
import { useStore } from "zustand";
import { openPicture } from "../../app/Dialog";
import { EmptyView } from "../../shared/EmptyView";
import { FRONT_MATTER, stem, weigh } from "../../shared/format.js";
import { ICONS } from "../../shared/icons.js";
import { Markdown } from "../../shared/markdown/Markdown";
import { useCode } from "../../shared/syntax/code";
import { languageOf } from "../../shared/syntax/languages";
import type { Look, Painted, Runs } from "../../shared/syntax/paint";
import { useSeen } from "../../shared/useSeen";
import { project, type Edits } from "../project/store";
import { t } from "./copy";
import { setMode, textOf, viewer, viewOf, type Body } from "./view";

export function Viewer() {
  const title = useStore(viewer, (s) => s.title);
  const mode = useStore(viewer, (s) => s.mode);
  const body = useStore(viewer, (s) => s.body);
  const shown = useStore(viewer, (s) => s.shown);
  if (body.kind === "picture") return <Picture key={shown} title={title} data={body.data} bytes={body.bytes} />;
  if (body.kind !== "text") return <Notice {...noticeOf(body)} />;
  const reading = mode === "view" && viewOf(title) === "reading";
  return (
    <>
      <Source hidden={reading} />
      {viewOf(title) === "reading" && <Reading hidden={!reading} />}
    </>
  );
}

const useEdits = () => {
  const opened = useStore(viewer, (s) => s.opened);
  return useStore(project, (s) => (opened ? s.touched.get(opened) : undefined));
};

export function ViewerHead() {
  const title = useStore(viewer, (s) => s.title) || t.noneOpen;
  const edits = useEdits();
  return (
    <>
      <span className="where" title={title}>
        {title}
      </span>
      <span className="marks">
        {edits && (
          <>
            <span className="plus">+{edits.plus}</span>
            <span className="minus">−{edits.minus}</span>
          </>
        )}
      </span>
    </>
  );
}

export function ViewerModes() {
  const title = useStore(viewer, (s) => s.title);
  const mode = useStore(viewer, (s) => s.mode);
  const text = useStore(viewer, (s) => s.body.kind === "text");
  const kind = viewOf(title);
  if (!kind || !text) return null;
  return (
    <div className="segment">
      <button type="button" aria-pressed={mode === "source"} onClick={() => setMode("source")}>
        {t.code}
      </button>
      <button type="button" aria-pressed={kind === "reading" && mode === "view"} onClick={() => setMode("view")}>
        {t.view}
      </button>
    </div>
  );
}

function useOpensAtTop<Box extends HTMLElement>() {
  const box = useRef<Box>(null);
  const shown = useStore(viewer, (s) => s.shown);
  useLayoutEffect(() => {
    const at = box.current;
    if (!at) return;
    const { line } = viewer.getState();
    at.scrollTop = line ? topOf(at, line) : 0;
    if (!line) at.querySelector('[data-touched="add"]')?.scrollIntoView({ block: "center" });
  }, [shown]);
  return box;
}

const topOf = (box: HTMLElement, line: number) => Math.max(0, (line - 1) * (parseFloat(getComputedStyle(box).getPropertyValue("--line")) || 0) - box.clientHeight / 2);

function Source({ hidden }: { hidden: boolean }) {
  const title = useStore(viewer, (s) => s.title);
  const text = useStore(viewer, (s) => textOf(s.body));
  const shown = useStore(viewer, (s) => s.shown);
  const edits = useEdits();
  const focus = useStore(viewer, (s) => s.line);
  const box = useOpensAtTop<HTMLDivElement>();
  const lines = useMemo(() => text.split(/\r?\n/), [text]);
  const code = useMemo(() => lines.join("\n"), [lines]);
  const language = useMemo(() => languageOf(title, text.slice(0, 200)), [title, text]);
  const painted = useCode(code, language);
  const colored = painted?.lines.length === lines.length ? painted : null;

  const blocks = [];
  for (let from = 0; from < lines.length; from += BLOCK) {
    blocks.push(<Block key={`${shown}/${from}`} from={from} lines={lines} colored={colored} edits={edits} focus={focus} />);
  }
  return (
    <div className="source" ref={box} hidden={hidden}>
      {title ? blocks : <p className="empty">{t.choose}</p>}
    </div>
  );
}

const BLOCK = 200;

interface BlockProps {
  from: number;
  lines: string[];
  colored: Painted | null;
  edits?: Edits;
  focus: number;
}

const Block = memo(function Block({ from, lines, colored, edits, focus }: BlockProps) {
  const box = useRef<HTMLDivElement>(null);
  const to = Math.min(from + BLOCK, lines.length);
  const touched = edits ? [...edits.add].some((line) => line > from && line <= to) : false;
  const seen = useSeen(box, { root: () => box.current?.closest(".source") ?? null, margin: "1500px 0px", now: from === 0 || touched || (focus > from && focus <= to) });
  if (!seen) return <div className="block" ref={box} style={{ height: `calc(var(--line) * ${to - from})` }} />;

  const rows = [];
  for (let at = from; at < to; at++) {
    rows.push(<Line key={at} number={at + 1} runs={colored?.lines[at] ?? lines[at]} looks={colored?.looks} added={edits?.add.has(at + 1) ?? false} focused={focus === at + 1} />);
  }
  return (
    <div className="block" ref={box}>
      {rows}
    </div>
  );
});

interface LineProps {
  number: number;
  runs: Runs | string;
  looks?: Look[];
  added: boolean;
  focused: boolean;
}

const Line = memo(function Line({ number, runs, looks, added, focused }: LineProps) {
  return (
    <div className="line" data-touched={added ? "add" : undefined} data-focused={focused ? "true" : undefined}>
      <span className="num">{number}</span>
      <span className="src">
        {typeof runs === "string"
          ? runs
          : runs.map(([text, look], at) => (look < 0 ? text : <span key={at} style={looks![look]}>{text}</span>))}
      </span>
    </div>
  );
});

function Reading({ hidden }: { hidden: boolean }) {
  const text = useStore(viewer, (s) => textOf(s.body));
  const box = useOpensAtTop<HTMLDivElement>();
  return (
    <div className="reading" ref={box} hidden={hidden}>
      <Markdown text={text.replace(FRONT_MATTER, "")} />
    </div>
  );
}

function Picture({ title, data, bytes }: { title: string; data: string; bytes: number }) {
  const [drawn, setDrawn] = useState("");
  const [broken, setBroken] = useState(false);
  if (broken) return <Notice art={ICONS.image} lead={t.unreadable} said={t.unreadableSaid(weigh(bytes))} />;
  return (
    <div className="sight-view">
      <button type="button" className="frame" title={t.enlarge} onClick={(event) => openPicture(stem(title), data, event.currentTarget)}>
        <img
          alt={stem(title)}
          src={data}
          onLoad={({ currentTarget: { naturalWidth, naturalHeight } }) => setDrawn(`${naturalWidth} × ${naturalHeight}`)}
          onError={() => setBroken(true)}
        />
      </button>
      <p className="size">{drawn ? `${drawn} · ${weigh(bytes)}` : weigh(bytes)}</p>
    </div>
  );
}

type Said = { art: string; lead: string; said: string };

function noticeOf(body: Exclude<Body, { kind: "text" | "picture" }>): Said {
  if (body.kind === "fault") return { art: ICONS.info, lead: t.openFailed, said: body.fault };
  if (body.kind === "tooBig") return { art: ICONS.info, lead: t.tooBig, said: t.tooBigSaid(weigh(body.bytes), weigh(body.cap)) };
  return { art: ICONS.info, lead: t.noPreview, said: t.noPreviewSaid(weigh(body.bytes)) };
}

function Notice(said: Said) {
  return (
    <div className="file-note" role="status">
      <EmptyView {...said} />
    </div>
  );
}
