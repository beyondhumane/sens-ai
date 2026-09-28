// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import type { Card, VoiceHeard } from "../../ipc/types";
import { dialog } from "../../app/modal";
import { chooseFolder } from "../../app/session";
import { blank, warm as warmChat } from "../chat/store";
import { focused } from "../panes/store";
import { accountLine, choose, loadCatalog, models, noteLimits, noteLockout, readAccount } from "../models/store";
import { project } from "../project/store";
import { settingsSheet } from "../settings/sheet";
import { settings } from "../settings/store";
import { voice } from "../voice/store";
import { Composer } from "./Composer";
import { BYPASS, composer, currentSettings, pickEffort, readRepo, readTrust, switchTo, toggleThinking } from "./store";

const ipc = vi.hoisted(() => ({
  commands: {
    providers: vi.fn(),
    models: vi.fn(),
    claudeAccount: vi.fn(),
    claudeCodeNewer: vi.fn(),
    repo: vi.fn(),
    checkout: vi.fn(),
    openSession: vi.fn(),
    chatSend: vi.fn(),
    chatStop: vi.fn(),
    chatWarm: vi.fn(),
    newSessionId: vi.fn(),
    workspaces: vi.fn(),
    folder: vi.fn(),
    trustProject: vi.fn(),
    projectTrusted: vi.fn(),
    findFiles: vi.fn(),
    voiceStart: vi.fn(),
    voiceStop: vi.fn(),
    voicePrepare: vi.fn(),
    providersState: vi.fn(),
    providerSignIn: vi.fn(),
  },
  listeners: new Set<(what: VoiceHeard) => void>(),
}));

vi.mock("../../ipc/commands", () => ({
  commands: ipc.commands,
  events: {
    claudeCode: () => Promise.resolve(() => {}),
    voice: (hear: (what: VoiceHeard) => void) => {
      ipc.listeners.add(hear);
      return Promise.resolve(() => void ipc.listeners.delete(hear));
    },
  },
}));
vi.mock("../../app/session", () => ({ resume: vi.fn(), draft: vi.fn(async () => {}), fresh: vi.fn(), chooseFolder: vi.fn(), showView: vi.fn() }));

const card = (id: string, over: Partial<Card> = {}): Card => ({
  id,
  label: id.replace("claude-", "").replace(/^\w/, (first) => first.toUpperCase()),
  description: "Best for everyday, complex tasks",
  latest: true,
  efforts: ["low", "medium", "high", "max"],
  effort: "medium",
  thinking: "toggle",
  ...over,
});

beforeAll(() => {
  vi.stubGlobal("ResizeObserver", class { observe() {} disconnect() {} });
});

beforeEach(async () => {
  localStorage.clear();
  for (const command of Object.values(ipc.commands)) command.mockReset().mockResolvedValue(undefined);
  ipc.listeners.clear();
  voice.setState({ ready: true, fetching: false, done: 0, total: 59_707_625, fault: "" });
  ipc.commands.providers.mockResolvedValue([{ id: "claude", vendor: "Anthropic", label: "Claude Code" }]);
  ipc.commands.models.mockResolvedValue([card("claude-sonnet"), card("claude-opus", { thinking: "always" }), card("claude-haiku", { latest: false, efforts: [] })]);
  ipc.commands.claudeAccount.mockResolvedValue({ billing: "subscription", plan: "max", source: "claude.ai", email: "ada@example.com" });
  ipc.commands.openSession.mockResolvedValue("s1");
  ipc.commands.workspaces.mockResolvedValue([]);
  ipc.commands.folder.mockResolvedValue([]);
  models.setState(models.getInitialState(), true);
  models.setState({ known: {}, hidden: new Set() });
  composer.setState(composer.getInitialState(), true);
  dialog.setState(dialog.getInitialState(), true);
  project.setState({ view: "", touched: new Map() });
  focused().chat.setState(focused().chat.getInitialState(), true);
  focused().desk.setState(focused().desk.getInitialState(), true);
  focused().desk.setState({ root: "C:/demo" });
  blank("");
  await loadCatalog();
  await act(async () => new Promise((settle) => setTimeout(settle)));
});

afterEach(cleanup);

