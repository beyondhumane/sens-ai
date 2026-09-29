import type { BarProject, SessionSummary } from "../ipc/types";
import { stem } from "../shared/format.js";

export const MOST_SESSIONS = 8;

export type Sessions = Record<string, SessionSummary[]>;

export type Choice =
  | { kind: "new"; root: string; name: string }
  | { kind: "session"; root: string; name: string; id: string; title: string }
  | { kind: "project"; root: string; name: string };

const lower = (text: string) => text.toLocaleLowerCase();

const nameOf = (projects: BarProject[], root: string) => projects.find((one) => one.root === root)?.name ?? stem(root);

const recent = (sessions: Sessions, root: string) =>
  (sessions[root] ?? []).filter((one) => !one.archived).sort((one, other) => other.startedAt - one.startedAt);

const sessionIn = (root: string, name: string) => (summary: SessionSummary): Choice => ({ kind: "session", root, name, id: summary.id, title: summary.title });

const projectOf = (one: BarProject): Choice => ({ kind: "project", root: one.root, name: one.name });

export function choicesOf(projects: BarProject[], sessions: Sessions, root: string, filter: string): Choice[] {
  const wanted = lower(filter.trim());
  if (!wanted) {
    const here: Choice[] = root ? [{ kind: "new", root, name: nameOf(projects, root) }, ...recent(sessions, root).slice(0, MOST_SESSIONS).map(sessionIn(root, nameOf(projects, root)))] : [];
    return [...here, ...projects.filter((one) => one.root !== root).map(projectOf)];
  }
  const ordered = [...projects].sort((one, other) => Number(other.root === root) - Number(one.root === root));
  const said = ordered.flatMap((one) => recent(sessions, one.root).map(sessionIn(one.root, one.name)));
  return [
    ...said.filter((choice) => choice.kind === "session" && lower(choice.title).includes(wanted)),
    ...ordered.filter((one) => lower(one.name).includes(wanted)).map(projectOf),
  ];
}

export function titleOf(sessions: Sessions, root: string, id: string) {
  return id ? (sessions[root] ?? []).find((one) => one.id === id)?.title ?? "" : "";
}

export const keyOf = (choice: Choice) => (choice.kind === "session" ? `session:${choice.id}` : `${choice.kind}:${choice.root}`);
