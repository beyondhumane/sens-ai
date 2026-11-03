import { useEffect, useState, type RefObject } from "react";

interface Sight {
  root?: () => Element | null;
  margin?: string;
  now?: boolean;
}

export function useSeen(target: RefObject<Element | null>, { root, margin, now = false }: Sight = {}) {
  const [seen, setSeen] = useState(false);
  useEffect(() => {
    if (seen || now || !target.current) return;
    if (typeof IntersectionObserver === "undefined") return setSeen(true);
    const sight = new IntersectionObserver(
      ([entry]) => {
        if (!entry?.isIntersecting) return;
        sight.disconnect();
        setSeen(true);
      },
      { root: root?.() ?? null, rootMargin: margin },
    );
    sight.observe(target.current);
    return () => sight.disconnect();
  }, [seen, now]);
  return seen || now;
}
