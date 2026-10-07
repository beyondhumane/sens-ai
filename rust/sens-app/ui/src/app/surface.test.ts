// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";
import { asker } from "../dev/acting";
import { viewer } from "../features/files/view";
import { project } from "../features/project/store";
import { consoles } from "../features/terminal/store";
import { web } from "../features/web/store";
import { panelShows, shell, toolsShown } from "./shell";
import { answerSurface } from "./surface";

const ipc = vi.hoisted(() => ({
  commands: { openFile: vi.fn(), folder: vi.fn(), actAnswer: vi.fn(), browserOpen: vi.fn(), browserShow: vi.fn(), browserPlace: vi.fn(), browserAct: vi.fn() },
}));

vi.mock("../ipc/commands", () => ({ commands: ipc.commands, events: {} }));

const ask = asker(ipc.commands.actAnswer);

answerSurface();

beforeEach(() => {
  for (const command of Object.values(ipc.commands)) command.mockReset().mockResolvedValue(undefined);
  ipc.commands.openFile.mockResolvedValue({ kind: "text", text: "uno\ndos\ntres\n" });
  project.setState({ root: "C:/demo", work: "C:/demo", session: "s1" });
  shell.setState(shell.getInitialState(), true);
  viewer.setState(viewer.getInitialState(), true);
  web.setState(web.getInitialState(), true);
});

describe("what Claude changes on screen", () => {
  it("opens a file at a line in the files pane", async () => {
    expect(await ask("open_file", { session: "s1", path: "src/app.ts", line: 2 })).toEqual({ ok: true, text: "Showing src/app.ts at line 2." });
    expect(ipc.commands.openFile).toHaveBeenCalledWith("C:/demo", "src/app.ts");
    expect(viewer.getState()).toMatchObject({ title: "src/app.ts", line: 2 });
    expect(panelShows("files")).toBe(true);
  });

  it("says why a file could not be read", async () => {
    ipc.commands.openFile.mockResolvedValue(Promise.reject("no existe"));
    expect(await ask("open_file", { session: "s1", path: "x.ts", line: null })).toEqual({ ok: false, text: "no existe" });
  });

  it("opens and closes the side pane", async () => {
    expect((await ask("show_pane", { session: "s1", pane: "changes" })).ok).toBe(true);
    expect(panelShows("changes")).toBe(true);
    await ask("close_pane", { session: "s1" });
    expect(toolsShown()).toBe(false);
  });

  it("leaves the screen alone when the person looks at another session", async () => {
    const said = await ask("show_pane", { session: "s2", pane: "web" });
    expect(said).toEqual({ ok: false, text: "The person is looking at another session, so Sens left their screen as it was." });
    expect(toolsShown()).toBe(false);
    expect((await ask("open_file", { session: "s2", path: "src/app.ts", line: null })).ok).toBe(false);
    expect(ipc.commands.openFile).not.toHaveBeenCalled();
  });

  it("tells what is on screen", async () => {
    consoles.setState({ open: [{ id: 3, root: "C:/demo", shell: "pwsh", ended: false }, { id: 4, root: "C:/demo", shell: "powershell", ended: false, title: "vite" }] });
    await ask("open_file", { session: "s1", path: "src/app.ts", line: null });
    expect((await ask("get_layout", { session: "s1" })).text).toBe(
      ["The person is looking at this session.", "Panels: files on the right, showing src/app.ts, with the tree.", "Terminals: 2 open, 1 started by Claude.", "One chat on screen."].join("\n"),
    );
    expect((await ask("get_layout", { session: "s9" })).text).toContain("The person is looking at another session, in C:/demo.");
  });

  it("browses to a page and waits until it has loaded", async () => {
    const said = ask("browse", { session: "s1", url: "localhost:5173" });
    await new Promise((done) => setTimeout(done, 0));
    web.setState({ loading: true });
    web.setState({ loading: false, url: "http://localhost:5173/", title: "Vite" });
    expect(await said).toEqual({ ok: true, text: "Loaded http://localhost:5173/ · Vite" });
    expect(ipc.commands.browserOpen).toHaveBeenCalledWith("http://localhost:5173", expect.anything(), 1);
    expect(panelShows("web")).toBe(true);
  });

  it("goes back and waits for the page it lands on", async () => {
    web.setState({ url: "http://localhost:5173/b" });
    const said = ask("browse", { session: "s1", url: "back" });
    await new Promise((done) => setTimeout(done, 0));
    web.setState({ loading: true });
    web.setState({ loading: false, url: "http://localhost:5173/a" });
    expect((await said).text).toBe("Loaded http://localhost:5173/a");
    expect(ipc.commands.browserAct).toHaveBeenCalledWith("back");
  });

  it("draws the page at a phone's width, and acts on the page only for the session on screen", async () => {
    expect(await ask("browser_width", { session: "s1", width: 375 })).toEqual({ ok: true, text: "The page is drawn 375 px wide." });
    expect(web.getState().width).toBe(375);
    expect(await ask("present", { session: "s1" })).toEqual({ ok: true, text: "" });
    expect((await ask("present", { session: "s2" })).ok).toBe(false);
  });
});
