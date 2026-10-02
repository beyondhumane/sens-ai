import { describe, expect, it } from "vitest";
import { choicesOf } from "../bar/choices";
import type { SessionSummary } from "../ipc/types";

const PROJECTS = [
  { root: "C:/nana", name: "nana" },
  { root: "C:/diseno", name: "Diseño" },
  { root: "C:/api", name: "api" },
];

const session = (id: string, title: string, startedAt: number): SessionSummary => ({ id, title, startedAt, tasks: 1, archived: false });

const SESSIONS = {
  "C:/nana": [session("a", "Canción de cuna", 10), session("b", "Menú de ajustes", 20)],
  "C:/api": [session("c", "Pingüino del logo", 30)],
};

const shown = (filter: string) =>
  choicesOf(PROJECTS, SESSIONS, "C:/nana", filter).map((choice) => (choice.kind === "session" ? choice.title : `${choice.kind}:${choice.name}`));

describe("searching in the bar", () => {
  it("finds sessions whatever the accents typed", () => {
    expect(shown("cancion")).toEqual(["Canción de cuna"]);
    expect(shown("CANCIÓN")).toEqual(["Canción de cuna"]);
    expect(shown("menu")).toEqual(["Menú de ajustes"]);
    expect(shown("pinguino")).toEqual(["Pingüino del logo"]);
  });

  it("finds projects whatever the accents typed", () => {
    expect(shown("diseno")).toEqual(["project:Diseño"]);
    expect(shown("DISEÑO")).toEqual(["project:Diseño"]);
  });

  it("still tells different words apart", () => {
    expect(shown("canciones")).toEqual([]);
    expect(shown("nana")).toEqual(["project:nana"]);
  });
});
