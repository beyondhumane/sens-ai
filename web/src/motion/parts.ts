export interface Rect {
  top: number;
  left: number;
  width: number;
  height: number;
}

export const one = <T extends Element = HTMLElement>(root: ParentNode, selector: string): T => {
  const found = root.querySelector<T>(selector);
  if (!found) throw new Error(`missing ${selector}`);
  return found;
};

export const all = <T extends Element = HTMLElement>(root: ParentNode, selector: string): T[] => [
  ...root.querySelectorAll<T>(selector),
];

export function localTo(origin: HTMLElement): (element: Element) => Rect {
  const box = origin.getBoundingClientRect();
  const scale = box.width / origin.offsetWidth || 1;
  return (element) => {
    const rect = element.getBoundingClientRect();
    return {
      top: (rect.top - box.top) / scale,
      left: (rect.left - box.left) / scale,
      width: rect.width / scale,
      height: rect.height / scale,
    };
  };
}

export const span = (first: Rect, last: Rect): Rect => ({
  top: first.top,
  left: Math.min(first.left, last.left),
  width: Math.max(first.left + first.width, last.left + last.width) - Math.min(first.left, last.left),
  height: last.top + last.height - first.top,
});

export const token = (name: string): string => getComputedStyle(document.documentElement).getPropertyValue(name).trim();
