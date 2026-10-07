import type { KeyboardEvent } from "react";

const MOVES: Record<string, (at: number, last: number) => number> = {
  ArrowRight: (at) => at + 1,
  ArrowLeft: (at) => at - 1,
  Home: () => 0,
  End: (_, last) => last,
};

const TAB = '[role="tab"]';

export function roveTabs<T>(event: KeyboardEvent<HTMLElement>, ids: readonly T[], at: T, pick: (id: T) => void) {
  const move = MOVES[event.key];
  if (!move || event.altKey || event.ctrlKey || event.metaKey || !(event.target as HTMLElement).matches(TAB)) return;
  event.preventDefault();
  const to = (move(ids.indexOf(at), ids.length - 1) + ids.length) % ids.length;
  pick(ids[to]);
  event.currentTarget.querySelectorAll<HTMLElement>(TAB)[to]?.focus();
}
