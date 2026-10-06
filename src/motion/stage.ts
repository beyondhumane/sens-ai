import { gsap } from "gsap";
import { WINDOW } from "./geometry";
import { FLIGHT, fly, ghostOf } from "./ghost";
import { all, localTo, one, span, token, type Rect } from "./parts";
import { durations, stagger } from "./tokens";

const DIM = 0.25;
const SOFT = 0.35;
const JUMP = 1.4;
const inset = (bottom: number) => `inset(0px 0px ${bottom}px 0px)`;

export interface Stage {
  go(chapter: number): void;
  set(chapter: number): void;
  kill(): void;
}

interface Parts {
  product: HTMLElement;
  thread: HTMLElement;
  fade: HTMLElement;
  inner: HTMLElement;
  request: HTMLElement;
  run: HTMLElement;
  runHead: HTMLElement;
  runBody: HTMLElement;
  line: HTMLElement;
  steps: HTMLElement[];
  before: HTMLElement;
  stop: HTMLElement;
  edit: HTMLElement;
  editTarget: HTMLElement;
  said: HTMLElement;
  foot: HTMLElement;
  totals: HTMLElement;
  clean: HTMLElement;
  rows: HTMLElement[];
  header: HTMLElement;
  headerHead: HTMLElement;
  headerName: HTMLElement;
  diff: HTMLElement[];
}

function partsOf(product: HTMLElement): Parts {
  const steps = all(product, "[data-step]");
  const edit = one(product, '[data-step="edit-header"]');
  const header = one(product, '[data-change="header"]');
  return {
    product,
    thread: one(product, '[data-part="thread"]'),
    fade: one(product, '[data-part="fade-top"]'),
    inner: one(product, '[data-part="thread-inner"]'),
    request: one(product, '[data-part="request"]'),
    run: one(product, '[data-part="run"]'),
    runHead: one(product, '[data-part="run-head"]'),
    runBody: one(product, '[data-part="run-body"]'),
    line: one(product, '[data-part="run-line"]'),
    steps,
    before: one(product, '[data-step="run-before"]'),
    stop: one(product, '[data-step="sens-stop"]'),
    edit,
    editTarget: one(edit, '[data-part="target"]'),
    said: one(product, '[data-part="said"]'),
    foot: one(product, '[data-part="foot"]'),
    totals: one(product, '[data-part="totals"]'),
    clean: one(product, '[data-part="clean"]'),
    rows: all(product, "[data-change]"),
    header,
    headerHead: one(header, '[data-part="change-head"]'),
    headerName: one(header, '[data-part="change-name"]'),
    diff: all(header, '[data-part="diff-row"]'),
  };
}

interface Layout {
  request: Rect;
  result: Rect;
  edit: Rect;
  target: Rect;
  name: Rect;
  view: Rect;
  runHeight: number;
  lineHeight: number;
}

interface Unfold {
  step: HTMLElement;
  body: HTMLElement;
  lines: HTMLElement[];
  after: HTMLElement[];
  row: Rect;
  opened: Rect;
  height: number;
}

function layoutOf(parts: Parts): Layout {
  const local = localTo(parts.product);
  return {
    request: local(parts.request),
    result: span(local(parts.runHead), local(parts.foot)),
    edit: local(parts.edit.firstElementChild ?? parts.edit),
    target: local(parts.editTarget),
    name: local(parts.headerName),
    view: local(parts.thread),
    runHeight: local(parts.runBody).height,
    lineHeight: local(parts.line).height,
  };
}

function unfoldOf(parts: Parts, step: HTMLElement): Unfold {
  const local = localTo(parts.product);
  const body = one(step, '[data-part="step-body"]');
  const row = local(step.firstElementChild ?? step);
  const height = local(body).height;
  return {
    step,
    body,
    lines: all(step, '[data-part="output-line"]'),
    after: parts.steps.slice(parts.steps.indexOf(step) + 1),
    row,
    opened: { ...row, height: row.height + height },
    height,
  };
}

const markAt = (rect: Rect, scroll: number, unit: number) => ({
  y: (rect.top - scroll) * unit,
  scaleY: rect.height / WINDOW.height,
});

const scrollFor = (rect: Rect, view: Rect): number => Math.max(0, rect.top + rect.height - (view.top + view.height - 24));