const button = (name: string | RegExp) => screen.getByRole("button", { name });
const field = () => screen.getByRole("textbox", { name: "Mensaje para Claude" });

describe("the composer", () => {
  it("picks the first model offered, and knows its account", () => {
    render(<Composer />);
    expect(document.getElementById("crew")?.textContent).toBe("Sonnet");
    expect(currentSettings()).toEqual({ provider: "claude", model: "claude-sonnet", effort: "medium", thinking: true, mode: "default" });
    act(() => noteLimits({ kind: "limits", status: "allowed", window: "", utilization: null, resetsAt: null, threshold: null, overage: null, windows: { five_hour: { utilization: 0.42, resetsAt: null } } }));
    expect(accountLine()).toEqual({ text: "Suscripción Max · claude.ai · ada@example.com · 42 % usado en 5 h", warn: false });
  });

  it("sends what is written with what is attached, and stops Claude while it works", async () => {
    render(<Composer />);
    expect((button("Enviar") as HTMLButtonElement).disabled).toBe(true);
    act(() => focused().desk.setState({ attached: [{ path: "src/app.ts", name: "app.ts", bytes: 2048, outside: false }] }));
    expect(screen.getByText("app.ts", { selector: ".clip-head" })).toBeTruthy();
    expect(screen.getByText("TS · 2 KB", { selector: ".clip-meta" })).toBeTruthy();
    fireEvent.change(field(), { target: { value: "  Revisa esto  " } });
    await act(async () => fireEvent.keyDown(field(), { key: "Enter" }));
    expect(ipc.commands.chatSend).toHaveBeenCalledWith("C:/demo", "s1", { text: "Revisa esto", files: ["src/app.ts"], images: [] }, currentSettings());
    expect((field() as HTMLTextAreaElement).value).toBe("");
    expect(document.querySelector(".clips")).toBeNull();

    expect(document.querySelector(".box")?.getAttribute("data-busy")).toBe("true");
    await act(async () => fireEvent.click(button("Parar")));
    expect(ipc.commands.chatStop).toHaveBeenCalledWith("s1");
    expect(button("Parando…")).toHaveProperty("disabled", true);
    act(() => focused().chat.setState({ busy: false, stopping: false }));
  });

  it("keeps what is written when Enter comes while Claude works, and sends it once the turn ends", async () => {
    act(() => focused().chat.setState({ busy: true }));
    render(<Composer />);
    fireEvent.change(field(), { target: { value: "Y luego esto" } });
    await act(async () => fireEvent.keyDown(field(), { key: "Enter" }));
    expect(ipc.commands.chatSend).not.toHaveBeenCalled();
    expect((field() as HTMLTextAreaElement).value).toBe("Y luego esto");

    act(() => focused().chat.setState({ busy: false }));
    await act(async () => fireEvent.keyDown(field(), { key: "Enter" }));
    expect(ipc.commands.chatSend).toHaveBeenCalledWith("C:/demo", "s1", { text: "Y luego esto", files: [], images: [] }, currentSettings());
    expect((field() as HTMLTextAreaElement).value).toBe("");
    act(() => focused().chat.setState({ busy: false, stopping: false }));
  });

  it("drops an attached file, and asks for a folder from its chip", () => {
    act(() => focused().desk.setState({ attached: [{ path: "C:/fuera/plan.pdf", name: "plan.pdf", bytes: 10, outside: true }] }));
    render(<Composer />);
    fireEvent.click(button("Quitar plan.pdf"));
    expect(focused().desk.getState().attached).toEqual([]);
    fireEvent.click(button(/demo/));
    expect(chooseFolder).toHaveBeenCalled();
  });

  it("chooses a model, and hides models while editing the list", () => {
    render(<Composer />);
    fireEvent.click(button(/Sonnet/));
    const picker = document.getElementById("picker")!;
    expect([...picker.querySelectorAll(".menu-head")].map((head) => head.textContent)).toEqual(["Anthropic · Claude Code", "Anteriores"]);
    fireEvent.click(within(picker).getByRole("menuitemradio", { name: /Opus/ }));
    expect(document.getElementById("crew")?.textContent).toBe("Opus");
    expect(button("Razonamiento").getAttribute("aria-disabled")).toBe("true");

    fireEvent.click(button(/Opus/));
    fireEvent.click(within(picker).getByRole("menuitem", { name: "Editar modelos…" }));
    fireEvent.click(within(picker).getByRole("menuitemcheckbox", { name: "Opus" }));
    expect(models.getState().hidden.has("claude-opus")).toBe(true);
    expect(document.getElementById("crew")?.textContent).toBe("Sonnet");
  });

  it("describes a model by its tagline, in Spanish, and spins while asking for models", () => {
    const offered = [
      card("claude-opus-5-5[1m]", { label: "Opus 5.5 (1M)", description: "Opus 5.5 with 1M context · Best for everyday, complex tasks" }),
      card("claude-opus-4-8", { label: "Opus 4.8", description: "", latest: false }),
    ];
    act(() => {
      models.setState({ known: { claude: offered } });
      choose("claude", "claude-opus-5-5[1m]");
    });
    render(<Composer />);
    fireEvent.click(button(/Opus 5\.5/));
    const picker = document.getElementById("picker")!;
    expect([...picker.querySelectorAll(".mode-sub")].map((said) => said.textContent)).toEqual(["El mejor para el trabajo complejo de cada día"]);

    act(() => models.setState({ fetching: true }));
    const refresh = document.getElementById("models-refresh")!;
    expect(refresh.getAttribute("aria-busy")).toBe("true");
    expect(refresh.textContent).toBe("Actualizando…");
  });

  it("asks to trust the folder before Sin control, and Sin control acts only in that folder", async () => {
    render(<Composer />);
    fireEvent.click(button(/Preguntar/));
    const sheet = document.getElementById("mode-sheet")!;
    fireEvent.click(within(sheet).getByRole("menuitemradio", { name: /Sin control/ }));
    expect(dialog.getState().title).toBe("¿Confías en este proyecto?");
    expect(currentSettings().mode).toBe("default");

    render(<>{dialog.getState().content}</>);
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Confiar y activar" })));
    expect(ipc.commands.trustProject).toHaveBeenCalledWith("C:/demo", true);
    expect(currentSettings().mode).toBe(BYPASS);
    expect(document.getElementById("mode-label")?.textContent).toBe("Sin control");
    fireEvent.click(button(/Sin control/));
    expect(within(sheet).getByRole("menuitemradio", { checked: true }).textContent).toMatch(/^Sin control/);

    act(() => focused().desk.setState({ root: "C:/otra" }));
    expect(currentSettings().mode).toBe("default");
    expect(document.getElementById("mode-label")?.textContent).toBe("Preguntar");
  });

  it("switches to Sin control without asking in a folder already trusted", async () => {
    ipc.commands.projectTrusted.mockResolvedValue(true);
    await act(async () => readTrust());
    expect(ipc.commands.projectTrusted).toHaveBeenCalledWith("C:/demo");
    render(<Composer />);
    fireEvent.click(button(/Preguntar/));
    fireEvent.click(within(document.getElementById("mode-sheet")!).getByRole("menuitemradio", { name: /Sin control/ }));
    expect(dialog.getState().open).toBe(false);
    expect(currentSettings().mode).toBe(BYPASS);
  });

  it("points to the newer Claude Code from the picker only when there is one", async () => {
    render(<Composer />);
    fireEvent.click(button(/Sonnet/));
    const picker = document.getElementById("picker")!;
    expect(document.getElementById("models-update")?.hidden).toBe(true);

    ipc.commands.claudeCodeNewer.mockResolvedValue("2.1.281");
    await act(async () => loadCatalog());
    const update = within(picker).getByRole("menuitem", { name: "Actualizar Claude Code…" });
    expect(update.title).toBe("Hay una versión nueva: v2.1.281");
    fireEvent.click(update);
    expect(settingsSheet.getState().open).toBe(true);
    expect(settings.getState().section).toBe("providers");
  });

  it("sets the permission mode, thinking and effort for the next turn", async () => {
    ipc.commands.projectTrusted.mockResolvedValue(true);
    await act(async () => readTrust());
    render(<Composer />);
    fireEvent.click(button(/Preguntar/));
    fireEvent.click(within(document.getElementById("mode-sheet")!).getByRole("menuitemradio", { name: /Sin control/ }));
    expect(document.getElementById("mode-pick")?.dataset.risky).toBe("true");
    expect(currentSettings().mode).toBe("bypassPermissions");

    fireEvent.click(button("Razonamiento"));
    expect(currentSettings().thinking).toBe(false);

    fireEvent.click(button(/Medio/));
    const track = screen.getByRole("slider", { name: "Esfuerzo" });
    fireEvent.keyDown(track, { key: "End" });
    expect(document.getElementById("effort")?.dataset.max).toBe("true");
    expect(currentSettings().effort).toBe("max");
    fireEvent.keyDown(track, { key: "ArrowLeft" });
    expect(track.getAttribute("aria-valuetext")).toBe("Alto");
  });

  it("shows the branch, and switches to another", async () => {
    ipc.commands.repo.mockResolvedValue({ branch: "main", detached: false, dirty: 2, branches: ["main", "feat/ui", "fix/css"] });
    ipc.commands.checkout.mockResolvedValue({ branch: "feat/ui", detached: false, dirty: 0, branches: ["main", "feat/ui", "fix/css"] });
    await act(async () => readRepo());
    render(<Composer />);
    expect(button(/main/).title).toBe("main · 2 ficheros sin confirmar");
    fireEvent.click(button(/main/));
    fireEvent.change(screen.getByPlaceholderText("Buscar ramas…"), { target: { value: "feat" } });
    expect([...document.querySelectorAll("#branch-rows .name")].map((name) => name.textContent)).toEqual(["feat/ui"]);
    await act(async () => fireEvent.click(button("feat/ui")));
    expect(ipc.commands.checkout).toHaveBeenCalledWith("C:/demo", "feat/ui");
    expect(document.getElementById("branch-name")?.textContent).toBe("feat/ui");
    expect(focused().chat.getState().turns.at(-1)).toMatchObject({ kind: "notice", parts: ["rama · ", { bold: "feat/ui" }] });
  });

  it("keeps the branch switched to when a read from before the switch answers after it", async () => {
    let before!: (repo: unknown) => void;
    ipc.commands.repo.mockReturnValueOnce(new Promise((settle) => (before = settle)));
    ipc.commands.checkout.mockResolvedValue({ branch: "feat/ui", detached: false, dirty: 0, branches: ["main", "feat/ui"] });
    let reading!: Promise<void>;
    act(() => void (reading = readRepo()));
    await act(async () => switchTo("feat/ui"));
    before({ branch: "main", detached: false, dirty: 0, branches: ["main", "feat/ui"] });
    await act(async () => reading);

    expect(focused().desk.getState().repo?.branch).toBe("feat/ui");
  });
});

