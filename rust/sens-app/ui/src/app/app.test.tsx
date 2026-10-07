import { act, cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { focused } from "../features/panes/store";
import { project } from "../features/project/store";
import { showLanguage } from "../shared/i18n";
import { App } from "./App";
import { dialog } from "./modal";
import { boot, draft, resume, showView } from "./session";
import { arrangements } from "./arrangements";
import { panelShows, readArrangement, shell, showTool } from "./shell";

const ipc = vi.hoisted(() => ({
  commands: new Proxy({} as Record<string, ReturnType<typeof vi.fn>>, {
    get: (known, name: string) => (known[name] ??= vi.fn(async () => undefined)),
  }),
  window: { isMaximized: vi.fn(async () => false), onResized: vi.fn(async () => () => {}), minimize: vi.fn(), toggleMaximize: vi.fn(async () => {}), close: vi.fn() },
}));

vi.mock("../ipc/commands", () => ({ commands: ipc.commands, events: { claudeCode: () => Promise.resolve(() => {}) } }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ipc.window }));

beforeAll(() => {
  vi.stubGlobal("ResizeObserver", class { observe() {} disconnect() {} });
  vi.stubGlobal("IntersectionObserver", class { observe() {} disconnect() {} });
  HTMLDialogElement.prototype.showModal = function () {
    this.open = true;
  };
  HTMLDialogElement.prototype.close = function () {
    this.open = false;
    this.dispatchEvent(new Event("close"));
  };
});

beforeEach(() => {
  localStorage.clear();
  shell.setState(shell.getInitialState(), true);
  arrangements.setState({ kept: [], before: null });
  dialog.setState(dialog.getInitialState(), true);
  project.setState({ root: "", work: "", session: "", view: "", touched: new Map() });
  focused().desk.setState({ root: "", session: "" });
  ipc.commands.workspaces.mockResolvedValue([]);
  ipc.commands.replay.mockResolvedValue([{ kind: "task", at: 1, text: "Hola", files: [], images: [] }]);
  ipc.commands.chatTasks.mockResolvedValue([]);
  ipc.commands.chatBusy.mockResolvedValue(false);
  ipc.commands.folder.mockResolvedValue([]);
});

afterEach(() => {
  cleanup();
  showLanguage("es");
});

const body = () => document.getElementById("body")!;
const endTabs = () => document.querySelector<HTMLElement>("#dock-end .tool-tabs")!;
const kept = () => JSON.parse(localStorage.getItem("sens.arrangement")!);

