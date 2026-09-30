import { describe, expect, it } from "vitest";
import { siteIcons } from "../src/content/icons";

const rays = [...siteIcons.theme.matchAll(/d="([^"]+)" class="sun-ray" style="--turn: (\d+)"/g)].map(([, path, turn]) => ({
  path,
  turn: Number(turn),
}));

describe("the theme icon", () => {
  it("gives each of the sun's rays its own step, clockwise from the top", () => {
    expect(rays.map(({ turn }) => turn).sort()).toEqual([0, 1, 2, 3, 4, 5, 6, 7]);
    expect(rays.find(({ path }) => path === "M12 2v2")?.turn).toBe(0);
    expect(rays.find(({ path }) => path === "M20 12h2")?.turn).toBe(2);
    expect(rays.find(({ path }) => path === "M12 20v2")?.turn).toBe(4);
    expect(rays.find(({ path }) => path === "M2 12h2")?.turn).toBe(6);
  });

  it("measures the sun's core and the moon as one length, so either can be drawn in", () => {
    expect(siteIcons.theme).toMatch(/<circle [^>]*class="sun-core" pathLength="1"/);
    expect(siteIcons.theme).toMatch(/<path [^>]*class="moon" pathLength="1"/);
  });
});
