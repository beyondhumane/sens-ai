import type { RefObject } from "react";
import { keepSize, sizing } from "./shell";

const SIZE_STEP = 16;

const KEYS = {
  x: { ArrowLeft: -1, ArrowRight: 1 },
  y: { ArrowUp: -1, ArrowDown: 1 },
} as Record<"x" | "y", Record<string, number>>;

export function Splitter({
  id,
  label,
  name,
  host,
  pane,
  grow,
  axis = "x",
  className = "",
}: {
  id: string;
  label: string;
  name: string;
  host: RefObject<HTMLElement | null>;
  pane: RefObject<HTMLElement | null>;
  grow: 1 | -1;
  axis?: "x" | "y";
  className?: string;
}) {
  const size = () => {
    const box = pane.current?.getBoundingClientRect();
    return Math.round((axis === "x" ? box?.width : box?.height) ?? 0);
  };
  const set = (px: number) => host.current?.style.setProperty(name, `${Math.round(px)}px`);
  const settle = () => {
    const now = size();
    keepSize(name, now);
    return now;
  };

  return (
    <div
      className={`split ${className}`.trim()}
      id={id}
      role="separator"
      aria-orientation={axis === "x" ? "vertical" : "horizontal"}
      aria-label={label}
      tabIndex={0}
      onFocus={(event) => event.currentTarget.setAttribute("aria-valuenow", String(size()))}
      onDoubleClick={() => {
        host.current?.style.removeProperty(name);
        keepSize(name, 0);
      }}
      onPointerDown={(event) => {
        if (event.button !== 0) return;
        event.preventDefault();
        const handle = event.currentTarget;
        const from = axis === "x" ? event.clientX : event.clientY;
        const start = size();
        handle.setPointerCapture(event.pointerId);
        handle.dataset.dragging = "true";
        sizing(true, axis);
        const move = (moved: PointerEvent) => set(start + ((axis === "x" ? moved.clientX : moved.clientY) - from) * grow);
        const done = () => {
          handle.removeEventListener("pointermove", move);
          handle.removeEventListener("pointerup", done);
          handle.removeEventListener("pointercancel", done);
          delete handle.dataset.dragging;
          handle.setAttribute("aria-valuenow", String(settle()));
          sizing(false);
        };
        handle.addEventListener("pointermove", move);
        handle.addEventListener("pointerup", done);
        handle.addEventListener("pointercancel", done);
      }}
      onKeyDown={(event) => {
        const step = KEYS[axis][event.key];
        if (!step) return;
        event.preventDefault();
        sizing(true, axis);
        set(size() + step * grow * SIZE_STEP * (event.shiftKey ? 4 : 1));
        event.currentTarget.setAttribute("aria-valuenow", String(settle()));
        sizing(false);
      }}
    />
  );
}
