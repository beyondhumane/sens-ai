import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import type { ClaudeCodeProgress, ProviderState, VoiceHeard } from "../../ipc/types";
import { dialog } from "../../app/modal";
import { models, readAccount, refreshModels } from "../models/store";
import { project } from "../project/store";
import { profile } from "../profile/store";
import { languageNow, showLanguage } from "../../shared/i18n";
import { look } from "../../shared/look";
import { updates } from "../updates/store";
import { voice } from "../voice/store";
import { Settings, SettingsDialog } from "./Settings";
import { settingsSheet } from "./sheet";
import { enterSettings, openSettings, settings, showSection, type Section } from "./store";

const ipc = vi.hoisted(() => ({
  commands: {
    profile: vi.fn(),
    saveProfile: vi.fn(),
    setUpdateCheck: vi.fn(),
    setNotify: vi.fn(),
    setKeepInTray: vi.fn(),
    setStartWithWindows: vi.fn(),
    shortcutState: vi.fn(),
    shortcutSet: vi.fn(),
    shortcutPause: vi.fn(),
    voiceMicrophones: vi.fn(),
    voiceMicrophone: vi.fn(),
    voiceChoose: vi.fn(),
    voiceTest: vi.fn(),
    voiceStop: vi.fn(),
    voicePrepare: vi.fn(),
    updateCheck: vi.fn(),
    providersState: vi.fn(),
    setProviderMethod: vi.fn(),
    saveApiKey: vi.fn(),
    forgetApiKey: vi.fn(),
    providerSignIn: vi.fn(),
    providerSignOut: vi.fn(),
    claudeCodeInstall: vi.fn(),
    claudeCodeNewer: vi.fn(),
    claudeCodeUpdate: vi.fn(),
    setLook: vi.fn(),
    setLanguage: vi.fn(),
    news: vi.fn(),
  },
  heard: { claudeCode: (_: ClaudeCodeProgress) => {}, voice: (_: VoiceHeard) => {} },
}));

vi.mock(import("../models/store"), async (original) => ({ ...(await original()), readAccount: vi.fn(async () => null), refreshModels: vi.fn(async () => {}) }));

vi.mock("../../ipc/commands", () => ({
  commands: ipc.commands,
  events: {
    claudeCode: (heard: (progress: ClaudeCodeProgress) => void) => {
      ipc.heard.claudeCode = heard;
      return Promise.resolve(() => {});
    },
    voice: (heard: (what: VoiceHeard) => void) => {
      ipc.heard.voice = heard;
      return Promise.resolve(() => {});
    },
  },
}));

const claude = (over: Partial<ProviderState> = {}): ProviderState => ({
  id: "claude",
  vendor: "Anthropic",
  label: "Claude Code",
  method: "subscription",
  keyHint: "",
  version: "2.1.0",
  account: { billing: "subscription", plan: "max", source: "claude.ai", email: "demo@example.com" },
  error: "",
  installed: true,
  ...over,
});

function later<T = void>() {
  let settle!: (value: T) => void;
  const promise = new Promise<T>((resolve) => (settle = resolve));
  return { promise, settle };
}

async function open(section: Section) {
  render(<Settings />);
  await act(async () => {
    showSection(section);
    enterSettings();
  });
}

beforeAll(() => {
  HTMLDialogElement.prototype.showModal = function () {
    this.open = true;
  };
  HTMLDialogElement.prototype.close = function () {
    this.open = false;
    this.dispatchEvent(new Event("close"));
  };
});

beforeEach(() => {
  settings.setState(settings.getInitialState(), true);
  profile.setState(profile.getInitialState(), true);
  updates.setState(updates.getInitialState(), true);
  models.setState({ behind: "" });
  for (const command of Object.values(ipc.commands)) command.mockReset().mockResolvedValue(undefined);
  ipc.commands.providersState.mockResolvedValue([claude()]);
  ipc.commands.voiceMicrophones.mockResolvedValue([]);
  ipc.commands.voiceMicrophone.mockResolvedValue(null);
  voice.setState(voice.getInitialState(), true);
  dialog.setState(dialog.getInitialState(), true);
  look.setState(look.getInitialState(), true);
  settingsSheet.setState(settingsSheet.getInitialState(), true);
  delete document.documentElement.dataset.mode;
  delete document.documentElement.dataset.accent;
});

