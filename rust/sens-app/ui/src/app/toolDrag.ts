import type { PointerEvent as ReactPointerEvent } from "react";
import { createStore } from "zustand/vanilla";
import { lift } from "../shared/lift";
import { closeTab, moveTool, pairTool, shell, visibleIn, type Dock, type Tool } from "./shell";

export type Drop = { dock: Dock | "shelf"; before: Tool | null; pair?: boolean };

export type Surface = "window" | "plan";

export const toolDrag = createStore(() => ({
  tool: null as Tool | null,
  on: "window" as Surface,
  target: null as Drop | null,
  x: 0,
  y: 0,
}));

const EDGE = 0.24;
const FLOOR = 0.62;

const sameDrop = (one: Drop | null, two: Drop | null) => one?.dock === two?.dock && one?.before === two?.before && one?.pair === two?.pair;

function beforeIn(strip: Element, x: number, y: number, attribute: string) {
  const tabs = [...strip.querySelectorAll<HTMLElement>(`[${attribute}]`)];
  const next = tabs.find((tab) => {
    const box = tab.getBoundingClientRect();
    return y < box.top || (y <= box.bottom && x < box.left + box.width / 2);
  });
  return (next?.getAttribute(attribute) as Tool | undefined) ?? null;
}

export function pairHalf(dock: Dock, box: DOMRect) {
  return dock === "bottom" ? { left: box.left + box.width / 2, top: box.top, width: box.width / 2, height: box.height } : { left: box.left, top: box.top + box.height / 2, width: box.width, height: box.height / 2 };
}

function dropOnDock(tool: Tool, x: number, y: number): Drop | null {
  const aside = document.elementFromPoint(x, y)?.closest<HTMLElement>(".dock[data-dock]");
  const dock = aside?.dataset.dock as Dock | undefined;
  if (!aside || !dock || !visibleIn(shell.getState(), dock)) return null;
  const half = pairHalf(dock, aside.getBoundingClientRect());
  if (x < half.left || y < half.top) return null;
  const { shown, paired } = shell.getState();
  if (shown[dock] === tool && !paired[dock]) return null;
  return { dock, before: null, pair: true };
}

function dropOnWindow(x: number, y: number, tool: Tool): Drop | null {
  const strip = document.elementFromPoint(x, y)?.closest<HTMLElement>("[data-tab-strip]");
  if (strip) return { dock: strip.dataset.tabStrip as Dock, before: beforeIn(strip, x, y, "data-tab") };
  const aimed = dropOnDock(tool, x, y);
  if (aimed) return aimed;
  const box = document.getElementById("body")?.getBoundingClientRect();
  if (!box || !box.width || !box.height) return null;
  const across = (x - box.left) / box.width;
  const down = (y - box.top) / box.height;
  if (across < 0 || across > 1 || down < 0 || down > 1) return null;
  if (across < EDGE) return { dock: "start", before: null };
  if (across > 1 - EDGE) return { dock: "end", before: null };
  if (down > FLOOR) return { dock: "bottom", before: null };
  return null;
}

function dropOnPlan(x: number, y: number): Drop | null {
  const slot = document.elementFromPoint(x, y)?.closest<HTMLElement>("[data-plan-dock]");
  if (!slot) return null;
  return { dock: slot.dataset.planDock as Drop["dock"], before: beforeIn(slot, x, y, "data-plan-tool") };
}

function land(tool: Tool, { dock, before, pair }: Drop) {
  if (dock === "shelf") return closeTab(tool);
  if (pair) return pairTool(tool, dock);
  if (before !== tool) moveTool(tool, dock, before);
}

const AIMS: Record<Surface, (x: number, y: number, tool: Tool) => Drop | null> = { window: dropOnWindow, plan: dropOnPlan };

export function liftTool(event: ReactPointerEvent<HTMLElement>, tool: Tool, on: Surface) {
  const aim = AIMS[on];
  lift(event, {
    kind: "tool",
    lifted: () => toolDrag.setState({ tool, on, target: null }),
    moved: (x, y) => {
      const target = aim(x, y, tool);
      toolDrag.setState(sameDrop(target, toolDrag.getState().target) ? { x, y } : { x, y, target });
    },
    ended: (commit) => {
      const { target } = toolDrag.getState();
      toolDrag.setState({ tool: null, target: null });
      if (commit && target) land(tool, target);
    },
  });
}

const KEYED: Record<string, Dock> = { ArrowLeft: "start", ArrowRight: "end", ArrowDown: "bottom" };

export function moveByKey(event: { key: string; altKey: boolean; preventDefault: () => void }, tool: Tool, needsAlt: boolean | null = true) {
  const dock = KEYED[event.key];
  if (!dock || (needsAlt !== null && event.altKey !== needsAlt)) return false;
  event.preventDefault();
  moveTool(tool, dock);
  return true;
}
