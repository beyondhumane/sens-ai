import { describe, expect, it } from "vitest";
import type { SessionSummary } from "../ipc/types";
import { MOST_SESSIONS, choicesOf, titleOf } from "./choices";

const PROJECTS = [
  { root: "C:/nitid", name: "nitid" },
  { root: "C:/web", name: "web" },
  { root: "C:/api", name: "api" },
];

const session = (id: string, title: string, startedAt: number, archived = false): SessionSummary => ({ id, title, startedAt, tasks: 1, archived });

const SESSIONS = {
  "C:/nitid": [session("a", "Refactor del login", 10), session("b", "Tests del botón", 30), session("c", "Vieja", 20, true)],
  "C:/web": [session("d", "Formulario de contacto", 40), session("e", "Login con Google", 5)],
};

const shown = (choices: ReturnType<typeof choicesOf>) =>
  choices.map((choice) => (choice.kind === "session" ? `${choice.name}/${choice.title}` : `${choice.kind}:${choice.name}`));

describe("where the bar can ask", () => {
  it("offers a new session first, then the project's sessions newest first, then the other projects", () => {
    expect(shown(choicesOf(PROJECTS, SESSIONS, "C:/nitid", ""))).toEqual(["new:nitid", "nitid/Tests del botón", "nitid/Refactor del login", "project:web", "project:api"]);
  });

  it("leaves archived sessions out, and shows no more than a handful", () => {
    const many = { "C:/nitid": Array.from({ length: MOST_SESSIONS + 4 }, (_, at) => session(`s${at}`, `Sesión ${at}`, at)) };
    const sessions = choicesOf(PROJECTS, many, "C:/nitid", "").filter((choice) => choice.kind === "session");
    expect(sessions).toHaveLength(MOST_SESSIONS);
    expect(shown(choicesOf(PROJECTS, SESSIONS, "C:/nitid", ""))).not.toContain("nitid/Vieja");
  });

  it("finds sessions of every project by their title, this project's first, and projects by their name", () => {
    expect(shown(choicesOf(PROJECTS, SESSIONS, "C:/web", "login"))).toEqual(["web/Login con Google", "nitid/Refactor del login"]);
    expect(shown(choicesOf(PROJECTS, SESSIONS, "C:/web", "  API "))).toEqual(["project:api"]);
    expect(choicesOf(PROJECTS, SESSIONS, "C:/web", "nada")).toEqual([]);
  });

  it("offers only the projects when no project is open yet", () => {
    expect(shown(choicesOf(PROJECTS, SESSIONS, "", ""))).toEqual(["project:nitid", "project:web", "project:api"]);
  });

  it("names the session the bar is on, and nothing for a new one", () => {
    expect(titleOf(SESSIONS, "C:/web", "d")).toBe("Formulario de contacto");
    expect(titleOf(SESSIONS, "C:/web", "")).toBe("");
    expect(titleOf(SESSIONS, "C:/web", "desconocida")).toBe("");
  });
});
