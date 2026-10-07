import { describe, expect, it } from "vitest";
import { boundsOf, linkPath, NODE_HEIGHT, type Placed } from "./layout";
import { centeredOn, fitted, inSight, LEAST_ZOOM, MOST_ZOOM, zoomedAt } from "./view";

const node = (name: string, x: number, y: number, width = 100): Placed => ({ name, label: name, x, y, width });

describe("the map's view", () => {
  it("fits what is drawn in the middle of the stage, never bigger than a little over life size", () => {
    const view = fitted({ left: 100, top: 50, width: 200, height: 100 }, { width: 1000, height: 600 });
    expect(view.k).toBe(1.2);
    expect(100 * view.k + view.x + (200 * view.k) / 2).toBeCloseTo(500);
    expect(50 * view.k + view.y + (100 * view.k) / 2).toBeCloseTo(300);
    expect(fitted({ left: 0, top: 0, width: 10000, height: 10000 }, { width: 400, height: 300 }).k).toBe(LEAST_ZOOM);
  });

  it("zooms around the pointer, keeping what is under it in place, within its limits", () => {
    const view = { x: 40, y: 20, k: 1 };
    const at = { x: 300, y: 200 };
    const closer = zoomedAt(view, at, 2);
    expect((at.x - closer.x) / closer.k).toBeCloseTo((at.x - view.x) / view.k);
    expect((at.y - closer.y) / closer.k).toBeCloseTo((at.y - view.y) / view.k);
    expect(zoomedAt(view, at, 100).k).toBe(MOST_ZOOM);
  });

  it("tells whether a box is in sight, and centers on a point", () => {
    const stage = { width: 400, height: 300 };
    const view = { x: 0, y: 0, k: 1 };
    expect(inSight(view, { x: 10, y: 10, width: 50, height: 50 }, stage)).toBe(true);
    expect(inSight(view, { x: 380, y: 10, width: 50, height: 50 }, stage)).toBe(false);
    expect(centeredOn(view, { x: 1000, y: 500 }, stage)).toEqual({ x: -800, y: -350, k: 1 });
  });
});

describe("the map's links", () => {
  it("leave one area at its edge and arrive at the other's", () => {
    const from = node("a", 0, 0);
    const to = node("b", 0, 200);
    const { d } = linkPath(from, to);
    const [, x1, y1, , , , x2, y2] = d.split(" ").map(Number);
    expect(y1).toBeCloseTo(NODE_HEIGHT);
    expect(y2).toBeLessThan(200);
    expect(y2).toBeGreaterThan(190);
    expect(Math.abs(x1 - 50)).toBeLessThan(50);
    expect(Math.abs(x2 - 50)).toBeLessThan(50);
  });

  it("bends the two ways between a pair apart, so both stay visible", () => {
    const a = node("a", 0, 0);
    const b = node("b", 300, 0);
    expect(linkPath(a, b).middle.y).not.toBeCloseTo(linkPath(b, a).middle.y);
  });

  it("swings a link that skips a row around the areas between", () => {
    const { middle } = linkPath(node("a", 0, 0), node("c", 0, 300, 140));
    expect(Math.abs(middle.x - 70)).toBeGreaterThan(40);
  });

  it("bounds what is drawn", () => {
    expect(boundsOf([node("a", 10, 20), node("b", 200, 120, 80)])).toEqual({ left: 10, top: 20, width: 270, height: 100 + NODE_HEIGHT });
  });
});
