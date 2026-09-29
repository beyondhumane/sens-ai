import {
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  type CSSProperties,
  type ClipboardEvent,
  type KeyboardEvent,
  type PointerEvent as HeldPointer,
  type RefObject,
} from "react";
import { useStore } from "zustand";
import { commands } from "../ipc/commands";
import { Flow, Live } from "../features/chat/Reply";
import { halt } from "../features/chat/store";
import type { Notice, Reply } from "../features/chat/turns";
import { useDictation } from "../features/composer/dictation";
import { canSend, pasteText, takeFiles, tooLong, writeMessage } from "../features/composer/store";
import { shared } from "../shared/copy";
import { seconds } from "../shared/format.js";
import { Icon } from "../shared/Icon";
import { ICONS } from "../shared/icons.js";
import { Mark } from "../shared/Mark";
import { Chips } from "./Chips";
import { t } from "./copy";
import { Picker, SessionChip } from "./Picker";
import { Seam } from "./Seam";
import {
  ask,
  bar,
  cycle,
  handOver,
  hear,
  hide,
  lastQuestion,
  lastReply,
  own,
  phaseOf,
  pin,
  resumeLast,
  spentOf,
  takeShot,
  toggleChoosing,
} from "./store";

const MARGIN = 24;
const DRAG_FROM = 4;
const FIELD_MOST = 120;
const NEAR_BOTTOM = 48;

export function Bar() {
  const float = useRef<HTMLDivElement>(null);
  const turns = useStore(own.chat, (s) => s.turns);
  const busy = useStore(own.chat, (s) => s.busy);
  const text = useStore(own.desk, (s) => s.text);
  const pinned = useStore(bar, (s) => s.pinned);
  const opened = useStore(bar, (s) => s.opened);
  const leaving = useStore(bar, (s) => s.leaving);
  const listening = useStore(bar, (s) => s.listening);
  const level = useStore(bar, (s) => s.level);
  const phase = listening ? "listening" : phaseOf(turns, busy, text);
  const reply = lastReply(turns);
  useFit(float);
  useKeys();

  return (
    <div
      className="float"
      ref={float}
      data-state={phase}
      data-motion={leaving ? "out" : opened % 2 ? "in" : "again"}
      data-pinned={pinned ? "true" : undefined}
      style={{ "--level": level } as CSSProperties}
    >
      <Row />
      <Picker />
      <Chips />
      <Seam />
      <Answer reply={reply} />
      {reply?.closed && !busy && <Footer reply={reply} />}
    </div>
  );
}

const growing = (answer: HTMLElement | null) => (answer ? Math.max(Number(answer.dataset.target ?? 0) - answer.getBoundingClientRect().height, 0) : 0);

function useFit(float: RefObject<HTMLDivElement | null>) {
  useLayoutEffect(() => {
    const box = float.current;
    if (!box) return;
    let asked = 0;
    const fit = () => {
      const height = Math.ceil(box.getBoundingClientRect().height + growing(box.querySelector<HTMLElement>(".bar-answer"))) + MARGIN * 2;
      if (height === asked) return;
      asked = height;
      void commands.barFit(height).catch(() => {});
    };
    const watcher = new ResizeObserver(fit);
    watcher.observe(box);
    fit();
    return () => watcher.disconnect();
  }, []);
}

function useGrowth(box: RefObject<HTMLDivElement | null>, inner: RefObject<HTMLDivElement | null>, shown: boolean) {
  useLayoutEffect(() => {
    const outer = box.current;
    const content = inner.current;
    if (!outer || !content) return;
    const style = getComputedStyle(outer);
    const most = parseFloat(style.maxHeight) || Infinity;
    const padding = parseFloat(style.paddingTop) + parseFloat(style.paddingBottom);
    if (!outer.style.height) {
      outer.style.height = "0px";
      outer.getBoundingClientRect();
    }
    const grow = () => {
      const target = Math.min(content.getBoundingClientRect().height + padding, most);
      outer.dataset.target = String(target);
      outer.dataset.growing = target > outer.getBoundingClientRect().height ? "true" : "false";
      outer.style.height = `${target}px`;
    };
    const settle = () => (outer.dataset.growing = "false");
    const watcher = new ResizeObserver(grow);
    watcher.observe(content);
    outer.addEventListener("transitionend", settle);
    grow();
    return () => {
      watcher.disconnect();
      outer.removeEventListener("transitionend", settle);
    };
  }, [shown]);
}

