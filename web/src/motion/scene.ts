import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { watchChapters, watchStuck } from "./chapters";
import { frameOf, NAV, WINDOW, type Frame } from "./geometry";
import { one } from "./parts";
import { createStage, type Stage } from "./stage";
import { waves } from "./waves";

interface Parts {
  scene: HTMLElement;
  back: HTMLElement;
  layer: HTMLElement;
  shot: HTMLElement;
  focus: HTMLElement;
  mark: HTMLElement;
  demo: HTMLElement;
  index: HTMLElement;
  wordmark: HTMLElement;
  product: HTMLElement;
}

interface Reading {
  frame: Frame;
  lift: number;
}

type Place = (reading: Reading) => gsap.TweenVars;

const REFRAME = 1;
const SMOOTHING = 1;
const TOUCH = NAV + 8;

const topShot: Place = ({ frame }) => ({ x: frame.hero.x, y: frame.hero.y, scale: frame.hero.s });
const heroShot: Place = ({ frame, lift }) => ({ x: frame.hero.x, y: frame.hero.y - lift, scale: frame.hero.s });
const stageShot: Place = ({ frame }) => ({ x: frame.stage.x, y: frame.stage.y, scale: frame.stage.s });
const heroAcross: Place = () => ({ x: 0, scaleX: 1 });
const stageAcross: Place = ({ frame }) => ({ x: frame.rail.x - frame.field.x, scaleX: frame.rail.width / frame.field.width });
const topDown: Place = () => ({ y: 0, scaleY: 1 });
const heroDown: Place = ({ lift }) => ({ y: -lift, scaleY: 1 });
const stageDown: Place = ({ frame }) => ({ y: frame.rail.y, scaleY: frame.rail.height / frame.field.height });
const topFocus: Place = (reading) => ({ ...heroAcross(reading), ...topDown(reading) });
const heroFocus: Place = (reading) => ({ ...heroAcross(reading), ...heroDown(reading) });
const stageFocus: Place = (reading) => ({ ...stageAcross(reading), ...stageDown(reading) });

function partsOf(scene: HTMLElement): Parts {
  const hero = one(scene, ".hero");
  return {
    scene,
    back: one(scene, "[data-back]"),
    layer: one(scene, "[data-layer]"),
    shot: one(scene, "[data-shot]"),
    focus: one(scene, "[data-focus]"),
    mark: one(scene, "[data-focus-mark]"),
    demo: one(scene, ".demo"),
    index: one(scene, "[data-index]"),
    wordmark: one(hero, ".wordmark"),
    product: one(scene, "[data-shot] .product"),
  };
}

function measure(parts: Parts): Reading {
  const frame = frameOf({ width: parts.layer.clientWidth, height: window.innerHeight });
  return { frame, lift: Math.max(0, frame.hero.y - TOUCH) };
}

const placementsOf = (frame: Frame): Record<string, string> => ({
  "--shot-x": `${frame.hero.x}px`,
  "--shot-y": `${frame.hero.y}px`,
  "--shot-s": `${frame.hero.s}`,
  "--field-x": `${frame.field.x}px`,
  "--field-h": `${frame.field.height}px`,
});

function paint(parts: Parts, frame: Frame): void {
  for (const [name, value] of Object.entries(placementsOf(frame))) parts.scene.style.setProperty(name, value);
  document.documentElement.style.setProperty("--split-x", `${frame.field.x}px`);
}

function moving(parts: Parts, active: boolean): void {
  parts.layer.classList.toggle("is-moving", active);
  parts.back.classList.toggle("is-moving", active);
}

function staged(parts: Parts, on: boolean): void {
  parts.scene.toggleAttribute("data-staged", on);
}

const lazy = (place: Place, read: () => Reading): gsap.TweenVars =>
  Object.fromEntries(Object.keys(place(read())).map((key) => [key, () => place(read())[key]]));

function scrub(parts: Parts, read: () => Reading): void {
  const travel = () => Math.round(window.innerHeight * REFRAME);
  const climb = Math.max(1, read().lift);
  const timeline = gsap.timeline({
    defaults: { ease: "none" },
    onUpdate: () => staged(parts, timeline.progress() > 0),
    scrollTrigger: {
      start: 0,
      end: () => read().lift + travel(),
      scrub: SMOOTHING,
      invalidateOnRefresh: true,
      onToggle: (self) => moving(parts, self.isActive),
    },
  });
  timeline
    .fromTo(parts.shot, lazy(topShot, read), { ...lazy(heroShot, read), duration: climb }, 0)
    .fromTo(parts.focus, lazy(topFocus, read), { ...lazy(heroFocus, read), duration: climb }, 0)
    .to(parts.shot, { ...lazy(stageShot, read), duration: travel() }, climb)
    .to(parts.focus, { ...lazy(stageFocus, read), duration: travel() }, climb)
    .fromTo(parts.wordmark, { autoAlpha: 1 }, { autoAlpha: 0, duration: travel() * 0.3 }, climb + travel() * 0.3);
}

function jump(parts: Parts, read: () => Reading): void {
  const place = (on: boolean) => {
    const reading = read();
    gsap.set(parts.shot, (on ? stageShot : heroShot)(reading));
    gsap.set(parts.focus, (on ? stageFocus : heroFocus)(reading));
    gsap.set(parts.wordmark, { autoAlpha: on ? 0 : 1 });
    staged(parts, on);
  };
  ScrollTrigger.create({
    start: () => read().lift,
    end: "max",
    onToggle: (self) => place(self.isActive),
    onRefresh: (self) => place(self.progress > 0),
  });
}

export function scene(root: HTMLElement, options: { quiet: boolean }): () => void {
  const parts = partsOf(root);
  let reading = measure(parts);
  paint(parts, reading.frame);

  const read = () => reading;
  if (options.quiet) jump(parts, read);
  else scrub(parts, read);
  const still = options.quiet ? () => {} : waves(parts.wordmark);
  const unstick = watchStuck(parts.index, NAV);

  const build = () =>
    createStage(parts.product, parts.mark, { unit: reading.frame.field.height / WINDOW.height, quiet: options.quiet });
  let chapter = 0;
  let stage: Stage = build();
  const stop = watchChapters(parts.demo, (next) => {
    chapter = next;
    stage.go(next);
  });

  const refresh = () => {
    reading = measure(parts);
    paint(parts, reading.frame);
    stage.kill();
    stage = build();
    stage.set(chapter);
  };
  ScrollTrigger.addEventListener("refreshInit", refresh);

  return () => {
    ScrollTrigger.removeEventListener("refreshInit", refresh);
    stop();
    still();
    unstick();
    stage.kill();
    moving(parts, false);
    staged(parts, false);
    for (const name of Object.keys(placementsOf(reading.frame))) parts.scene.style.removeProperty(name);
    document.documentElement.style.removeProperty("--split-x");
  };
}
