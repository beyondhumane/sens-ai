import { describe, expect, it } from "vitest";
import { median, niceScale, pathOf, spread } from "../src/lib/chart";

describe("chart helpers", () => {
  it("takes the median the way the paper does", () => {
    expect(median([484, 496, 564])).toBe(496);
    expect(median([424, 436, 456])).toBe(436);
    expect(median([1, 2, 3, 4])).toBe(2.5);
  });

  it("rounds the axis up to a readable top with even ticks", () => {
    expect(niceScale(564)).toEqual({ min: 0, max: 600, ticks: [0, 200, 400, 600] });
    expect(niceScale(11_930_000).max).toBe(12_500_000);
  });

  it("pushes end labels apart so none overlaps the one above", () => {
    expect(spread([100, 104, 300], 18)).toEqual([100, 118, 300]);
    expect(spread([120, 100], 18)).toEqual([120, 100]);
  });

  it("draws a path through the points", () => {
    expect(pathOf([[0, 10], [5, 2.25]])).toBe("M0.0,10.0 L5.0,2.3");
  });
});