function useKeys() {
  useEffect(() => {
    const keys = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        if (bar.getState().choosing) toggleChoosing();
        else hide();
        return;
      }
      if (!event.ctrlKey || event.altKey || event.metaKey) return;
      const key = event.key.toLowerCase();
      if (key === "enter") {
        event.preventDefault();
        void handOver();
      } else if (key === "p" && !event.shiftKey) {
        event.preventDefault();
        pin();
      } else if (key === "s" && event.shiftKey) {
        event.preventDefault();
        void takeShot();
      }
    };
    document.addEventListener("keydown", keys);
    return () => document.removeEventListener("keydown", keys);
  }, []);
}

function follow(held: HeldPointer<HTMLElement>) {
  if (held.button !== 0) return;
  const { clientX: x, clientY: y } = held;
  const moved = (now: PointerEvent) => {
    if (Math.hypot(now.clientX - x, now.clientY - y) < DRAG_FROM) return;
    stop();
    void commands.barDrag().catch(() => {});
  };
  const stop = () => {
    removeEventListener("pointermove", moved);
    removeEventListener("pointerup", stop);
  };
  addEventListener("pointermove", moved);
  addEventListener("pointerup", stop);
}

const recenter = () => void commands.barRecenter().catch(() => {});

function useGrow(field: RefObject<HTMLTextAreaElement | null>, text: string) {
  useLayoutEffect(() => {
    const box = field.current;
    if (!box) return;
    box.style.height = "auto";
    box.style.height = `${Math.min(box.scrollHeight, FIELD_MOST)}px`;
  }, [text]);
}

function Row() {
  const field = useRef<HTMLTextAreaElement>(null);
  const text = useStore(own.desk, (s) => s.text);
  const root = useStore(own.desk, (s) => s.root);
  const provider = useStore(own.desk, (s) => s.choice.provider);
  const clips = useStore(own.desk, (s) => s.pasted.length + s.attached.length);
  const busy = useStore(own.chat, (s) => s.busy);
  const stopping = useStore(own.chat, (s) => s.stopping);
  const ended = useStore(own.chat, (s) => s.ended);
  const asked = useStore(own.chat, (s) => lastQuestion(s.turns));
  const opened = useStore(bar, (s) => s.opened);
  const pinned = useStore(bar, (s) => s.pinned);
  const called = useStore(bar, (s) => s.called);
  const answered = useRef(called);
  const setText = (next: string) => writeMessage(next, own);
  const dictation = useDictation(own, text, setText, field);
  useGrow(field, text);

  useEffect(() => {
    if (called === answered.current) return;
    answered.current = called;
    if (root) dictation.listen();
  }, [called]);

  useEffect(() => hear(dictation.phase === "listening", dictation.level), [dictation.phase, dictation.level]);

  useEffect(() => {
    field.current?.focus();
  }, [opened]);

  const ready = Boolean(root && provider && (text.trim() || clips));
  const label = busy ? (stopping ? t.stopping : t.stop) : t.send;

  async function go() {
    if (busy || !canSend(text, own)) return;
    dictation.hush();
    await ask();
  }

  function keyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (event.nativeEvent.isComposing || event.ctrlKey || event.altKey || event.metaKey) return;
    if (event.key === "Tab") {
      event.preventDefault();
      cycle(event.shiftKey ? -1 : 1);
    } else if (event.key === "ArrowUp" && !text) {
      void resumeLast().then((resumed) => resumed && field.current?.focus());
    } else if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void go();
    }
  }

  function paste(event: ClipboardEvent<HTMLTextAreaElement>) {
    const files = [...(event.clipboardData?.files || [])];
    const said = event.clipboardData?.getData("text/plain") ?? "";
    if (files.length && !said) {
      event.preventDefault();
      void takeFiles(files, own);
      return;
    }
    const pictures = files.filter((file) => file.type.startsWith("image/"));
    if (pictures.length) void takeFiles(pictures, own);
    if (!tooLong(said)) return;
    event.preventDefault();
    void pasteText(said, own);
  }

  return (
    <div
      className="bar-row"
      onPointerDown={(event) => event.target === event.currentTarget && follow(event)}
      onDoubleClick={(event) => event.target === event.currentTarget && recenter()}
    >
      <span className="bar-handle" title={t.move} onPointerDown={follow} onDoubleClick={recenter}>
        <Mark key={ended} className="bar-mark" size={22} micro />
      </span>
      <textarea
        ref={field}
        id="task"
        className="bar-field"
        rows={1}
        placeholder={asked || t.placeholder}
        data-asked={asked ? "true" : undefined}
        aria-label={t.message}
        autoComplete="off"
        spellCheck={false}
        disabled={!root}
        value={text}
        onChange={(event) => setText(event.target.value)}
        onKeyDown={keyDown}
        onPaste={paste}
      />
      <SessionChip />
      <button type="button" className="round pin" aria-pressed={pinned} title={pinned ? t.pinned : t.pinAction} aria-label={pinned ? t.pinned : t.pinAction} onClick={pin}>
        <Icon svg={ICONS.pin} />
      </button>
      <button
        type="button"
        className="round dictate"
        title={dictation.label}
        aria-label={dictation.label}
        aria-pressed={dictation.phase === "listening"}
        aria-busy={dictation.phase === "finishing"}
        disabled={!root}
        style={{ "--level": dictation.level } as CSSProperties}
        onClick={dictation.toggle}
      >
        <Icon svg={ICONS.mic} />
      </button>
      <button
        type="button"
        className="round send"
        data-ready={ready ? "true" : undefined}
        title={label}
        aria-label={label}
        disabled={busy ? stopping : !ready}
        onClick={() => (busy ? void halt(own) : void go())}
      >
        <Icon svg={busy ? ICONS.stopSquare : ICONS.arrowUp} />
      </button>
    </div>
  );
}

