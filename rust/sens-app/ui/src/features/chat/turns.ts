import type { Answers, Asking, ChatEvent, Finding, Finished, Link, Suggested, ToolDetail, ToolInput } from "../../ipc/types";
import { SILENT } from "./looks";


export interface Said {
  kind: "said";
  key: number;
  text: string;
  streamed: boolean;
  settled: boolean;
  done: boolean;
}

export interface Thought {
  kind: "thought";
  key: number;
  text: string;
  settled: boolean;
  done: boolean;
  began: number | null;
  ended: number | null;
}

export interface Step {
  kind: "step";
  key: number;
  id: string;
  name: string;
  input: ToolInput;
  state: "running" | "done" | "failed" | "stopped";
  output: string;
  detail: ToolDetail | null;
  links: Link[];
  began: number | null;
  ended: number | null;
}

export interface Ask {
  kind: "ask";
  key: number;
  event: Asking;
  active: boolean;
  state: "" | "waiting" | "allowed" | "refused" | "expired";
  answers: Answers | null;
}

export interface Fault {
  kind: "fault";
  key: number;
  text: string;
}

export interface Foot {
  kind: "foot";
  key: number;
  millis: number;
  tokens: number;
  stopped: boolean;
}

export interface Compacted {
  kind: "compacted";
  key: number;
  before: number;
  auto: boolean;
}

export interface Sens {
  kind: "sens";
  key: number;
  stage: string;
  findings: Finding[];
  suggestions: Suggested[];
}

export interface Held {
  kind: "held";
  key: number;
  findings: Finding[];
}

export type Part = Said | Thought | Step | Ask | Fault | Foot | Compacted | Sens | Held;

export interface Reply {
  kind: "reply";
  key: number;
  who: string;
  parts: Part[];
  open: number | null;
  working: string;
  began: number;
  closed: boolean;
}

export type Picture = string | Promise<string>;

export interface You {
  kind: "you";
  key: number;
  text: string;
  files: string[];
  pictures: Picture[];
}

export type Piece = string | { bold: string };

export interface Notice {
  kind: "notice";
  key: number;
  parts: Piece[];
  tone: "" | "warn";
}

export type Turn = You | Reply | Notice;

let keys = 0;
export const nextKey = () => ++keys;

export const CLOSING = new Set(["finished", "failed"]);
export const SENS_STOPPED = "Sens stopped this";

export const opening = (who = ""): Reply => ({
  kind: "reply",
  key: nextKey(),
  who,
  parts: [],
  open: null,
  working: "",
  began: performance.now(),
  closed: false,
});

const said = (text: string, streamed: boolean): Said => ({ kind: "said", key: nextKey(), text, streamed, settled: !streamed, done: !streamed });
const thought = (text: string, settled: boolean, began: number | null): Thought => ({ kind: "thought", key: nextKey(), text, settled, done: settled, began, ended: null });

const clock = (live: boolean) => (live ? performance.now() : null);

const swap = <Kind extends Part>(reply: Reply, key: number, change: (part: Kind) => Kind): Reply => ({
  ...reply,
  parts: reply.parts.map((part) => (part.key === key ? change(part as Kind) : part)),
});

const ending = (part: Thought, now: number | null): Thought => (part.began === null || part.ended !== null || now === null ? part : { ...part, ended: now });

function leave(reply: Reply, now: number | null): Reply {
  const open = reply.parts.findLast((part) => part.key === reply.open);
  if (open?.kind !== "thought" || ending(open, now) === open) return reply;
  return swap<Thought>(reply, open.key, (part) => ending(part, now));
}

function delta(reply: Reply, thinking: boolean, text: string, now: number | null): Reply {
  const kind = thinking ? "thought" : "said";
  const open = reply.parts.findLast((part) => part.key === reply.open);
  if (open?.kind === kind) return swap<Said | Thought>(reply, open.key, (part) => ({ ...part, text: part.text + text }));
  const left = leave(reply, now);
  const fresh = thinking ? thought(text, false, now) : said(text, true);
  return { ...left, parts: [...left.parts, fresh], open: fresh.key };
}