afterEach(() => {
  cleanup();
  showLanguage("es");
});

describe("general settings", () => {
  it("saves the name and reloads the profile the rail footer paints from", async () => {
    profile.setState({ person: { name: "Demo", checkUpdates: true, welcomed: true, seen: "", notify: true } });
    ipc.commands.profile.mockResolvedValue({ name: "Nuevo", checkUpdates: true });
    await open("general");

    const name = screen.getByLabelText("Nombre");
    expect(name).toHaveProperty("value", "Demo");
    fireEvent.change(name, { target: { value: "Nuevo" } });
    await act(async () => fireEvent.click(screen.getByText("Guardar")));

    expect(ipc.commands.saveProfile).toHaveBeenCalledWith("Nuevo");
    expect(profile.getState().person.name).toBe("Nuevo");
    expect(screen.getByText("Guardado.")).toBeTruthy();
  });

  it("says why the name was not saved", async () => {
    ipc.commands.saveProfile.mockRejectedValue("disco lleno");
    await open("general");
    await act(async () => fireEvent.click(screen.getByText("Guardar")));
    expect(screen.getByText("disco lleno").className).toBe("note fault");
  });

  it("puts the switch back when the preference cannot be saved", async () => {
    ipc.commands.setUpdateCheck.mockRejectedValue("sin permiso");
    await open("general");
    const toggle = screen.getByRole("switch", { name: "Buscar al abrir Sens" });
    expect(toggle.getAttribute("aria-checked")).toBe("true");
    await act(async () => fireEvent.click(toggle));
    expect(toggle.getAttribute("aria-checked")).toBe("true");
    expect(screen.getByRole("alert").textContent).toBe("sin permiso");
  });

  it("switches the notices off, and keeps them on when that cannot be saved", async () => {
    profile.setState({ person: { name: "Demo", checkUpdates: true, welcomed: true, seen: "", notify: true } });
    await open("general");
    const toggle = screen.getByRole("switch", { name: /Avisar cuando Claude termina/ });
    await act(async () => fireEvent.click(toggle));
    expect(ipc.commands.setNotify).toHaveBeenCalledWith(false);
    expect(toggle.getAttribute("aria-checked")).toBe("false");
    expect(profile.getState().person.notify).toBe(false);

    ipc.commands.setNotify.mockRejectedValue("sin permiso");
    await act(async () => fireEvent.click(toggle));
    expect(toggle.getAttribute("aria-checked")).toBe("false");
    expect(screen.getByRole("alert").textContent).toBe("sin permiso");
  });

  it("keeps Sens in the tray and starts it with Windows, and says how to quit it", async () => {
    await open("general");
    const tray = screen.getByRole("switch", { name: "Seguir en la bandeja al cerrar" });
    const startup = screen.getByRole("switch", { name: "Iniciar con Windows" });
    const quitting = screen.getByText("Para salir de Sens, elige Salir en el menú de la bandeja.");
    expect([tray.getAttribute("aria-checked"), startup.getAttribute("aria-checked")]).toEqual(["true", "true"]);
    expect(quitting.hidden).toBe(false);

    await act(async () => fireEvent.click(tray));
    expect(ipc.commands.setKeepInTray).toHaveBeenCalledWith(false);
    expect(tray.getAttribute("aria-checked")).toBe("false");
    expect(profile.getState().person.keepInTray).toBe(false);
    expect(quitting.hidden).toBe(true);

    await act(async () => fireEvent.click(startup));
    expect(ipc.commands.setStartWithWindows).toHaveBeenCalledWith(false);
    expect(startup.getAttribute("aria-checked")).toBe("false");
    expect(profile.getState().person.startWithWindows).toBe(false);
  });

  it("reads both from a profile that has them off, and keeps them off when turning one on fails", async () => {
    profile.setState({ person: { name: "Demo", checkUpdates: true, welcomed: true, seen: "", notify: true, keepInTray: false, startWithWindows: false } });
    ipc.commands.setStartWithWindows.mockRejectedValue("sin permiso");
    await open("general");
    const startup = screen.getByRole("switch", { name: "Iniciar con Windows" });
    expect(screen.getByRole("switch", { name: "Seguir en la bandeja al cerrar" }).getAttribute("aria-checked")).toBe("false");
    expect(screen.getByText("Para salir de Sens, elige Salir en el menú de la bandeja.").hidden).toBe(true);

    await act(async () => fireEvent.click(startup));
    expect(ipc.commands.setStartWithWindows).toHaveBeenCalledWith(true);
    expect(startup.getAttribute("aria-checked")).toBe("false");
    expect(screen.getByRole("alert").textContent).toBe("sin permiso");
  });

  it("opens the update panel", async () => {
    updates.setState({ latest: { version: "9.9.9", notes: "", page: "", size: 0 } });
    await open("general");
    fireEvent.click(screen.getByText("Ver Sens 9.9.9"));
    expect(dialog.getState()).toMatchObject({ open: true, title: "Sens 9.9.9" });
  });

  it("shows the news of this version over the chat, and gets out of the way", async () => {
    ipc.commands.news.mockResolvedValue([]);
    project.setState({ view: "" });
    await act(async () => openSettings("general"));
    render(<SettingsDialog />);

    await act(async () => fireEvent.click(screen.getByText("Ver novedades")));
    expect(project.getState().view).toBe("news");
    expect(settingsSheet.getState().open).toBe(false);
    expect(ipc.commands.news).toHaveBeenCalledTimes(1);
    project.setState({ view: "" });
  });
  it("puts the profile first and the welcome last, and leaves the shortcut and the voice to their own tab", async () => {
    await open("general");
    const labels = [...document.querySelectorAll("#settings-pane .pair > .label")].map((label) => label.textContent);
    expect(labels).toEqual(["Nombre", "Inicio y bandeja", "Avisos", "Actualizaciones", "Bienvenida"]);
    expect(screen.queryByText("Atajo")).toBeNull();
    expect(screen.queryByRole("combobox", { name: "Micrófono" })).toBeNull();
    expect(screen.getAllByRole("tab").map((tab) => tab.textContent)).toEqual(["General", "Focus y voz", "Apariencia", "Idioma", "Proveedores", "Proyecto"]);
  });
});