function Answer({ reply }: { reply: Reply | undefined }) {
  const box = useRef<HTMLDivElement>(null);
  const inner = useRef<HTMLDivElement>(null);
  const stick = useRef(true);
  const turns = useStore(own.chat, (s) => s.turns);
  const notices = useMemo(() => turns.slice(turns.findLastIndex((turn) => turn.kind === "you") + 1).filter((turn): turn is Notice => turn.kind === "notice"), [turns]);
  const shown = Boolean(reply) || notices.length > 0;
  useGrowth(box, inner, shown);

  useEffect(() => {
    const follow = new ResizeObserver(() => {
      if (stick.current && box.current) box.current.scrollTop = box.current.scrollHeight;
    });
    if (inner.current) follow.observe(inner.current);
    return () => follow.disconnect();
  }, [shown]);

  if (!shown) return null;
  return (
    <div
      className="bar-answer"
      ref={box}
      onScroll={() => {
        const shown = box.current!;
        stick.current = shown.scrollHeight - shown.scrollTop - shown.clientHeight < NEAR_BOTTOM;
      }}
    >
      <div className="bar-answer-inner" ref={inner}>
        {reply && <Flow turn={reply} />}
        {reply?.working && <Live said={reply.working} began={reply.began} />}
        {notices.map((notice) => (
          <p key={notice.key} className={notice.tone ? `bar-notice ${notice.tone}` : "bar-notice"}>
            {notice.parts.map((part) => (typeof part === "string" ? part : part.bold)).join("")}
          </p>
        ))}
      </div>
    </div>
  );
}

function Footer({ reply }: { reply: Reply }) {
  const pinned = useStore(bar, (s) => s.pinned);
  const { reads, millis } = spentOf(reply);
  const meta = [reads ? t.reads(reads) : "", millis ? seconds(millis) : ""].filter(Boolean).join(" · ");
  return (
    <footer className="bar-foot">
      <div className="keys">
        <span>
          <kbd>{`${shared.ctrl} ⏎`}</kbd>
          {t.openInSens}
        </span>
        <span>
          <kbd>{`${shared.ctrl} P`}</kbd>
          {pinned ? t.unpin : t.pin}
        </span>
        <span>
          <kbd>Esc</kbd>
          {t.close}
        </span>
      </div>
      {meta && <span className="meta">{meta}</span>}
    </footer>
  );
}
