import { gsap } from "gsap";
import { FLIGHT, fly, ghostOf } from "./ghost";
import { all, localTo, one, type Rect } from "./parts";
import { stagger } from "./tokens";

const SOFT = 0.35;
const SHUT = "inset(0px 0px 100% 0px)";
const SHOWN = "inset(0px 0px 0% 0px)";
const fade = { duration: 0.3, ease: "control" };

interface Card {
  root: HTMLElement;
  timeline: gsap.core.Timeline;
}

function markIn(crop: HTMLElement, rect: Rect): HTMLElement {
  const mark = document.createElement("span");
  mark.className = "card-mark";
  crop.append(mark);
  gsap.set(mark, { y: rect.top, scaleY: rect.height });
  return mark;
}

const rowOf = (step: HTMLElement): Element => step.firstElementChild ?? step;

function activity(crop: HTMLElement): gsap.core.Timeline {
  const local = localTo(crop);
  const steps = all(crop, "[data-step]");
  const before = one(crop, '[data-step="run-before"]');
  const mark = markIn(crop, local(rowOf(before)));
  const timeline = gsap.timeline({ paused: true });
  timeline.fromTo(steps, { opacity: 0, y: 4 }, { opacity: 1, y: 0, duration: 0.24, ease: "enter", stagger: stagger.steps / 1000 });
  steps.forEach((step, index) => {
    const at = (index * stagger.steps) / 1000;
    const thought = step.classList.contains("thought");
    timeline
      .set(step, { attr: thought ? { "data-live": "true" } : { "data-state": "running" } }, at)
      .set(step, { attr: thought ? { "data-live": "false" } : { "data-state": "done" } }, at + 0.26);
  });
  return timeline
    .to(steps.filter((step) => step !== before), { opacity: SOFT, ...fade }, "+=0.4")
    .fromTo(mark, { autoAlpha: 0 }, { autoAlpha: 1, ...fade }, "<");
}

const unfold = (id: string) => (crop: HTMLElement): gsap.core.Timeline => {
  const local = localTo(crop);
  const before = one(crop, `[data-step="${id}"]`);
  const body = one(before, '[data-part="step-body"]');
  const next = before.nextElementSibling;
  const row = local(rowOf(before));
  const height = local(body).height;
  const mark = markIn(crop, row);
  gsap.set(body, { clipPath: SHUT });
  if (next) gsap.set(next, { y: -height });
  const timeline = gsap.timeline({ paused: true, defaults: { duration: 0.36, ease: "move" } });
  timeline
    .fromTo(body, { clipPath: SHUT }, { clipPath: SHOWN }, 0.2)
    .fromTo(mark, { scaleY: row.height }, { scaleY: row.height + height }, 0.2)
    .fromTo(all(before, '[data-part="output-line"]'), { opacity: 0 }, { opacity: 1, duration: 0.2, ease: "enter", stagger: 0.04 }, 0.32);
  if (next) timeline.fromTo(next, { y: -height }, { y: 0 }, 0.2);
  return timeline;
};

function relation(crop: HTMLElement, quiet: boolean): gsap.core.Timeline {
  const local = localTo(crop);
  const edit = one(crop, '[data-step="edit-header"]');
  const target = one(edit, '[data-part="target"]');
  const header = one(crop, '[data-change="header"]');
  const head = one(header, '[data-part="change-head"]');
  const name = one(header, '[data-part="change-name"]');
  const diff = all(header, '[data-part="diff-row"]');
  const from = local(target);
  const to = local(name);
  markIn(crop, local(rowOf(edit)));
  const timeline = gsap.timeline({ paused: true });
  if (quiet) {
    return timeline.set([target, head], { attr: { "data-landed": "true" } });
  }
  const ghost = ghostOf(crop, target.textContent, { x: from.left, y: from.top });
  gsap.set(diff, { opacity: 0 });
  return fly(timeline, ghost, { x: to.left, y: to.top }, head, 0.2)
    .fromTo(diff, { opacity: 0, y: 3 }, { opacity: 1, y: 0, duration: 0.22, ease: "enter", stagger: 0.03 }, 0.2 + FLIGHT);
}

const builders: Record<string, (crop: HTMLElement, quiet: boolean) => gsap.core.Timeline> = {
  "card-2": activity,
  "card-3": unfold("run-before"),
  "card-4": unfold("sens-stop"),
  "card-5": relation,
};

export function cards(root: HTMLElement, options: { quiet: boolean }): () => void {
  const list: Card[] = all(root, "[data-card]").flatMap((card) => {
    const crop = card.querySelector<HTMLElement>("[data-crop]");
    const build = crop ? builders[crop.dataset.crop ?? ""] : undefined;
    return crop && build ? [{ root: card, timeline: build(crop, options.quiet) }] : [];
  });

  if (options.quiet) {
    for (const card of list) card.timeline.progress(1);
    return () => {
      for (const card of list) card.timeline.progress(0).kill();
    };
  }

  const seen = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        list.find((card) => card.root === entry.target)?.timeline.restart();
        seen.unobserve(entry.target);
      }
    },
    { threshold: 0.25 },
  );

  const replays = list.map((card) => {
    seen.observe(card.root);
    const button = card.root.querySelector<HTMLButtonElement>("[data-replay]");
    const replay = () => card.timeline.restart();
    button?.addEventListener("click", replay);
    return () => button?.removeEventListener("click", replay);
  });

  return () => {
    seen.disconnect();
    for (const remove of replays) remove();
    for (const card of list) card.timeline.progress(0).kill();
    for (const extra of all(root, ".card-mark, [data-crop] > .ghost")) extra.remove();
  };
}
