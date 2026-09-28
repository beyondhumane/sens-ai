import { act, cleanup, render } from "@testing-library/react";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import type { HandOver } from "../../ipc/types";
import { focused } from "../panes/store";
import { project } from "../project/store";
import { hearHandOver } from "./store";

const ipc = vi.hoisted(() => ({
  commands: new Proxy({} as Record<string, ReturnType<typeof vi.fn>>, {
    get: (known, name: string) => (known[name] ??= vi.fn(async () => undefined)),
  }),
  heard: null as ((hand: HandOver) => void) | null,
}));

vi.mock("../../ipc/commands", () => ({
  commands: ipc.commands,
  events: { barHandOver: (heard: (hand: HandOver) => void) => ((ipc.heard = heard), Promise.resolve(() => {})) },
}));

const handOver = (hand: HandOver) =>
  act(async () => {
    ipc.heard!(hand);
    await new Promise((done) => setTimeout(done, 50));
  });

const field = () => document.getElementById("task");
const written = () => focused().desk.getState().text;

beforeAll(() => {
  hearHandOver();
});

beforeEach(() => {
  project.setState({ root: "", work: "", session: "", view: "", touched: new Map() });
  focused().desk.setState({ root: "C:/demo", session: "", text: "" });
  for (const command of Object.values(ipc.commands)) command.mockClear();
  ipc.commands.workspaces.mockResolvedValue([]);
  ipc.commands.replay.mockResolvedValue([{ kind: "task", at: 1, text: "Arregla el login", files: [], images: [] }]);
  ipc.commands.chatTasks.mockResolvedValue([]);
  ipc.commands.chatBusy.mockResolvedValue(false);
  ipc.commands.folder.mockResolvedValue([]);
  render(<textarea id="task" />);
});

afterEach(cleanup);

describe("what the bar hands over", () => {
  it("opens the session the bar was on, with what you were writing ready in the field", async () => {
    await handOver({ root: "C:/demo", session: "s7", text: "Y añade un test" });
    expect(project.getState()).toMatchObject({ root: "C:/demo", session: "s7" });
    expect(focused().chat.getState().turns).toMatchObject([{ kind: "you", text: "Arregla el login" }]);
    expect(written()).toBe("Y añade un test");
    expect(document.activeElement).toBe(field());
  });

  it("starts a new session in the bar's project when the bar had not sent anything", async () => {
    await handOver({ root: "C:/web", session: "", text: "Revisa el formulario" });
    expect(project.getState()).toMatchObject({ root: "C:/web", session: "" });
    expect(ipc.commands.remember).toHaveBeenCalledWith("C:/web");
    expect(ipc.commands.replay).not.toHaveBeenCalled();
    expect(written()).toBe("Revisa el formulario");
    expect(document.activeElement).toBe(field());
  });

  it("keeps what the field already had and adds the bar's text below it", async () => {
    focused().desk.setState({ text: "Mi borrador" });
    await handOver({ root: "C:/demo", session: "s7", text: "Y añade un test" });
    expect(written()).toBe("Mi borrador\n\nY añade un test");
  });

  it("only takes the focus when nothing was written in the bar", async () => {
    focused().desk.setState({ text: "Mi borrador" });
    await handOver({ root: "C:/demo", session: "s7", text: "" });
    expect(project.getState().session).toBe("s7");
    expect(written()).toBe("Mi borrador");
    expect(document.activeElement).toBe(field());
  });

  it("stays in the project on screen when the bar had none", async () => {
    await handOver({ root: "", session: "", text: "Hola" });
    expect(project.getState().root).toBe("C:/demo");
    expect(ipc.commands.remember).not.toHaveBeenCalled();
    expect(written()).toBe("Hola");
    expect(document.activeElement).toBe(field());
  });
});
