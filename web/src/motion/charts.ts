import { all, one } from "./parts";

interface Point {
  x: number;
  title: string;
  rows: { label: string; tone: string; value: string }[];
}

function tipOf(point: Point): DocumentFragment {
  const tip = document.createDocumentFragment();
  const title = document.createElement("p");
  title.className = "chart-tip-title";
  title.textContent = point.title;
  tip.append(title);
  for (const row of point.rows) {
    const line = document.createElement("p");
    line.dataset.tone = row.tone;
    const key = document.createElement("span");
    key.className = "chart-key";
    const value = document.createElement("b");
    value.textContent = row.value;
    line.append(key, `${row.label} `, value);
    tip.append(line);
  }
  return tip;
}

function follow(figure: HTMLElement): void {
  const points = JSON.parse(figure.dataset.chart ?? "[]") as Point[];
  const plot = one(figure, ".chart-plot");
  const svg = one<SVGSVGElement>(figure, "svg");
  const cross = one<SVGLineElement>(figure, ".chart-cross");
  const tip = one(figure, ".chart-tip");
  let shown = -1;

  const show = (at: number) => {
    const point = points[at];
    if (!point) return;
    shown = at;
    cross.setAttribute("x1", String(point.x));
    cross.setAttribute("x2", String(point.x));
    cross.classList.add("on");
    tip.replaceChildren(tipOf(point));
    tip.hidden = false;
    const ratio = point.x / svg.viewBox.baseVal.width;
    tip.style.left = `${ratio * svg.clientWidth}px`;
    tip.dataset.side = ratio > 0.55 ? "left" : "right";
  };

  const hide = () => {
    shown = -1;
    cross.classList.remove("on");
    tip.hidden = true;
  };

  const nearest = (clientX: number) => {
    const box = svg.getBoundingClientRect();
    const x = ((clientX - box.left) / box.width) * svg.viewBox.baseVal.width;
    return points.reduce((best, point, at) => (Math.abs(point.x - x) < Math.abs(points[best].x - x) ? at : best), 0);
  };

  plot.addEventListener("pointermove", (event) => show(nearest(event.clientX)));
  plot.addEventListener("pointerleave", hide);
  plot.addEventListener("focus", () => show(points.length - 1));
  plot.addEventListener("blur", hide);
  plot.addEventListener("keydown", (event) => {
    const step = { ArrowRight: 1, ArrowLeft: -1 }[event.key];
    if (!step) return;
    event.preventDefault();
    show(Math.min(points.length - 1, Math.max(0, (shown < 0 ? points.length - 1 : shown) + step)));
  });
}

export function charts(): void {
  for (const figure of all(document, "[data-chart]")) follow(figure);
}