export function createStage(product: HTMLElement, mark: HTMLElement, options: { unit: number; quiet: boolean }): Stage {
  const parts = partsOf(product);
  const layout = layoutOf(parts);
  const run = unfoldOf(parts, parts.before);
  const stop = unfoldOf(parts, parts.stop);
  const { unit, quiet } = options;
  const lime = token("--sens-signal-500");
  const hot = token("--sens-signal-100");
  const { steps } = parts;
  const moved = [parts.said, parts.foot];
  const scrollEdit = scrollFor(layout.edit, layout.view);
  const takeoff = { x: layout.target.left, y: layout.target.top - scrollEdit };
  const ghost = quiet ? null : ghostOf(product, parts.editTarget.textContent, takeoff);
  const open = layout.runHeight;
  const fade = { duration: 0.3, ease: "control" };

  gsap.set(parts.runBody, { visibility: "visible", clipPath: inset(open) });
  for (const unfold of [run, stop]) gsap.set(unfold.body, { visibility: "visible", clipPath: inset(unfold.height) });
  gsap.set(parts.fade, { opacity: 0 });
  gsap.set(parts.clean, { autoAlpha: 0 });
  gsap.set(mark, { y: 0, scaleY: 1, scaleX: 1, backgroundColor: lime });

  const timeline = gsap.timeline({ paused: true, defaults: { duration: durations.scene / 1000, ease: "move" } });

  const unfold = (target: Unfold, at: string) => {
    const scroll = scrollFor(target.opened, layout.view);
    timeline
      .to(target.step, { opacity: 1, ...fade }, at)
      .set(target.step, { attr: { "data-open": "true" } }, at)
      .to(target.body, { clipPath: inset(0), duration: 0.36 }, at)
      .to(parts.runBody, { clipPath: inset(-target.height), duration: 0.36 }, at)
      .to(parts.fade, { opacity: scroll > 0 ? 1 : 0, duration: 0.36 }, at)
      .to(target.after, { y: target.height, duration: 0.36 }, at)
      .to(parts.line, { scaleY: (layout.lineHeight + target.height) / layout.lineHeight, duration: 0.36 }, at)
      .to(moved, { y: open + target.height, duration: 0.36 }, at)
      .set(target.lines, { opacity: 0 }, `${at}+=0.12`)
      .to(target.lines, { opacity: 1, duration: 0.2, ease: "enter", stagger: 0.04 }, `${at}+=0.12`)
      .to(parts.inner, { y: -scroll, duration: 0.36 }, at)
      .to(mark, { ...markAt(target.opened, scroll, unit), duration: 0.36 }, at);
  };

  const fold = (target: Unfold, at: string) =>
    timeline
      .to(target.body, { clipPath: inset(target.height), duration: 0.3 }, at)
      .to(parts.runBody, { clipPath: inset(0), duration: 0.3 }, at)
      .to(target.after, { y: 0, duration: 0.3 }, at)
      .to(parts.line, { scaleY: 1, duration: 0.3 }, at)
      .to(moved, { y: open, duration: 0.3 }, at)
      .set(target.step, { attr: { "data-open": "false" } }, `${at}+=0.3`)
      .to(target.step, { opacity: SOFT, ...fade }, at);

  timeline
    .addLabel("s0")
    .to([parts.run, ...moved], { opacity: DIM, ...fade }, "s0")
    .to([...parts.rows, parts.totals], { autoAlpha: 0, ...fade }, "s0")
    .to(parts.diff, { opacity: 0, duration: 0.01 }, "s0+=0.3")
    .to(parts.clean, { autoAlpha: 1, ...fade }, "s0+=0.2")
    .to(mark, { ...markAt(layout.request, 0, unit), duration: 0.5 }, "s0")
    .addLabel("s1");

  const cascade = 0.25;
  timeline
    .to(parts.run, { opacity: 1, ...fade }, "s1")
    .set(parts.run, { attr: { "data-open": "true" } }, "s1")
    .to(parts.runBody, { clipPath: inset(0), duration: 0.6 }, "s1")
    .to(moved, { y: open, duration: 0.6 }, "s1")
    .set(steps, { opacity: 0, y: 4 }, `s1+=${cascade}`)
    .to(steps, { opacity: 1, y: 0, duration: 0.24, ease: "enter", stagger: stagger.steps / 1000 }, `s1+=${cascade}`);

  steps.forEach((step, index) => {
    const at = cascade + (index * stagger.steps) / 1000;
    const thought = step.classList.contains("thought");
    timeline
      .set(step, { attr: thought ? { "data-live": "true" } : { "data-state": "running" } }, `s1+=${at}`)
      .set(step, { attr: thought ? { "data-live": "false" } : { "data-state": "done" } }, `s1+=${at + 0.26}`);
  });

  const focusAt = cascade + (steps.length * stagger.steps) / 1000 + 0.26 + 0.4;
  timeline
    .to(steps.filter((step) => step !== parts.before), { opacity: SOFT, ...fade }, `s1+=${focusAt}`)
    .to(mark, { ...markAt(run.row, 0, unit), duration: 0.5 }, `s1+=${focusAt}`)
    .addLabel("s2");

  unfold(run, "s2");
  timeline.addLabel("s3");

  fold(run, "s3");
  unfold(stop, "s3+=0.3");
  timeline.addLabel("s4");

  fold(stop, "s4");
  timeline
    .to(parts.fade, { opacity: scrollEdit > 0 ? 1 : 0, duration: 0.3 }, "s4")
    .to(parts.inner, { y: -scrollEdit, duration: 0.3 }, "s4")
    .to(parts.edit, { opacity: 1, ...fade }, "s4")
    .to(mark, { ...markAt(layout.edit, scrollEdit, unit), duration: 0.5 }, "s4")
    .to(parts.clean, { autoAlpha: 0, duration: 0.2, ease: "control" }, "s4+=0.2")
    .to([parts.header, parts.totals], { autoAlpha: 1, ...fade }, "s4+=0.3");

  const land = 0.35 + FLIGHT;
  if (ghost) {
    fly(timeline, ghost, { x: layout.name.left, y: layout.name.top }, parts.headerHead, "s4+=0.35");
  } else {
    timeline.set([parts.editTarget, parts.headerHead], { attr: { "data-landed": "true" } }, "s4+=0.3");
  }

  timeline
    .set(parts.diff, { opacity: 0, y: 3 }, `s4+=${land}`)
    .to(parts.diff, { opacity: 1, y: 0, duration: 0.22, ease: "enter", stagger: 0.03 }, `s4+=${land}`)
    .to(parts.rows.filter((row) => row !== parts.header), { autoAlpha: 1, ...fade }, `s4+=${land + 0.4}`)
    .addLabel("s5");

  timeline
    .set([parts.headerHead, parts.editTarget], { attr: { "data-landed": "false" } }, "s5")
    .to(parts.runBody, { clipPath: inset(open), duration: 0.5 }, "s5")
    .to(parts.fade, { opacity: 0, duration: 0.5 }, "s5")
    .to(moved, { y: 0, duration: 0.5 }, "s5")
    .set(parts.run, { attr: { "data-open": "false" } }, "s5+=0.5")
    .to(parts.inner, { y: 0, duration: 0.5 }, "s5")
    .to(steps, { opacity: 1, duration: 0.01 }, "s5+=0.5")
    .to(moved, { opacity: 1, ...fade }, "s5+=0.2")
    .to(mark, { ...markAt(layout.result, 0, unit), duration: 0.5 }, "s5")
    .to(mark, { scaleX: 1.5, backgroundColor: hot, duration: 0.2, ease: "enter" }, "s5+=0.5")
    .to(mark, { scaleX: 1, backgroundColor: lime, duration: (durations.confirm - 200) / 1000, ease: "control" }, "s5+=0.7")
    .addLabel("s6");

  let travel: gsap.core.Tween | null = null;
  let at = 0;

  const set = (chapter: number) => {
    travel?.kill();
    timeline.seek(`s${chapter}`);
    at = chapter;
  };

  return {
    set,
    go(chapter) {
      if (quiet) {
        set(chapter);
        return;
      }
      const label = `s${chapter}`;
      const distance = Math.abs(timeline.labels[label] - timeline.time());
      travel?.kill();
      travel = timeline.tweenTo(label, { duration: Math.abs(chapter - at) > 1 ? Math.min(JUMP, distance) : distance, ease: "none" });
      at = chapter;
    },
    kill() {
      travel?.kill();
      timeline.progress(0).kill();
      ghost?.remove();
      gsap.set([parts.runBody, run.body, stop.body], { clearProps: "visibility,clipPath" });
      gsap.set(parts.fade, { clearProps: "opacity" });
    },
  };
}
