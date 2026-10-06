import { all } from "./parts";

export function contents(): void {
  const links = all<HTMLAnchorElement>(document, "[data-toc]");
  const sections = links.flatMap((link) => {
    const section = document.getElementById(link.dataset.toc ?? "")?.closest("section");
    return section ? [{ link, section }] : [];
  });
  const mark = (current: Element) => {
    for (const { link, section } of sections) {
      if (section === current) link.setAttribute("aria-current", "location");
      else link.removeAttribute("aria-current");
    }
  };
  const seen = new IntersectionObserver(
    (entries) => {
      const shown = entries.filter((entry) => entry.isIntersecting).sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top)[0];
      if (shown) mark(shown.target);
    },
    { rootMargin: "-30% 0px -60% 0px" },
  );
  for (const { section } of sections) seen.observe(section);
}
