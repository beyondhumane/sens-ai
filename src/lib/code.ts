import { createHighlighter } from "shiki";

export type Language = "tsx" | "shellscript";

export interface Token {
  content: string;
  color: string | null;
}

const THEMES = { light: "light-plus", dark: "dark-plus" } as const;

const highlighter = await createHighlighter({ themes: Object.values(THEMES), langs: ["tsx", "shellscript"] });

const plain = {
  light: highlighter.getTheme(THEMES.light).fg.toLowerCase(),
  dark: highlighter.getTheme(THEMES.dark).fg.toLowerCase(),
};

const named = (color: string | undefined, fallback: string): string | null => {
  const opaque = color?.toLowerCase().replace(/^(#[0-9a-f]{6})ff$/, "$1");
  return opaque && opaque !== fallback ? opaque : null;
};

export function tokensOf(code: string, lang: Language): Token[][] {
  return highlighter.codeToTokensWithThemes(code, { lang, themes: THEMES }).map((line) =>
    line.map(({ content, variants }) => {
      const light = named(variants.light?.color, plain.light);
      const dark = named(variants.dark?.color, plain.dark);
      const color = light || dark ? `light-dark(${light ?? "var(--text)"}, ${dark ?? "var(--text)"})` : null;
      return { content, color };
    }),
  );
}
