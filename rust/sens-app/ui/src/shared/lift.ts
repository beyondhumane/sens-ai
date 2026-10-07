import type { PointerEvent as ReactPointerEvent } from "react";

const LIFT = 5;

export interface Lifting {
  kind: string;
  lifted: () => void;
  moved: (x: number, y: number) => void;
  ended: (commit: boolean) => void;
}

function swallow(event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
}

export function lift(event: ReactPointerEvent<HTMLElement>, { kind, lifted, moved, ended }: Lifting) {
  if (event.button !== 0) return;
  const start = { x: event.clientX, y: event.clientY };
  let up = false;

  const move = (pointer: PointerEvent) => {
    if (!up) {
      if (Math.hypot(pointer.clientX - start.x, pointer.clientY - start.y) < LIFT) return;
      up = true;
      document.documentElement.dataset.dragging = kind;
      window.addEventListener("click", swallow, true);
      lifted();
    }
    moved(pointer.clientX, pointer.clientY);
  };

  const end = (commit: boolean) => {
    window.removeEventListener("pointermove", move);
    window.removeEventListener("pointerup", drop);
    window.removeEventListener("pointercancel", cancel);
    window.removeEventListener("keydown", key, true);
    window.removeEventListener("blur", cancel);
    if (!up) return;
    delete document.documentElement.dataset.dragging;
    setTimeout(() => window.removeEventListener("click", swallow, true));
    ended(commit);
  };

  const drop = () => end(true);
  const cancel = () => end(false);
  const key = (pressed: KeyboardEvent) => {
    if (pressed.key !== "Escape") return;
    pressed.preventDefault();
    pressed.stopPropagation();
    cancel();
  };

  window.addEventListener("pointermove", move);
  window.addEventListener("pointerup", drop);
  window.addEventListener("pointercancel", cancel);
  window.addEventListener("keydown", key, true);
  window.addEventListener("blur", cancel);
}