const ALT_SPACE = { ctrl: false, alt: true, shift: false, win: false, key: "Space" };
const CTRL_SHIFT_K = { ctrl: true, alt: false, shift: true, win: false, key: "K" };

describe("focus and voice settings", () => {
  const keycaps = () => [...screen.getByText("Atajo").parentElement!.querySelectorAll("kbd")].map((key) => key.textContent);
  const taken = () => screen.getByText("Otra app ya usa este atajo: cámbialo o abre el modo focus desde la bandeja.");

  it("shows the focus mode's shortcut, and says when another app already has it", async () => {
    ipc.commands.shortcutState.mockResolvedValue({ keys: ALT_SPACE, named: "Alt+Espacio", taken: false });
    await open("focus");
    expect(keycaps()).toEqual(["Alt", "Espacio"]);
    expect(taken().hidden).toBe(true);
    expect(screen.getByText("Volver a Alt+Espacio").hidden).toBe(true);

    cleanup();
    ipc.commands.shortcutState.mockResolvedValue({ keys: ALT_SPACE, named: "Alt+Espacio", taken: true });
    await open("focus");
    expect(taken().hidden).toBe(false);
  });

  it("leaves the shortcut out when Sens cannot say what it is", async () => {
    ipc.commands.shortcutState.mockRejectedValue("sin atajo");
    await open("focus");
    expect(screen.queryByText("Atajo")).toBeNull();
    expect(screen.getByRole("combobox", { name: "Micrófono" })).toBeTruthy();
  });

  it("listens for the next keys with the shortcut paused, and keeps them", async () => {
    ipc.commands.shortcutState.mockResolvedValue({ keys: ALT_SPACE, named: "Alt+Espacio", taken: false });
    ipc.commands.shortcutSet.mockResolvedValue({ keys: CTRL_SHIFT_K, named: "Ctrl+Mayús+K", taken: false });
    await open("focus");
    const change = screen.getByRole("button", { name: "Cambiar" });

    await act(async () => fireEvent.click(change));
    expect(ipc.commands.shortcutPause).toHaveBeenCalledWith(true);
    expect(change.textContent).toBe("Pulsa la combinación · Esc cancela");

    await act(async () => fireEvent.keyDown(change, { key: "Control", code: "ControlLeft", ctrlKey: true }));
    expect(ipc.commands.shortcutSet).not.toHaveBeenCalled();
    await act(async () => fireEvent.keyDown(change, { key: "K", code: "KeyK", ctrlKey: true, shiftKey: true }));

    expect(ipc.commands.shortcutSet).toHaveBeenCalledWith(CTRL_SHIFT_K);
    expect(keycaps()).toEqual(["Ctrl", "Mayús", "K"]);
    expect(change.textContent).toBe("Cambiar");
    expect(screen.getByRole("button", { name: "Volver a Alt+Espacio" }).hidden).toBe(false);
  });

  it("says why keys without Ctrl, Alt or Win cannot open Sens, and Esc gives the old shortcut back", async () => {
    ipc.commands.shortcutState.mockResolvedValue({ keys: ALT_SPACE, named: "Alt+Espacio", taken: false });
    await open("focus");
    const change = screen.getByRole("button", { name: "Cambiar" });
    await act(async () => fireEvent.click(change));

    await act(async () => fireEvent.keyDown(change, { key: "k", code: "KeyK", shiftKey: true }));
    expect(ipc.commands.shortcutSet).not.toHaveBeenCalled();
    expect(screen.getByRole("alert").textContent).toBe("Usa Ctrl, Alt o Win con una letra, un número, F1–F24 o Espacio");

    await act(async () => fireEvent.keyDown(change, { key: "Escape", code: "Escape" }));
    expect(ipc.commands.shortcutPause).toHaveBeenLastCalledWith(false);
    expect(change.textContent).toBe("Cambiar");
  });

  it("keeps the old shortcut and says so when another app has the new one, and goes back to Alt+Space", async () => {
    ipc.commands.shortcutState.mockResolvedValue({ keys: CTRL_SHIFT_K, named: "Ctrl+Mayús+K", taken: false });
    ipc.commands.shortcutSet.mockRejectedValueOnce("Otra app ya usa Ctrl+F9; Sens se queda con el que tenía");
    await open("focus");
    const change = screen.getByRole("button", { name: "Cambiar" });
    await act(async () => fireEvent.click(change));
    await act(async () => fireEvent.keyDown(change, { key: "F9", code: "F9", ctrlKey: true }));
    expect(screen.getByRole("alert").textContent).toBe("Otra app ya usa Ctrl+F9; Sens se queda con el que tenía");
    expect(keycaps()).toEqual(["Ctrl", "Mayús", "K"]);

    ipc.commands.shortcutSet.mockResolvedValue({ keys: ALT_SPACE, named: "Alt+Espacio", taken: false });
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Volver a Alt+Espacio" })));
    expect(ipc.commands.shortcutSet).toHaveBeenLastCalledWith(ALT_SPACE);
    expect(keycaps()).toEqual(["Alt", "Espacio"]);
  });

  it("gives the shortcut back when the tab closes while it listens", async () => {
    ipc.commands.shortcutState.mockResolvedValue({ keys: ALT_SPACE, named: "Alt+Espacio", taken: false });
    await open("focus");
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Cambiar" })));
    cleanup();
    expect(ipc.commands.shortcutPause).toHaveBeenLastCalledWith(false);
  });

  it("chooses the microphone, tests it with a level bar, and says where the voice model is", async () => {
    ipc.commands.voiceMicrophones.mockResolvedValue([
      { name: "Voicemeeter Out B2", default: true },
      { name: "Micrófono (USB)", default: false },
    ]);
    voice.setState({ ready: false, fetching: true, done: 30, total: 100 });
    await open("focus");
    const microphone = screen.getByRole("combobox", { name: "Micrófono" }) as HTMLSelectElement;
    expect([...microphone.options].map((one) => one.textContent)).toEqual(["El predeterminado de Windows · Voicemeeter Out B2", "Voicemeeter Out B2", "Micrófono (USB)"]);
    expect(screen.getByText("Descargando el modelo de voz, una sola vez · 30 %")).toBeTruthy();

    await act(async () => fireEvent.change(microphone, { target: { value: "Micrófono (USB)" } }));
    expect(ipc.commands.voiceChoose).toHaveBeenCalledWith("Micrófono (USB)");
    expect(voice.getState().chosen).toBe("Micrófono (USB)");

    ipc.commands.voiceTest.mockResolvedValue(3);
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Probar" })));
    act(() => ipc.heard.voice({ kind: "level", id: 3, level: 0.42 }));
    expect(screen.getByRole("meter", { name: "Nivel del micrófono" }).getAttribute("aria-valuenow")).toBe("42");
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Parar" })));
    expect(ipc.commands.voiceStop).toHaveBeenCalled();
    act(() => ipc.heard.voice({ kind: "ended", id: 3, refusal: null }));
    expect(screen.getByRole("button", { name: "Probar" })).toBeTruthy();

    act(() => voice.setState({ ready: true, fetching: false, total: 190_085_487 }));
    expect(screen.getByText("Modelo de voz en este equipo · 181 MB")).toBeTruthy();
  });

  it("offers to download the voice model again when it could not come", async () => {
    voice.setState({ ready: false, fetching: false, fault: "no pude descargar el modelo" });
    await open("focus");
    expect(screen.getByText("no pude descargar el modelo")).toBeTruthy();
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Descargar" })));
    expect(ipc.commands.voicePrepare).toHaveBeenCalled();
  });
});

