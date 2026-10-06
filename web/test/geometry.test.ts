import { describe, expect, it } from "vitest";
import { frameOf, NAV, WINDOW } from "../src/motion/geometry";

const sizes = [
  { width: 1024, height: 768 },
  { width: 1280, height: 800 },
  { width: 1536, height: 740 },
  { width: 1440, height: 900 },
  { width: 1536, height: 864 },
  { width: 1888, height: 930 },
  { width: 1920, height: 1080 },
  { width: 2560, height: 1440 },
];

describe("frameOf", () => {
  for (const { width, height } of sizes) {
    const frame = frameOf({ width, height });
    const bottom = frame.hero.y + WINDOW.height * frame.hero.s;

    it(`keeps the stage window inside the viewport at ${width}×${height}`, () => {
      expect(frame.stage.x).toBeGreaterThan(0);
      expect(frame.stage.x + WINDOW.width * frame.stage.s).toBeLessThanOrEqual(width);
      expect(frame.stage.y).toBeGreaterThanOrEqual(NAV);
      expect(frame.stage.y + WINDOW.height * frame.stage.s).toBeLessThanOrEqual(height);
    });

    it(`never shrinks the window when it moves to the stage at ${width}×${height}`, () => {
      expect(frame.stage.s).toBeGreaterThanOrEqual(frame.hero.s);
      expect(frame.stage.s).toBeLessThanOrEqual(1);
    });

    it(`keeps the hero window inside the viewport and at least half size at ${width}×${height}`, () => {
      expect(frame.hero.x + WINDOW.width * frame.hero.s).toBeLessThanOrEqual(width);
      expect(frame.hero.s).toBeGreaterThanOrEqual(0.5);
    });

    it(`lets the field run behind the window and below it at ${width}×${height}`, () => {
      expect(frame.field.x).toBeGreaterThan(frame.hero.x);
      expect(frame.field.height).toBeGreaterThan(bottom);
      expect(frame.field.x + frame.field.width).toBe(width);
    });

    it(`sets the mark just left of the stage window, as tall as it, at ${width}×${height}`, () => {
      expect(frame.rail.x + frame.rail.width).toBeLessThan(frame.stage.x);
      expect(frame.rail.height).toBeCloseTo(WINDOW.height * frame.stage.s);
      expect(frame.rail.y).toBe(frame.stage.y);
    });
  }
});