describe("a lost sign-in", () => {
  const claudeCode = { id: "claude", vendor: "Anthropic", label: "Claude Code", method: "subscription", keyHint: "", version: "2.1.0", account: null, error: "", installed: true };

  it("asks to sign in above the message, and goes once Claude Code is signed in again", async () => {
    render(<Composer />);
    expect(screen.queryByRole("alert")).toBeNull();
    act(() => noteLockout("signIn"));
    const notice = screen.getByRole("alert");
    expect(notice.textContent).toContain("Tu sesión de Claude se ha cerrado");
    expect(notice.nextElementSibling?.className).toBe("workspace");
    ipc.commands.providersState.mockResolvedValue([claudeCode]);
    await act(async () => fireEvent.click(within(notice).getByRole("button", { name: "Iniciar sesión" })));
    await act(async () => new Promise((settle) => setTimeout(settle)));
    expect(ipc.commands.providerSignIn).toHaveBeenCalledWith("subscription");
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("stays while the sign-in is not finished", async () => {
    render(<Composer />);
    act(() => noteLockout("billing"));
    expect(screen.getByRole("alert").textContent).toContain("Claude no puede usar tu suscripción");
    ipc.commands.providersState.mockResolvedValue([claudeCode]);
    ipc.commands.providerSignIn.mockRejectedValue("no terminaste el inicio de sesión");
    await act(async () => fireEvent.click(button("Iniciar sesión")));
    await act(async () => new Promise((settle) => setTimeout(settle)));
    expect(screen.getByRole("alert").textContent).toContain("Claude no puede usar tu suscripción");
  });

  it("shows while Claude Code is signed out, and sends the trouble of a key to Providers", async () => {
    ipc.commands.claudeAccount.mockResolvedValue({ billing: "signedOut", plan: "", source: "", email: "" });
    await readAccount();
    render(<Composer />);
    expect(screen.getByRole("alert").textContent).toContain("Tu sesión de Claude se ha cerrado");
    act(() => models.setState({ account: { billing: "elsewhere", plan: "", source: "ANTHROPIC_API_KEY", email: "" }, lockout: "billing" }));
    expect(screen.getByRole("alert").textContent).toContain("La cuenta de la clave de API no tiene saldo");
    await act(async () => fireEvent.click(button("Abrir Proveedores")));
    expect(settingsSheet.getState().open).toBe(true);
    expect(settings.getState().section).toBe("providers");
    act(() => settingsSheet.setState({ open: false }));
  });
});

describe("a change mid-session that reads the conversation again", () => {
  const ranOn = (model: string, ranWith: { effort: string; thinking: boolean } | null = { effort: "medium", thinking: true }) =>
    act(() => focused().chat.setState({ ranOn: model, ranWith, context: { used: 31_400, window: 200_000 } }));

  it("names another model, and goes back to the model of the session", () => {
    render(<Composer />);
    ranOn("claude-sonnet");
    expect(screen.queryByRole("status")).toBeNull();
    act(() => choose("claude", "claude-opus"));
    const notice = screen.getByRole("status");
    expect(notice.textContent).toContain("Cambiar de modelo a mitad de sesión gasta más de tu plan");
    expect(notice.textContent).toContain("Esta sesión iba con Sonnet. Con Opus, el próximo mensaje vuelve a leer toda la conversación (31,4k tokens) sin caché");
    fireEvent.click(within(notice).getByRole("button", { name: "Deshacer" }));
    expect(focused().desk.getState().choice.model).toBe("claude-sonnet");
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("names thinking and effort changed together, and undoes both", () => {
    render(<Composer />);
    ranOn("claude-sonnet");
    act(() => toggleThinking());
    expect(screen.getByRole("status").textContent).toContain("Cambiar el razonamiento a mitad de sesión gasta más de tu plan");
    expect(screen.getByRole("status").textContent).toContain("Esta sesión iba con razonamiento activado. Con razonamiento desactivado,");
    act(() => pickEffort(2));
    const notice = screen.getByRole("status");
    expect(notice.textContent).toContain("Estos cambios a mitad de sesión gastan más de tu plan");
    expect(notice.textContent).toContain("Esta sesión iba con razonamiento activado y esfuerzo medio. Con razonamiento desactivado y esfuerzo alto,");
    fireEvent.click(within(notice).getByRole("button", { name: "Deshacer" }));
    expect(currentSettings()).toMatchObject({ model: "claude-sonnet", effort: "medium", thinking: true });
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("knows what the last message went with, and warns when the effort changes after it", async () => {
    render(<Composer />);
    fireEvent.change(field(), { target: { value: "Hola" } });
    await act(async () => fireEvent.keyDown(field(), { key: "Enter" }));
    act(() => focused().chat.setState({ busy: false, stopping: false }));
    expect(screen.queryByRole("status")).toBeNull();
    act(() => pickEffort(0));
    expect(screen.getByRole("status").textContent).toContain("Cambiar el esfuerzo a mitad de sesión gasta más de tu plan");
  });

  it("recommends a new session, which keeps the model chosen", async () => {
    const { draft } = await import("../../app/session");
    render(<Composer />);
    ranOn("claude-sonnet");
    act(() => choose("claude", "claude-opus"));
    fireEvent.click(button("Sesión nueva"));
    expect(draft).toHaveBeenCalledWith("C:/demo", focused());
    act(() => blank(""));
    expect(screen.queryByRole("status")).toBeNull();
    expect(focused().desk.getState().choice.model).toBe("claude-opus");
  });

  it("stays away for a new session, the same model with another context window, and knobs of a session only read back", () => {
    render(<Composer />);
    act(() => choose("claude", "claude-opus"));
    expect(screen.queryByRole("status")).toBeNull();
    ranOn("claude-opus[1m]");
    expect(screen.queryByRole("status")).toBeNull();
    act(() => choose("claude", "claude-sonnet"));
    ranOn("claude-sonnet", null);
    act(() => pickEffort(0));
    expect(screen.queryByRole("status")).toBeNull();
  });
});

describe("the height of the message", () => {
  it("is measured only while the chat shows, and again when it shows up", () => {
    const watchers: (() => void)[] = [];
    vi.stubGlobal("ResizeObserver", class { constructor(then: () => void) { watchers.push(then); } observe() {} disconnect() {} });
    let shown = false;
    vi.spyOn(HTMLElement.prototype, "getClientRects").mockImplementation(() => (shown ? [{}] : []) as unknown as DOMRectList);
    Object.defineProperty(HTMLTextAreaElement.prototype, "scrollHeight", { configurable: true, get: () => 52 });
    Object.defineProperty(HTMLTextAreaElement.prototype, "clientWidth", { configurable: true, get: () => (shown ? 600 : 0) });
    try {
      render(<Composer />);
      act(() => focused().desk.setState({ text: "Una línea\ny otra" }));
      expect((field() as HTMLTextAreaElement).style.height).toBe("");
      shown = true;
      act(() => watchers.forEach((then) => then()));
      expect((field() as HTMLTextAreaElement).style.height).toBe("52px");
    } finally {
      vi.restoreAllMocks();
      delete (HTMLTextAreaElement.prototype as { scrollHeight?: number }).scrollHeight;
      delete (HTMLTextAreaElement.prototype as { clientWidth?: number }).clientWidth;
      vi.stubGlobal("ResizeObserver", class { observe() {} disconnect() {} });
    }
  });
});

describe("suggestions while writing", () => {
  const offered = [
    { name: "compact", description: "Resume la conversación", hint: "<instrucciones>" },
    { name: "context", description: "Enseña el contexto", hint: "" },
    { name: "frontend-design", description: "Interfaces con carácter", hint: "" },
  ];
  const write = (value: string) => {
    fireEvent.focus(field());
    fireEvent.change(field(), { target: { value } });
  };
  const options = () => screen.queryAllByRole("option").map((option) => option.querySelector(".suggest-name")?.textContent);

  beforeEach(() => focused().desk.setState({ slashes: offered }));

  it("offers the commands of Claude Code for a message that starts with a slash, and writes the one picked", () => {
    render(<Composer />);
    write("/co");
    expect(options()).toEqual(["/compact", "/context"]);
    expect(field().getAttribute("aria-activedescendant")).toBe(screen.getAllByRole("option")[0].id);

    fireEvent.keyDown(field(), { key: "ArrowDown" });
    fireEvent.keyDown(field(), { key: "Enter" });
    expect(focused().desk.getState().text).toBe("/context ");
    expect(ipc.commands.chatSend).not.toHaveBeenCalled();
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  it("offers the files of the project after an @, and a click writes the mention", async () => {
    ipc.commands.findFiles.mockResolvedValue([
      { name: "App.tsx", path: "src/App.tsx", dir: false, ignored: false },
      { name: "app-notes.md", path: "docs/app-notes.md", dir: false, ignored: false },
    ]);
    render(<Composer />);
    write("mira @app");
    await act(async () => new Promise((settle) => setTimeout(settle, 200)));

    expect(ipc.commands.findFiles).toHaveBeenLastCalledWith("C:/demo", "app");
    expect(options()).toEqual(["src/App.tsx", "docs/app-notes.md"]);
    fireEvent.click(screen.getAllByRole("option")[1]);
    expect(focused().desk.getState().text).toBe("mira @docs/app-notes.md ");
  });

  it("closes with Escape until what is written changes", () => {
    render(<Composer />);
    write("/co");
    fireEvent.keyDown(field(), { key: "Escape" });
    expect(screen.queryByRole("listbox")).toBeNull();
    write("/com");
    expect(options()).toEqual(["/compact"]);
  });

  it("mentions the file in the project folder while the session's worktree is still to be made", async () => {
    focused().desk.setState({ isolate: true, repo: { branch: "main", detached: false, dirty: 0, branches: ["main"] } });
    ipc.commands.findFiles.mockResolvedValue([{ name: "spec.md", path: "docs/spec.md", dir: false, ignored: false }]);
    render(<Composer />);
    write("lee @spec");
    await act(async () => new Promise((settle) => setTimeout(settle, 200)));
    fireEvent.click(screen.getAllByRole("option")[0]);
    expect(focused().desk.getState().text).toBe("lee @C:/demo/docs/spec.md ");
  });

  it("asks Claude Code for its commands again when a start brought none", async () => {
    const settings = { provider: "claude", model: "claude-sonnet", effort: "", thinking: true, mode: "default" };
    ipc.commands.newSessionId.mockResolvedValue("s-new");
    ipc.commands.chatWarm.mockResolvedValueOnce([]).mockResolvedValue(offered);
    focused().desk.setState({ slashes: [] });
    await warmChat(settings);
    expect(focused().desk.getState().slashes).toEqual([]);
    await warmChat(settings);
    await warmChat(settings);
    expect(ipc.commands.chatWarm).toHaveBeenCalledTimes(2);
    expect(focused().desk.getState().slashes).toEqual(offered);
  });

  it("stays shut while the message is not being written", () => {
    render(<Composer />);
    write("/co");
    fireEvent.blur(field());
    expect(screen.queryByRole("listbox")).toBeNull();
  });
});

describe("the context meter", () => {
  it("stays away until a turn says how full the context is", () => {
    render(<Composer />);
    expect(screen.queryByRole("button", { name: /Contexto usado/ })).toBeNull();
  });

  it("shows how full it is, warns near the end, and compacts without taking what is attached", async () => {
    focused().desk.setState({ session: "s1", attached: [{ path: "a.ts", name: "a.ts", bytes: 1, outside: false }] });
    focused().chat.setState({ context: { used: 185_000, window: 200_000 } });
    render(<Composer />);
    const meter = button("Contexto usado: 93 %");
    expect(meter.closest(".meter")?.getAttribute("data-level")).toBe("full");
    expect(meter.title).toBe("Contexto: 185k de 200k tokens");

    fireEvent.click(meter);
    await act(async () => fireEvent.click(button("Compactar ahora")));
    expect(ipc.commands.chatSend).toHaveBeenCalledWith("C:/demo", "s1", { text: "/compact", files: [], images: [] }, expect.anything());
    expect(focused().desk.getState().attached).toHaveLength(1);
  });
});

describe("dictation", () => {
  const hear = (what: VoiceHeard) => act(() => [...ipc.listeners].forEach((listener) => listener(what)));
  const written = () => (field() as HTMLTextAreaElement).value;
  const lastNotice = () => focused().chat.getState().turns.at(-1);

  it("writes what the microphone of Sens hears, phrase by phrase, and waits for the last one after stopping", async () => {
    ipc.commands.voiceStart.mockResolvedValue(5);
    render(<Composer />);
    fireEvent.change(field(), { target: { value: "Primero " } });
    await act(async () => fireEvent.click(button("Dictar")));
    expect(ipc.commands.voiceStart).toHaveBeenCalledWith("es");
    const microphone = button("Parar el dictado");
    expect(microphone.getAttribute("aria-pressed")).toBe("true");

    hear({ kind: "level", id: 5, level: 0.6 });
    expect(microphone.style.getPropertyValue("--level")).toBe("0.6");
    hear({ kind: "guess", id: 5, text: "Añade un test para el formu" });
    expect(written()).toBe("Primero Añade un test para el formu");
    hear({ kind: "guess", id: 5, text: "Añade un test para el formulario de" });
    expect(written()).toBe("Primero Añade un test para el formulario de");
    hear({ kind: "phrase", id: 5, text: "Añade un test para el formulario de contacto." });
    expect(written()).toBe("Primero Añade un test para el formulario de contacto.");
    hear({ kind: "phrase", id: 4, text: "de otra vez" });
    expect(written()).toBe("Primero Añade un test para el formulario de contacto.");

    await act(async () => fireEvent.click(microphone));
    expect(ipc.commands.voiceStop).toHaveBeenCalled();
    expect(button("Transcribiendo…").getAttribute("aria-busy")).toBe("true");
    hear({ kind: "phrase", id: 5, text: "Revisa el componente del botón." });
    hear({ kind: "ended", id: 5, refusal: null });
    expect(written()).toBe("Primero Añade un test para el formulario de contacto. Revisa el componente del botón.");
    expect(button("Dictar").getAttribute("aria-pressed")).toBe("false");
    expect(document.activeElement).toBe(field());
  });

  it("says the voice model is on its way instead of listening before it arrives", async () => {
    voice.setState({ ready: false, fetching: false });
    render(<Composer />);
    await act(async () => fireEvent.click(button("Dictar")));
    expect(ipc.commands.voicePrepare).toHaveBeenCalled();
    expect(ipc.commands.voiceStart).not.toHaveBeenCalled();
    expect(lastNotice()).toMatchObject({ kind: "notice", tone: "warn", parts: [expect.stringContaining("una sola vez")] });

    act(() => voice.setState({ fetching: true, done: 30, total: 100 }));
    const preparing = button("Preparando el modelo de voz · 30 %");
    await act(async () => fireEvent.click(preparing));
    expect(lastNotice()).toMatchObject({ parts: ["Preparando el modelo de voz · 30 %"] });
    expect(ipc.commands.voiceStart).not.toHaveBeenCalled();
  });

  it("says why the microphone refused, and where another one is chosen", async () => {
    ipc.commands.voiceStart.mockRejectedValue({ cause: "microphone", message: "Sens no puede escuchar el micrófono: dispositivo no disponible" });
    render(<Composer />);
    await act(async () => fireEvent.click(button("Dictar")));
    expect(lastNotice()).toMatchObject({
      kind: "notice",
      tone: "warn",
      parts: ["Sens no puede escuchar el micrófono: dispositivo no disponible · elige el micrófono en Ajustes › General › Voz"],
    });
    expect(button("Dictar").getAttribute("aria-pressed")).toBe("false");
  });

  it("stops dictating when the message is sent, and writes nothing more into the next one", async () => {
    ipc.commands.voiceStart.mockResolvedValue(9);
    render(<Composer />);
    await act(async () => fireEvent.click(button("Dictar")));
    hear({ kind: "phrase", id: 9, text: "Añade un test" });
    await act(async () => fireEvent.keyDown(field(), { key: "Enter" }));
    expect(ipc.commands.chatSend).toHaveBeenCalledWith("C:/demo", "s1", { text: "Añade un test", files: [], images: [] }, currentSettings());
    expect(ipc.commands.voiceStop).toHaveBeenCalled();

    hear({ kind: "phrase", id: 9, text: "y otro" });
    hear({ kind: "ended", id: 9, refusal: null });
    expect(written()).toBe("");
    act(() => focused().chat.setState({ busy: false, stopping: false }));
  });
});

describe("a worktree for a new session", () => {
  const repo = { branch: "main", detached: false, dirty: 0, branches: ["main"] };

  it("is offered only to a new session of a repository, and the choice is kept", () => {
    render(<Composer />);
    expect(screen.queryByRole("button", { name: "Worktree" })).toBeNull();

    act(() => focused().desk.setState({ repo }));
    const toggle = button("Worktree");
    expect(toggle.getAttribute("aria-pressed")).toBe("false");
    fireEvent.click(toggle);
    expect(toggle.getAttribute("aria-pressed")).toBe("true");
    expect(focused().desk.getState().isolate).toBe(true);
    expect(localStorage.getItem("sens.isolate")).toBe("true");

    act(() => focused().desk.setState({ session: "s1" }));
    expect(screen.queryByRole("button", { name: "Worktree" })).toBeNull();
  });

  it("names the branch and where it came from once the session works in one", () => {
    focused().desk.setState({ repo, session: "s1", worktree: { path: "C:/demo/.sens/worktrees/ab12cd34", branch: "sens/ab12cd34", base: "main" } });
    render(<Composer />);
    const label = document.getElementById("worktree")!;
    expect(label.textContent).toBe("worktree");
    expect(label.title).toBe("Trabaja en un worktree aparte, en la rama sens/ab12cd34 (creada desde main): C:/demo/.sens/worktrees/ab12cd34");
  });
});
