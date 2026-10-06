// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";
import { viewer } from "../features/files/view";
import { project } from "../features/project/store";
import { consoles } from "../features/terminal/store";
import { perform } from "./acts";
import { shell } from "./shell";
import { answerSurface } from "./surface";

const ipc = vi.hoisted(() => ({
  commands: { openFile: vi.fn(), folder: vi.fn(), actAnswer: vi.fn() },
}));

vi.mock("../ipc/commands", () => ({ commands: ipc.commands, events: {} }));

const ask = async (act: string, input: Record<string, unknown>) => {
  await perform({ ask: 1, act, input });
  const [, ok, text] = ipc.commands.actAnswer.mock.calls.at(-1)!;
  return { ok, text };
};

answerSurface();

beforeEach(() => {
  for (const command of Object.values(ipc.commands)) command.mockReset().mockResolvedValue(undefined);
  ipc.commands.openFile.mockResolvedValue({ kind: "text", text: "uno\ndos\ntres\n" });
  project.setState({ root: "C:/demo", work: "C:/demo", session: "s1" });
  shell.setState(shell.getInitialState(), true);
  viewer.setState(viewer.getInitialState(), true);
});

describe("what Claude changes on screen", () => {
  it("opens a file at a line in the files pane", async () => {
    expect(await ask("open_file", { session: "s1", path: "src/app.ts", line: 2 })).toEqual({ ok: true, text: "Showing src/app.ts at line 2." });
    expect(ipc.commands.openFile).toHaveBeenCalledWith("C:/demo", "src/app.ts");
    expect(viewer.getState()).toMatchObject({ title: "src/app.ts", line: 2 });
    expect(shell.getState()).toMatchObject({ toolsOpen: true, tool: "files" });
  });

  it("says why a file could not be read", async () => {
    ipc.commands.openFile.mockResolvedValue(Promise.reject("no existe"));
    expect(await ask("open_file", { session: "s1", path: "x.ts", line: null })).toEqual({ ok: false, text: "no existe" });
  });

  it("opens and closes the side pane", async () => {
    expect((await ask("show_pane", { session: "s1", pane: "changes" })).ok).toBe(true);
    expect(shell.getState()).toMatchObject({ toolsOpen: true, tool: "changes" });
    await ask("close_pane", { session: "s1" });
    expect(shell.getState().toolsOpen).toBe(false);
  });

  it("leaves the screen alone when the person looks at another session", async () => {
    const said = await ask("show_pane", { session: "s2", pane: "web" });
    expect(said).toEqual({ ok: false, text: "The person is looking at another session, so Sens left their screen as it was." });
    expect(shell.getState().toolsOpen).toBe(false);
    expect((await ask("open_file", { session: "s2", path: "src/app.ts", line: null })).ok).toBe(false);
    expect(ipc.commands.openFile).not.toHaveBeenCalled();
  });

  it("tells what is on screen", async () => {
    consoles.setState({ open: [{ id: 3, root: "C:/demo", shell: "pwsh", ended: false }, { id: 4, root: "C:/demo", shell: "powershell", ended: false, title: "vite" }] });
    await ask("open_file", { session: "s1", path: "src/app.ts", line: null });
    expect((await ask("get_layout", { session: "s1" })).text).toBe(
      ["The person is looking at this session.", "Side pane: files, showing src/app.ts, with the tree.", "Terminals: 2 open, 1 started by Claude.", "One chat on screen."].join("\n"),
    );
    expect((await ask("get_layout", { session: "s9" })).text).toContain("The person is looking at another session, in C:/demo.");
  });
});
