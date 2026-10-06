import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import brand from "../src/brand/brand.json";
import { WINDOW } from "../src/motion/geometry";
import { bezierOf, curves, durations, motionCss } from "../src/motion/tokens";

describe("motion tokens", () => {
  it("take the hover time, the confirmation time and the control curve from the brand", () => {
    expect(`${durations.fast}ms`).toBe(brand.motion.hover);
    expect(`${durations.confirm}ms`).toBe(brand.motion.confirm);
    expect(curves.control).toBe(brand.motion.ease);
  });

  it("turn every curve into the four numbers GSAP reads", () => {
    for (const curve of ["control", "enter", "exit", "move"] as const) {
      expect(bezierOf(curve).split(",")).toHaveLength(4);
    }
  });

  it("write the same values as CSS variables", () => {
    const css = motionCss();
    expect(css).toContain(`--dur-fast: ${durations.fast}ms;`);
    expect(css).toContain(`--ease-move: ${curves.move};`);
  });
});

describe("window size", () => {
  it("is the same in the geometry, the page and the product styles", () => {
    const base = readFileSync(new URL("../src/styles/base.css", import.meta.url), "utf8");
    const product = readFileSync(new URL("../src/styles/product.css", import.meta.url), "utf8");
    expect(base).toContain(`--window-w: ${WINDOW.width}px;`);
    expect(base).toContain(`--window-h: ${WINDOW.height}px;`);
    const thread = Number(product.match(/--thread: (\d+)px;/)?.[1]);
    const tools = Number(product.match(/--tools: (\d+)px;/)?.[1]);
    expect(thread + tools).toBe(WINDOW.width);
    expect(product).toContain(`height: ${WINDOW.height}px;`);
  });
});