describe("the shell", () => {
  it("folds the rail from the title bar and with Ctrl+B, and remembers it", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Ocultar la barra lateral" }));
    expect(body().dataset.rail).toBe("closed");
    expect(document.getElementById("rail")?.hasAttribute("inert")).toBe(true);
    expect(kept().railClosed).toBe(true);
    fireEvent.keyDown(document, { key: "b", ctrlKey: true });
    expect(body().dataset.rail).toBe("open");
  });

  it("folds the rail for a tool in a narrow window, without forgetting it was open", () => {
    render(<App />);
    act(() => shell.setState({ narrow: true }));
    act(() => showTool("changes"));
    expect(body().dataset).toMatchObject({ rail: "closed", end: "open" });
    expect(document.getElementById("rail")?.hasAttribute("inert")).toBe(true);
    act(() => shell.setState({ narrow: false }));
    expect(body().dataset).toMatchObject({ rail: "open", end: "open" });
    act(() => shell.setState({ narrow: true }));
    fireEvent.click(screen.getByRole("button", { name: "Mostrar la barra lateral" }));
    expect(body().dataset).toMatchObject({ rail: "open", end: "closed" });
    expect(kept().railClosed).toBe(false);
  });

  it("opens a tool from the tools menu, and closes the panel", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Herramientas" }));
    fireEvent.click(within(document.getElementById("tool-menu")!).getByRole("menuitemradio", { name: /Cambios/ }));
    expect(body().dataset.end).toBe("open");
    expect(document.querySelector<HTMLElement>('.tool[data-tool="changes"]')?.hidden).toBe(false);
    expect(document.querySelector<HTMLElement>('.tool[data-tool="files"]')?.hidden).toBe(true);
    fireEvent.click(within(endTabs()).getByRole("button", { name: "Cerrar" }));
    expect(body().dataset.end).toBe("closed");
  });

  it("keeps each tool opened as a tab, and closing one shows its neighbour", () => {
    render(<App />);
    act(() => showTool("changes"));
    act(() => showTool("map"));
    act(() => showTool("files"));
    const tabs = () => within(endTabs()).getAllByRole("tab");
    expect(tabs().map((tab) => tab.textContent)).toEqual(["Cambios", "Mapa", "Ficheros"]);
    expect(tabs()[2].getAttribute("aria-selected")).toBe("true");
    fireEvent.click(tabs()[0]);
    expect(shell.getState().shown.end).toBe("changes");
    fireEvent.click(screen.getByRole("button", { name: "Cerrar Cambios" }));
    expect(shell.getState()).toMatchObject({ tabs: ["map", "files"], shown: { end: "map" }, open: { end: true } });
    expect(kept().tabs).toEqual(["map", "files"]);
    fireEvent.click(screen.getByRole("button", { name: "Cerrar Mapa" }));
    fireEvent.click(screen.getByRole("button", { name: "Cerrar Ficheros" }));
    expect(shell.getState()).toMatchObject({ tabs: [], open: { end: false } });
  });

  it("puts a view over the chat, and a new session brings the chat back", async () => {
    render(<App />);
    act(() => showView("artifacts"));
    expect(document.querySelector<HTMLElement>("section.chat")?.hidden).toBe(true);
    expect(document.getElementById("shelf")?.hidden).toBe(false);
    await act(async () => draft("C:/demo"));
    expect(project.getState()).toMatchObject({ root: "C:/demo", session: "", view: "" });
    expect(ipc.commands.remember).toHaveBeenCalledWith("C:/demo");
    expect(document.querySelector<HTMLElement>("section.chat")?.hidden).toBe(false);
  });

  it("opens the last project at start under a view already on screen", async () => {
    ipc.commands.lastProject.mockResolvedValue("C:/demo");
    ipc.commands.news.mockResolvedValue([]);
    project.setState({ view: "news" });
    render(<App />);
    await act(async () => boot());
    expect(project.getState()).toMatchObject({ root: "C:/demo", session: "", view: "news" });
    expect(document.querySelector<HTMLElement>("section.chat")?.hidden).toBe(true);
    expect(document.getElementById("news-view")?.hidden).toBe(false);
  });

  it("opens a saved session of the project", async () => {
    render(<App />);
    await act(async () => resume("C:/demo", "s7"));
    expect(project.getState().session).toBe("s7");
    expect(focused().chat.getState().turns).toMatchObject([{ kind: "you", text: "Hola" }]);
  });

  it("names the project of the session in focus in the title bar", async () => {
    render(<App />);
    expect(document.getElementById("project-title")).toBeNull();
    await act(async () => resume("C:/trabajo/demo", "s7"));
    expect(document.getElementById("project-title")?.textContent).toBe("demo");
    await act(async () => resume("C:\\trabajo\\web\\", "s8"));
    expect(document.getElementById("project-title")?.textContent).toBe("web");
  });

  it("shows a dialog, closes it on Escape's close, and gives the focus back", () => {
    render(<App />);
    const me = screen.getByRole("button", { name: /Sin nombre/ });
    fireEvent.click(me);
    fireEvent.click(screen.getByRole("menuitem", { name: "Atajos de teclado" }));
    const panel = document.getElementById("panel") as HTMLDialogElement;
    expect(panel.open).toBe(true);
    expect(panel.querySelector("table.keys")).toBeTruthy();
    act(() => panel.close());
    expect(dialog.getState().open).toBe(false);
    expect(document.activeElement).toBe(me);
  });

  it("keeps the width a splitter was moved to", () => {
    render(<App />);
    const split = screen.getByRole("separator", { name: "Ancho de la barra lateral" });
    document.getElementById("rail")!.getBoundingClientRect = () => new DOMRect(0, 0, 280, 600);
    fireEvent.keyDown(split, { key: "ArrowRight" });
    expect(shell.getState().sizes["--rail-width"]).toBe(280);
    expect(body().style.getPropertyValue("--rail-width")).toBe("280px");
    fireEvent.doubleClick(split);
    expect(shell.getState().sizes["--rail-width"]).toBeUndefined();
  });

  it("speaks the language chosen", () => {
    showLanguage("en");
    render(<App />);
    expect(screen.getByRole("button", { name: "Hide sidebar" }).title).toBe("Hide sidebar (Ctrl+B)");
    fireEvent.click(screen.getByRole("button", { name: "Tools" }));
    expect(within(document.getElementById("tool-menu")!).getByRole("menuitemradio", { name: /Changes/ }).textContent).toContain("What differs from the last commit");
    expect(screen.getByRole("separator", { name: "Sidebar width" })).toBeTruthy();
  });
});

