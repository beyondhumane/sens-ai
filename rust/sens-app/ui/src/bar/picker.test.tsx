import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import type { SessionSummary } from "../ipc/types";
import { Picker, SessionChip } from "./Picker";
import { bar, own } from "./store";

const ipc = vi.hoisted(() => ({
  commands: new Proxy({} as Record<string, ReturnType<typeof vi.fn>>, {
    get: (known, name: string) => (known[name] ??= vi.fn(async () => undefined)),
  }),
}));

vi.mock("../ipc/commands", () => ({ commands: ipc.commands, events: {} }));

const session = (id: string, title: string, startedAt: number): SessionSummary => ({ id, title, startedAt, tasks: 1, archived: false });

function Both() {
  return (
    <>
      <textarea id="task" />
      <SessionChip />
      <Picker />
    </>
  );
}

const chip = () => screen.getByRole("button", { expanded: bar.getState().choosing });
const search = () => screen.getByRole("combobox", { name: "Busca una sesión o un proyecto" });
const options = () => screen.getAllByRole("option").map((option) => option.textContent);
const active = () => document.querySelector('[data-active="true"]')?.textContent;

beforeAll(() => {
  Element.prototype.scrollIntoView = vi.fn();
});

beforeEach(() => {
  for (const command of Object.values(ipc.commands)) command.mockClear();
  ipc.commands.replay.mockResolvedValue([{ kind: "task", at: 1, text: "Arregla el login", files: [], images: [] }]);
  ipc.commands.chatTasks.mockResolvedValue([]);
  ipc.commands.chatBusy.mockResolvedValue(false);
  own.chat.setState({ turns: [], busy: false });
  own.desk.setState({ root: "C:/nitid", session: "", text: "sigue con esto" });
  bar.setState({
    choosing: false,
    filter: "",
    projects: [
      { root: "C:/nitid", name: "nitid" },
      { root: "C:/web", name: "web" },
    ],
    sessions: { "C:/nitid": [session("a", "Refactor del login", 10), session("b", "Tests del botón", 30)], "C:/web": [session("d", "Formulario de contacto", 40)] },
  });
});

afterEach(cleanup);

describe("the session picker", () => {
  it("names the project and the session, and opens on the one the bar is on with the search ready", async () => {
    render(<Both />);
    expect(chip().textContent).toBe("nitidNueva sesión");

    await act(async () => fireEvent.click(chip()));
    expect(document.activeElement).toBe(search());
    expect(options()).toEqual(["Nueva sesión", "Tests del botón", "Refactor del login", "webnueva sesión"]);
    expect(active()).toBe("Nueva sesión");
    expect(screen.getByText("Otros proyectos")).toBeTruthy();
  });

  it("goes with the arrows and resumes the session chosen with Enter, keeping what was written", async () => {
    render(<Both />);
    await act(async () => fireEvent.click(chip()));
    fireEvent.keyDown(search(), { key: "ArrowDown" });
    fireEvent.keyDown(search(), { key: "ArrowDown" });
    expect(active()).toBe("Refactor del login");

    await act(async () => fireEvent.keyDown(search(), { key: "Enter" }));
    expect(bar.getState().choosing).toBe(false);
    expect(own.desk.getState()).toMatchObject({ root: "C:/nitid", session: "a", text: "sigue con esto" });
    expect(ipc.commands.replay).toHaveBeenCalledWith("C:/nitid", "a");
    expect(own.chat.getState().turns).toMatchObject([{ kind: "you", text: "Arregla el login" }]);
    expect(chip().textContent).toBe("nitidRefactor del login");
    expect(document.activeElement).toBe(document.getElementById("task"));
  });

  it("finds a session of another project by its title, and starts anew in a project picked by name", async () => {
    render(<Both />);
    await act(async () => fireEvent.click(chip()));
    fireEvent.change(search(), { target: { value: "formulario" } });
    expect(options()).toEqual(["Formulario de contactoweb"]);
    await act(async () => fireEvent.click(screen.getByRole("option")));
    expect(own.desk.getState()).toMatchObject({ root: "C:/web", session: "d" });

    await act(async () => fireEvent.click(chip()));
    fireEvent.change(search(), { target: { value: "nitid" } });
    await act(async () => fireEvent.keyDown(search(), { key: "Enter" }));
    expect(own.desk.getState()).toMatchObject({ root: "C:/nitid", session: "" });
  });

  it("says when nothing is called that", async () => {
    render(<Both />);
    await act(async () => fireEvent.click(chip()));
    fireEvent.change(search(), { target: { value: "zzz" } });
    expect(screen.queryAllByRole("option")).toEqual([]);
    expect(screen.getByText("Ninguna sesión ni proyecto se llama así")).toBeTruthy();
  });
});
