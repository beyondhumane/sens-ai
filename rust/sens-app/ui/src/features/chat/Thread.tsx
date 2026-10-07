import { memo, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { useStore } from "zustand";
import { look } from "../../shared/look";
import { useIds, usePane } from "../panes/context";
import { grain } from "./grain";
import { Reply } from "./Reply";
import { HINTS, t } from "./thread.copy";
import type { Notice as NoticeTurn } from "./turns";
import { You } from "./You";

const NEAR_BOTTOM = 160;

export function Thread() {
  const pane = usePane();
  const id = useIds();
  const turns = useStore(pane.chat, (s) => s.turns);
  const hint = useStore(pane.chat, (s) => s.hint);
  const replaying = useStore(pane.chat, (s) => s.replaying);
  const spoken = useStore(pane.chat, (s) => s.spoken);
  const thread = useRef<HTMLDivElement>(null);
  const inner = useRef<HTMLDivElement>(null);
  const stick = useRef(true);
  const [edges, setEdges] = useState({ over: false, under: false });

  const paint = () => {
    const box = thread.current;
    if (!box) return;
    const room = box.scrollHeight - box.clientHeight;
    const over = box.scrollTop > 8;
    const under = room - box.scrollTop > 8;
    setEdges((was) => (was.over === over && was.under === under ? was : { over, under }));
  };

  useEffect(() => {
    const follow = new ResizeObserver(() => {
      if (stick.current && thread.current) thread.current.scrollTop = thread.current.scrollHeight;
      paint();
    });
    if (inner.current) follow.observe(inner.current);
    if (thread.current) follow.observe(thread.current);
    return () => follow.disconnect();
  }, []);

  useLayoutEffect(() => {
    stick.current = true;
    if (thread.current) thread.current.scrollTop = thread.current.scrollHeight;
  }, [turns.length]);

  return (
    <div className="stream" id={id("stream")} data-over={String(edges.over)} data-under={String(edges.under)}>
      <div
        className="thread"
        id={id("thread")}
        ref={thread}
        data-replaying={replaying ? "true" : undefined}
        onScroll={() => {
          const box = thread.current!;
          stick.current = box.scrollHeight - box.scrollTop - box.clientHeight < NEAR_BOTTOM;
          paint();
        }}
      >
        <div className="thread-inner" id={id("thread-inner")} ref={inner}>
          {turns.length ? (
            turns.map((turn) =>
              turn.kind === "you" ? <You key={turn.key} turn={turn} /> : turn.kind === "notice" ? <Notice key={turn.key} turn={turn} /> : <Reply key={turn.key} turn={turn} />,
            )
          ) : (
            <Hello hint={hint} />
          )}
        </div>
      </div>
      <p className="spoken" role="status">
        {spoken && <span key={spoken.nth}>{spoken.said}</span>}
      </p>
      <div className="fade top" aria-hidden="true" />
      <div className="fade bottom" aria-hidden="true" />
    </div>
  );
}

const Notice = memo(function Notice({ turn }: { turn: NoticeTurn }) {
  return (
    <div className={turn.tone ? `tick ${turn.tone}` : "tick"}>
      <span className="dot" />
      <span>{turn.parts.map((part, at) => (typeof part === "string" ? part : <b key={at}>{part.bold}</b>))}</span>
    </div>
  );
});

let lastHint = -1;

function nextHint() {
  let at = lastHint;
  while (at === lastHint) at = Math.floor(Math.random() * HINTS);
  return (lastHint = at);
}

function Hello({ hint }: { hint: string }) {
  const mark = useRef<HTMLDivElement>(null);
  const [lit, setLit] = useState(false);
  const tone = useStore(look, (s) => `${s.shown} ${s.chosen.accent}`);
  const pane = usePane();
  const folder = useStore(pane.desk, (s) => Boolean(s.root));
  const nth = useMemo(nextHint, [hint]);

  useEffect(() => {
    if (!mark.current) return;
    const canvas = Object.assign(document.createElement("canvas"), { ariaHidden: "true" });
    const stop = grain(canvas, mark.current);
    if (!stop) return;
    mark.current.prepend(canvas);
    setLit(true);
    return () => {
      stop();
      canvas.remove();
    };
  }, [tone]);

  return (
    <div className="hello">
      <div className="hello-mark" ref={mark} data-grain={lit ? "on" : undefined}>
        <span className="hello-word">sens AI</span>
      </div>
      <p className="hello-hint">{folder ? t.hint(nth) : t.noFolder}</p>
    </div>
  );
}