describe("the arrangement", () => {
  const tabsOf = (dock: string) => [...document.querySelectorAll(`#dock-${dock} [role="tab"]`)].map((tab) => tab.textContent);
  const sheet = () => document.getElementById("arrange-sheet")!;

  it("shows the terminal below while the files stay on the right", () => {
    render(<App />);
    act(() => showTool("files"));
    act(() => showTool("terminal"));
    expect(body().dataset).toMatchObject({ end: "open", bottom: "open", start: "closed" });
    expect(panelShows("files") && panelShows("terminal")).toBe(true);
    expect(screen.getByRole("separator", { name: "Alto del panel inferior" }).getAttribute("aria-orientation")).toBe("horizontal");
  });

  it("moves a tab to another edge with Alt and an arrow, and keeps where it went", () => {
    render(<App />);
    act(() => showTool("changes"));
    act(() => showTool("map"));
    fireEvent.keyDown(within(endTabs()).getByRole("tab", { name: "Mapa" }), { key: "ArrowLeft", altKey: true });
    expect(tabsOf("start")).toEqual(["Mapa"]);
    expect(tabsOf("end")).toEqual(["Cambios"]);
    expect(body().dataset).toMatchObject({ start: "open", end: "open" });
    expect(kept().homes.map).toBe("start");
  });

  it("drops a dragged tab on the edge it is let go over", async () => {
    render(<App />);
    act(() => showTool("files"));
    document.elementFromPoint = () => null;
    body().getBoundingClientRect = () => new DOMRect(0, 0, 1000, 600);
    fireEvent.pointerDown(within(endTabs()).getByRole("tab", { name: "Ficheros" }), { button: 0, clientX: 900, clientY: 10 });
    fireEvent.pointerMove(window, { clientX: 600, clientY: 300 });
    fireEvent.pointerMove(window, { clientX: 500, clientY: 560 });
    expect(document.querySelector('.dock-slot[data-dock="bottom"]')?.classList.contains("snap-slot")).toBe(true);
    fireEvent.pointerUp(window, { clientX: 500, clientY: 560 });
    expect(tabsOf("bottom")).toEqual(["Ficheros"]);
    expect(panelShows("files")).toBe(true);
    expect(document.querySelector(".dock-drops")).toBeNull();
    await act(() => new Promise((done) => setTimeout(done)));
  });

  it("starts from a layout, and undoes it", () => {
    render(<App />);
    fireEvent.keyDown(document, { key: "L", code: "KeyL", ctrlKey: true, shiftKey: true });
    expect(sheet().hidden).toBe(false);
    fireEvent.click(within(sheet()).getByRole("button", { name: "Código" }));
    expect(tabsOf("end")).toEqual(["Ficheros", "Cambios", "Mapa"]);
    expect(tabsOf("bottom")).toEqual(["Terminal", "Segundo plano"]);
    expect(within(sheet()).getByRole("button", { name: "Código" }).getAttribute("aria-pressed")).toBe("true");
    fireEvent.click(within(sheet()).getByRole("button", { name: "Deshacer" }));
    expect(body().dataset).toMatchObject({ end: "closed", bottom: "closed" });
  });

  it("leaves only the conversation, and brings the panels back as they were", () => {
    render(<App />);
    act(() => showTool("changes"));
    fireEvent.keyDown(document, { key: ")", code: "Digit0", ctrlKey: true, shiftKey: true });
    expect(body().dataset.end).toBe("closed");
    expect(screen.queryByRole("button", { name: "Herramientas" })).toBeNull();
    fireEvent.keyDown(document, { key: ")", code: "Digit0", ctrlKey: true, shiftKey: true });
    expect(body().dataset.end).toBe("open");
    expect(panelShows("changes")).toBe(true);
  });

  it("puts the sidebar on the right and the bottom panel across the window", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Disposición" }));
    fireEvent.click(within(sheet()).getByRole("radio", { name: "Derecha" }));
    fireEvent.click(within(sheet()).getByRole("switch", { name: "Panel inferior a todo el ancho" }));
    expect(body().dataset.railSide).toBe("end");
    expect(body().style.gridTemplateAreas).toBe(`"s c e r" "b b b r"`);
    expect(kept()).toMatchObject({ rail: "end", wide: true });
  });

  it("keeps an arrangement by name and brings it back with its keys", () => {
    render(<App />);
    act(() => showTool("web"));
    fireEvent.click(screen.getByRole("button", { name: "Disposición" }));
    fireEvent.click(within(sheet()).getByRole("button", { name: "Guardar esta disposición" }));
    fireEvent.change(within(sheet()).getByRole("textbox", { name: "Nombre" }), { target: { value: "Web" } });
    fireEvent.click(within(sheet()).getByRole("button", { name: "Guardar" }));
    expect(JSON.parse(localStorage.getItem("sens.arrangements")!)).toMatchObject([{ name: "Web", arrangement: { shown: { end: "web" } } }]);
    act(() => showTool("terminal"));
    fireEvent.keyDown(document, { key: "!", code: "Digit1", ctrlKey: true, shiftKey: true });
    expect(body().dataset).toMatchObject({ end: "open", bottom: "closed" });
    expect(panelShows("web")).toBe(true);
  });

  it("reads what an older version kept", () => {
    const arrangement = readArrangement({ railClosed: true, tabs: ["map", "nope", "map"], sizes: { "--end-width": 500, "--x": -1 }, shown: { end: "files" }, open: { end: true } });
    expect(arrangement).toMatchObject({ railClosed: true, tabs: ["map"], sizes: { "--end-width": 500 }, shown: { end: "map", bottom: null }, open: { end: true } });
  });
});