describe("the settings sheet", () => {
  it("opens over the window on the section asked for, and gives the focus back when it closes", async () => {
    const opener = Object.assign(document.createElement("button"), { textContent: "Ajustes" });
    document.body.append(opener);
    render(<SettingsDialog />);
    expect(screen.queryByRole("tablist", { name: "Secciones de ajustes" })).toBeNull();

    await act(async () => openSettings("look", opener));
    expect(screen.getByRole("tab", { name: "Apariencia" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByRole("tabpanel").textContent).toContain("Oscuro · Señal");

    await act(async () => fireEvent.keyDown(screen.getByRole("tab", { name: "Apariencia" }), { key: "ArrowRight" }));
    expect(settings.getState().section).toBe("language");
    expect(document.activeElement).toBe(screen.getByRole("tab", { name: "Idioma" }));
    await act(async () => fireEvent.keyDown(screen.getByRole("tab", { name: "Idioma" }), { key: "ArrowRight" }));
    expect(settings.getState().section).toBe("providers");

    await act(async () => fireEvent.click(screen.getByLabelText("Cerrar ajustes")));
    expect(settingsSheet.getState().open).toBe(false);
    expect(screen.queryByRole("tablist", { name: "Secciones de ajustes" })).toBeNull();
    expect(document.activeElement).toBe(opener);
    opener.remove();
  });
});

describe("appearance settings", () => {
  it("shows the chosen look at once and keeps it", async () => {
    await open("look");
    expect(screen.getByRole("radio", { name: /Oscuro/ })).toHaveProperty("checked", true);

    await act(async () => fireEvent.click(screen.getByRole("radio", { name: /Claro/ })));
    expect(document.documentElement.dataset).toMatchObject({ mode: "light", accent: "signal" });
    await act(async () => fireEvent.click(screen.getByRole("radio", { name: "Rosa" })));

    expect(ipc.commands.setLook).toHaveBeenLastCalledWith({ mode: "light", accent: "rose" });
    expect(document.documentElement.dataset.accent).toBe("rose");
    expect(screen.getByText("Claro · Rosa")).toBeTruthy();
    expect(screen.getByRole("radio", { name: "Rosa" })).toHaveProperty("checked", true);
  });

  it("goes back to the look it had when the new one cannot be kept", async () => {
    ipc.commands.setLook.mockRejectedValue("sin permiso");
    await open("look");

    await act(async () => fireEvent.click(screen.getByRole("radio", { name: "Iris" })));

    expect(document.documentElement.dataset.accent).toBe("signal");
    expect(screen.getByRole("radio", { name: "Señal" })).toHaveProperty("checked", true);
    expect(screen.getByRole("alert").textContent).toBe("sin permiso");
  });
});

describe("language settings", () => {
  it("names each language in its own words, keeps the one chosen and speaks it at once", async () => {
    await open("language");
    expect(screen.getByRole("radiogroup", { name: "Idioma" })).toBeTruthy();
    expect(screen.getAllByRole("radio").map((one) => one.parentElement!.textContent)).toEqual(["English", "Español", "Français", "Deutsch", "日本語", "简体中文"]);
    expect(screen.getByRole("radio", { name: "Español" })).toHaveProperty("checked", true);

    await act(async () => fireEvent.click(screen.getByRole("radio", { name: "Français" })));

    expect(ipc.commands.setLanguage).toHaveBeenCalledWith("fr");
    expect(languageNow()).toBe("fr");
    expect(document.documentElement.lang).toBe("fr-FR");
    expect(screen.getByRole("radio", { name: "Français" })).toHaveProperty("checked", true);
    expect(screen.getByText("Sens passe immédiatement à cette langue. Claude vous répond dans la langue dans laquelle vous écrivez.")).toBeTruthy();
  });

  it("stays in the language it had when the new one cannot be kept", async () => {
    ipc.commands.setLanguage.mockRejectedValue("sin permiso");
    await open("language");

    await act(async () => fireEvent.click(screen.getByRole("radio", { name: "Deutsch" })));

    expect(languageNow()).toBe("es");
    expect(screen.getByRole("radio", { name: "Español" })).toHaveProperty("checked", true);
    expect(screen.getByRole("alert").textContent).toBe("sin permiso");
  });
});

describe("providers settings", () => {

  it("asks for a key before telling Rust to use one", async () => {
    await open("providers");
    await act(async () => fireEvent.click(screen.getByLabelText(/Clave de API/)));
    expect(ipc.commands.setProviderMethod).not.toHaveBeenCalled();
    expect(screen.getByPlaceholderText("sk-ant-…")).toBeTruthy();
  });

  it("clears the key field once the key is saved", async () => {
    ipc.commands.providersState.mockResolvedValue([claude({ method: "apiKey" })]);
    await open("providers");
    const key = screen.getByLabelText("Clave de API", { selector: "input" });
    fireEvent.change(key, { target: { value: "sk-ant-secreto" } });
    await act(async () => fireEvent.click(screen.getByText("Guardar clave")));
    expect(ipc.commands.saveApiKey).toHaveBeenCalledWith("claude", "sk-ant-secreto");
    expect(key).toHaveProperty("value", "");
  });

  it("holds the sign-in button and tells the model picker while the browser is open", async () => {
    const signing = later();
    ipc.commands.providerSignIn.mockReturnValue(signing.promise);
    await open("providers");

    await act(async () => fireEvent.click(screen.getByText("Iniciar sesión con Claude")));
    const button = screen.getByText("Esperando a que termines…");
    expect(button).toHaveProperty("disabled", true);
    expect(settings.getState().connecting).toBe(true);

    await act(async () => signing.settle());
    expect(settings.getState().connecting).toBe(false);
    expect(readAccount).toHaveBeenCalled();
    expect(refreshModels).toHaveBeenCalled();
  });

  it("follows the Claude Code download and locks the card meanwhile", async () => {
    const installing = later<string>();
    ipc.commands.claudeCodeInstall.mockReturnValue(installing.promise);
    ipc.commands.providersState.mockResolvedValue([claude({ installed: false, account: null })]);
    await open("providers");

    await act(async () => fireEvent.click(screen.getByText("Instalar ahora")));
    await act(async () => ipc.heard.claudeCode({ stage: "downloading", done: 50, total: 100 * 1048576 }));
    expect(screen.getByRole("status").textContent).toMatch(/^Descargando Claude Code… 0 % de 100 MB$/);
    expect(screen.getByText("Comprobar otra vez")).toHaveProperty("disabled", true);

    await act(async () => installing.settle("2.1.0"));
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("offers a newer Claude Code, updates it, and asks again for its models", async () => {
    const updating = later<string>();
    ipc.commands.claudeCodeUpdate.mockReturnValue(updating.promise);
    models.setState({ behind: "2.1.281" });
    await open("providers");

    expect(screen.getByText(/Hay una versión nueva de Claude Code: v2\.1\.281\./)).toBeTruthy();
    await act(async () => fireEvent.click(screen.getByText("Actualizar Claude Code")));
    expect(screen.getByRole("status").textContent).toBe("Actualizando Claude Code…");
    expect(screen.getByText("Actualizar Claude Code")).toHaveProperty("disabled", true);

    await act(async () => updating.settle("2.1.281"));
    expect(screen.queryByRole("status")).toBeNull();
    expect(ipc.commands.claudeCodeNewer).toHaveBeenCalled();
    expect(refreshModels).toHaveBeenCalled();
    expect(screen.queryByText("Actualizar Claude Code")).toBeNull();
  });
});
