import { beforeEach, describe, expect, it, vi } from "vitest";
import type { BarOpened, HandOver } from "../ipc/types";
import { bar, hear, hide, opened, own } from "./store";

const ipc = vi.hoisted(() => ({
  commands: new Proxy({} as Record<string, ReturnType<typeof vi.fn>>, {
    get: (known, name: string) => (known[name] ??= vi.fn(async () => undefined)),
  }),
}));

vi.mock("../ipc/commands", () => ({ commands: ipc.commands, events: {} }));

const PROJECTS = [
  { root: "C:/Proyectos/nitid", name: "nitid" },
  { root: "C:/Proyectos/web", name: "web" },
];

const open = (resume: HandOver | null) =>
  opened({ look: { mode: "dark", accent: "signal" }, language: "es", front: null, pinned: false, resume } as BarOpened);

beforeEach(() => {
  for (const command of Object.values(ipc.commands)) command.mockClear();
  ipc.commands.barProjects.mockResolvedValue(PROJECTS);
  ipc.commands.workspaces.mockResolvedValue([]);
  ipc.commands.replay.mockResolvedValue([{ kind: "task", at: 1, text: "Arregla el login", files: [], images: [] }]);
  ipc.commands.chatTasks.mockResolvedValue([]);
  ipc.commands.chatBusy.mockResolvedValue(false);
  own.chat.setState({ turns: [], busy: false });
  own.desk.setState({ root: "", session: "", text: "" });
});

describe("how the bar comes and goes", () => {
  it("fades out before it hides, and stays faded until it opens again", async () => {
    vi.useFakeTimers();
    vi.stubGlobal("matchMedia", () => ({ matches: false }));
    hide();
    expect(bar.getState().leaving).toBe(true);
    expect(ipc.commands.barHide).not.toHaveBeenCalled();
    vi.advanceTimersByTime(140);
    expect(ipc.commands.barHide).toHaveBeenCalled();
    expect(bar.getState().leaving).toBe(true);
    vi.useRealTimers();

    await open(null);
    expect(bar.getState().leaving).toBe(false);
    vi.unstubAllGlobals();
  });

  it("hides at once when the system asks for less motion", () => {
    vi.useFakeTimers();
    vi.stubGlobal("matchMedia", () => ({ matches: true }));
    hide();
    vi.advanceTimersByTime(0);
    expect(ipc.commands.barHide).toHaveBeenCalled();
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("knows while you dictate, and how loud", () => {
    hear(true, 0.4);
    expect(bar.getState()).toMatchObject({ listening: true, level: 0.4 });
    hear(false, 0);
    expect(bar.getState()).toMatchObject({ listening: false, level: 0 });
  });
});

describe("where the bar opens", () => {
  it("on a new session in the last project, when the shortcut opens it", async () => {
    await open(null);
    expect(own.desk.getState()).toMatchObject({ root: "C:/Proyectos/nitid", session: "" });
    expect(ipc.commands.replay).not.toHaveBeenCalled();
  });

  it("on the session Sens had open, when focus mode is entered from Sens", async () => {
    await open({ root: "C:/Proyectos/web", session: "s7", text: "" });
    expect(own.desk.getState()).toMatchObject({ root: "C:/Proyectos/web", session: "s7" });
    expect(own.chat.getState().turns).toMatchObject([{ kind: "you", text: "Arregla el login" }]);
  });

  it("on a new session in Sens's project, when Sens had not sent anything there yet", async () => {
    await open({ root: "C:/Proyectos/web", session: "", text: "" });
    expect(own.desk.getState()).toMatchObject({ root: "C:/Proyectos/web", session: "" });
    expect(ipc.commands.replay).not.toHaveBeenCalled();
  });
});
