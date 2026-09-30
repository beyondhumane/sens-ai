import brand from "../brand/brand.json";
import { motionCss } from "../motion/tokens";

export function rootCss(): string {
  return [
    ":root {",
    `  --font-ui: ${brand.fontSans};`,
    `  --font-mono: ${brand.fontMono};`,
    "}",
    motionCss(),
  ].join("\n");
}
