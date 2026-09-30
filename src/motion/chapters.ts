import { ScrollTrigger } from "gsap/ScrollTrigger";
import { all } from "./parts";

const LINE = "45%";

export function watchChapters(root: ParentNode, onChange: (chapter: number) => void): () => void {
  const items = all(root, "[data-chapter]");
  const links = all<HTMLAnchorElement>(root, "[data-to]");
  let current = -1;

  const select = (chapter: number) => {
    if (chapter === current) return;
    current = chapter;
    for (const item of items) item.toggleAttribute("data-active", Number(item.dataset.chapter) === chapter);
    for (const link of links) {
      if (Number(link.dataset.to) === chapter) link.setAttribute("aria-current", "step");
      else link.removeAttribute("aria-current");
    }
    onChange(chapter);
  };

  const triggers = items.map((item, index) =>
    ScrollTrigger.create({
      trigger: item,
      start: `top ${LINE}`,
      end: `bottom ${LINE}`,
      onToggle: (self) => {
        if (self.isActive) select(Number(item.dataset.chapter));
      },
      onLeaveBack: () => {
        if (index === 0) select(0);
      },
    }),
  );

  if (current === -1) select(0);

  return () => {
    for (const trigger of triggers) trigger.kill();
    for (const item of items) item.removeAttribute("data-active");
    for (const link of links) link.removeAttribute("aria-current");
  };
}

export function watchStuck(element: HTMLElement, top: number): () => void {
  const observer = new IntersectionObserver(
    ([entry]) => element.toggleAttribute("data-stuck", entry.intersectionRatio < 1 && entry.boundingClientRect.top < top + 1),
    { rootMargin: `-${top + 1}px 0px 0px 0px`, threshold: 1 },
  );
  observer.observe(element);
  return () => {
    observer.disconnect();
    element.removeAttribute("data-stuck");
  };
}
