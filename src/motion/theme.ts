import brand from "../brand/brand.json";

export type Theme = "light" | "dark";

export const THEME_KEY = "sens-theme";

export const pageColors: Record<Theme, string> = {
  light: brand.palette.paper,
  dark: brand.palette["carbon-950"],
};

const SCAN = ["--scan-x", "--scan-y", "--scan-far"] as const;

const opposite = (theme: Theme): Theme => (theme === "dark" ? "light" : "dark");

const shown = (root: HTMLElement): Theme => (root.dataset.theme === "dark" ? "dark" : "light");

function chosen(): Theme | null {
  try {
    const stored = localStorage.getItem(THEME_KEY);
    return stored === "dark" || stored === "light" ? stored : null;
  } catch {
    return null;
  }
}

function remember(theme: Theme): void {
  try {
    localStorage.setItem(THEME_KEY, theme);
  } catch {
    return;
  }
}

const afterPaint = () =>
  new Promise<void>((painted) => requestAnimationFrame(() => requestAnimationFrame(() => painted())));

function centreScanOn(root: HTMLElement, origin: Element): void {
  const box = origin.getBoundingClientRect();
  const x = box.left + box.width / 2;
  const y = box.top + box.height / 2;
  const far = Math.hypot(Math.max(x, innerWidth - x), Math.max(y, innerHeight - y));
  root.style.setProperty("--scan-x", `${x}px`);
  root.style.setProperty("--scan-y", `${y}px`);
  root.style.setProperty("--scan-far", `${far}px`);
}

export function theme(options: { quiet: boolean }): void {
  const toggle = document.querySelector<HTMLButtonElement>("[data-theme-toggle]");
  if (!toggle) return;
  const root = document.documentElement;
  const meta = document.querySelector('meta[name="theme-color"]');
  let switches = 0;

  const paint = (next: Theme) => {
    root.dataset.theme = next;
    meta?.setAttribute("content", pageColors[next]);
    toggle.setAttribute("aria-pressed", String(next === "dark"));
  };

  const settle = (switched: number) => () => {
    if (switched !== switches) return;
    delete root.dataset.theming;
    for (const name of SCAN) root.style.removeProperty(name);
  };

  const snap = (next: Theme) => {
    const switched = ++switches;
    root.dataset.theming = "";
    paint(next);
    afterPaint().then(settle(switched));
  };

  const sweep = (next: Theme) => {
    const switched = ++switches;
    centreScanOn(root, toggle);
    root.dataset.theming = next;
    document.startViewTransition(() => paint(next)).finished.finally(settle(switched));
  };

  toggle.setAttribute("aria-pressed", String(shown(root) === "dark"));
  toggle.addEventListener("click", () => {
    const next = opposite(shown(root));
    remember(next);
    if (options.quiet || !("startViewTransition" in document)) snap(next);
    else sweep(next);
  });

  matchMedia("(prefers-color-scheme: dark)").addEventListener("change", (event) => {
    if (!chosen()) snap(event.matches ? "dark" : "light");
  });
}
