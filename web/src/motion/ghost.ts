import { gsap } from "gsap";

interface Point {
  x: number;
  y: number;
}

export const FLIGHT = 0.7;

export function ghostOf(host: HTMLElement, label: string | null, takeoff: Point): HTMLElement {
  const ghost = document.createElement("span");
  ghost.className = "ghost";
  ghost.textContent = label;
  host.append(ghost);
  gsap.set(ghost, { ...takeoff, autoAlpha: 0 });
  return ghost;
}

export function fly(timeline: gsap.core.Timeline, ghost: HTMLElement, landing: Point, target: Element, at: gsap.Position): gsap.core.Timeline {
  return timeline
    .to(ghost, { autoAlpha: 1, duration: 0.12, ease: "control" }, at)
    .to(ghost, { ...landing, duration: FLIGHT, ease: "move" }, "<")
    .set(target, { attr: { "data-landed": "true" } }, ">")
    .to(ghost, { autoAlpha: 0, duration: 0.2, ease: "control" }, "<");
}
