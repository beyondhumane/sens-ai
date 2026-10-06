import { gsap } from "gsap";
import { CustomEase } from "gsap/CustomEase";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { cards } from "./cards";
import { scene } from "./scene";
import { theme } from "./theme";
import { bezierOf, curves, type Curve } from "./tokens";

const DESKTOP = "(min-width: 1024px) and (min-height: 640px)";

function menus(): void {
  const all = [...document.querySelectorAll<HTMLDetailsElement>("[data-menu]")];
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    for (const details of all.filter((candidate) => candidate.open)) {
      details.open = false;
      details.querySelector("summary")?.focus();
    }
  });
  document.addEventListener("click", (event) => {
    for (const details of all) if (details.open && !details.contains(event.target as Node)) details.open = false;
  });
  for (const details of all) {
    details.addEventListener("click", (event) => {
      if ((event.target as Element).closest("a")) details.open = false;
    });
  }
}

export function boot(): void {
  gsap.registerPlugin(ScrollTrigger, CustomEase);
  for (const curve of Object.keys(curves) as Curve[]) {
    if (curve !== "linked") CustomEase.create(curve, bezierOf(curve));
  }

  const root = document.documentElement;
  const quiet = root.dataset.motion === "reduce";
  const sceneRoot = document.querySelector<HTMLElement>("[data-scene]");

  menus();
  theme({ quiet });

  if (sceneRoot) {
    gsap.matchMedia().add({ desktop: DESKTOP, narrow: `not all and ${DESKTOP}` }, (context) =>
      context.conditions?.desktop ? scene(sceneRoot, { quiet }) : cards(sceneRoot, { quiet }),
    );
  }

  root.dataset.ready = "";
}
