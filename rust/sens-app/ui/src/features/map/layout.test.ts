import { describe, expect, it } from "vitest";
import { layered, shortName } from "./layout";

const link = (from: string, to: string) => ({ from, to, weight: 1 });

describe("the layered layout", () => {
  it("puts what is used below what uses it, and lines up a layer under what it uses", () => {
    const laid = layered(["app", "ui", "core", "db"], [link("app", "ui"), link("ui", "core"), link("app", "db")]);
    const y = (name: string) => laid.nodes.find((node) => node.name === name)!.y;
    expect(y("app")).toBeLessThan(y("ui"));
    expect(y("ui")).toBeLessThan(y("core"));
    expect(y("core")).toBe(y("db"));
    expect(laid.nodes.every((node) => node.x >= 0 && node.x + node.width <= laid.width)).toBe(true);
  });

  it("survives a cycle between areas and ignores links to areas it does not draw", () => {
    const laid = layered(["a", "b"], [link("a", "b"), link("b", "a"), link("a", "gone")]);
    expect(laid.nodes.map((node) => node.name).sort()).toEqual(["a", "b"]);
    expect(laid.height).toBeGreaterThan(0);
  });

  it("is the same every time", () => {
    const names = ["x", "y", "z"];
    const links = [link("x", "z"), link("y", "z")];
    expect(layered(names, links)).toEqual(layered(names, links));
  });

  it("names a deep area by its last two folders", () => {
    expect(shortName("rust/sens-app/ui/src/features/chat")).toBe("features/chat");
    expect(shortName("src")).toBe("src");
  });
});