function settle(reply: Reply, thinking: boolean, text: string, now: number | null): Reply {
  const kind = thinking ? "thought" : "said";
  const waiting = reply.parts.find((part): part is Said | Thought => part.kind === kind && !part.settled);
  if (!waiting) return { ...reply, parts: [...reply.parts, thinking ? thought(text, true, null) : said(text, false)] };
  const settled = swap<Said | Thought>(reply, waiting.key, (part) =>
    part.kind === "thought" ? ending({ ...part, text, settled: true, done: true }, now) : { ...part, text, settled: true },
  );
  return reply.open === waiting.key ? { ...settled, open: null } : settled;
}

function close(reply: Reply, now: number | null): Reply {
  const left = leave(reply, now);
  return {
    ...left,
    open: null,
    working: "",
    closed: true,
    parts: left.parts.map((part): Part => {
      if (part.kind === "said" || part.kind === "thought") return part.done ? part : { ...part, done: true };
      if (part.kind === "ask" && part.state !== "allowed" && part.state !== "refused") return { ...part, active: false, state: "expired" };
      if (part.kind === "step" && part.state === "running") return { ...part, state: "stopped" };
      return part;
    }),
  };
}

const fault = (text: string): Fault => ({ kind: "fault", key: nextKey(), text });

const footOf = ({ millis, tokensOut, stopped }: Finished): Foot[] =>
  millis || tokensOut || stopped ? [{ kind: "foot", key: nextKey(), millis, tokens: tokensOut, stopped }] : [];

export function heard(reply: Reply, event: ChatEvent, live: boolean): Reply {
  const now = clock(live);
  switch (event.kind) {
    case "delta":
      return delta(reply, event.thinking, event.text, now);
    case "said":
      return settle(reply, false, event.text, now);
    case "thought":
      return settle(reply, true, event.text, now);
    case "tool": {
      const left = leave(reply, now);
      if (SILENT.has(event.name)) return { ...left, open: null };
      return {
        ...left,
        open: null,
        parts: [
          ...left.parts,
          { kind: "step", key: nextKey(), id: event.id, name: event.name, input: event.input || {}, state: "running", output: "", detail: null, links: [], began: now, ended: null },
        ],
      };
    }
    case "toolDone": {
      const step = reply.parts.find((part): part is Step => part.kind === "step" && part.id === event.id);
      if (!step) return reply;
      const stopped = event.error && event.output.includes(SENS_STOPPED);
      return swap<Step>(reply, step.key, (part) => ({ ...part, state: stopped ? "stopped" : event.error ? "failed" : "done", output: event.output, detail: event.detail, ended: now }));
    }
    case "consulted": {
      const step = reply.parts.find((part): part is Step => part.kind === "step" && part.id === event.tool);
      if (!step) return reply;
      return swap<Step>(reply, step.key, (part) => {
        const known = new Map(part.links.map((link) => [link.url, link]));
        for (const link of event.links) known.set(link.url, link);
        return { ...part, links: [...known.values()] };
      });
    }
    case "asking": {
      const left = leave(reply, now);
      return {
        ...left,
        open: null,
        parts: [...left.parts, { kind: "ask", key: nextKey(), event, active: live, state: live ? "waiting" : "", answers: null }],
      };
    }
    case "answered": {
      const ask = reply.parts.find((part): part is Ask => part.kind === "ask" && part.event.request === event.request);
      if (!ask) return reply;
      return swap<Ask>(reply, ask.key, (part) => ({ ...part, active: false, state: event.allowed ? "allowed" : "refused", answers: event.answers }));
    }
    case "compacted":
      return { ...reply, parts: [...reply.parts, { kind: "compacted", key: nextKey(), before: event.before, auto: event.auto }] };
    case "canon": {
      if (event.stage === "reviewed" && event.findings.length === 0) return reply;
      const left = leave(reply, now);
      return { ...left, open: null, parts: [...left.parts, { kind: "sens", key: nextKey(), stage: event.stage, findings: event.findings, suggestions: event.suggestions }] };
    }
    case "held": {
      const left = leave(reply, now);
      return { ...left, open: null, parts: [...left.parts, { kind: "held", key: nextKey(), findings: event.findings }] };
    }
    case "finished": {
      const ended = close(reply, now);
      return { ...ended, parts: [...ended.parts, ...(event.error ? [fault(event.error)] : []), ...footOf(event)] };
    }
    case "failed": {
      const ended = close(reply, now);
      return { ...ended, parts: [...ended.parts, fault(event.reason)] };
    }
    default:
      return reply;
  }
}

export const answered = (reply: Reply, request: string, allowed: boolean, answers: Answers | null): Reply =>
  heard(reply, { kind: "answered", request, allowed, answers }, false);
