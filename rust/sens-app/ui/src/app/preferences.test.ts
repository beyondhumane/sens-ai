// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";
import { asker } from "../dev/acting";
import { project } from "../features/project/store";
import { updates } from "../features/updates/store";
import { look } from "../shared/look";
import { answerPreferences } from "./preferences";

const ipc = vi.hoisted(() => ({
  commands: { actAnswer: vi.fn() },
  chooseLook: vi.fn(),
  chooseLanguage: vi.fn(),
  setNotices: vi.fn(),
  setResident: vi.fn(),
  setAutomatic: vi.fn(),
  checkUpdates: vi.fn(),
  openSettings: vi.fn(),
  showView: vi.fn(),
  loadCapabilities: vi.fn(),
}));

vi.mock("../ipc/commands", () => ({ commands: ipc.commands, events: {} }));
vi.mock("../features/look/store", () => ({ chooseLook: ipc.chooseLook }));
vi.mock("../features/look/useLanguageChoice", () => ({ chooseLanguage: ipc.chooseLanguage }));
vi.mock("../features/notify/store", () => ({ setNotices: ipc.setNotices }));
vi.mock("../features/settings/store", () => ({ setResident: ipc.setResident, openSettings: ipc.openSettings }));
vi.mock("../features/capabilities/store", () => ({ loadCapabilities: ipc.loadCapabilities }));
vi.mock("./session", () => ({ showView: ipc.showView }));
vi.mock(import("../features/updates/store"), async (original) => ({ ...(await original()), checkUpdates: ipc.checkUpdates, setAutomatic: ipc.setAutomatic }));

const ask = asker(ipc.commands.actAnswer);

answerPreferences();

beforeEach(() => {
  for (const mock of [ipc.commands.actAnswer, ipc.chooseLook, ipc.chooseLanguage, ipc.setNotices, ipc.setResident, ipc.setAutomatic, ipc.checkUpdates, ipc.openSettings, ipc.showView, ipc.loadCapabilities]) {
    mock.mockReset().mockResolvedValue(undefined);
  }
  look.setState({ chosen: { mode: "dark", accent: "signal" } });
  project.setState({ session: "s1", view: "" });
  updates.setState(updates.getInitialState(), true);
});

describe("Claude setting up Sens", () => {
  it("changes only the part of the look it was asked for", async () => {
    expect(await ask("set_look", { mode: "light", accent: null })).toEqual({ ok: true, text: "Sens looks light with the signal accent." });
    expect(ipc.chooseLook).toHaveBeenCalledWith({ mode: "light", accent: "signal" });
  });

  it("turns each preference with the setter the settings use", async () => {
    await ask("set_preference", { name: "keep_in_tray", on: false });
    await ask("set_preference", { name: "check_updates", on: true });
    await ask("set_preference", { name: "notify", on: true });
    expect(ipc.setResident).toHaveBeenCalledWith("keepInTray", false);
    expect(ipc.setAutomatic).toHaveBeenCalledWith(true);
    expect(ipc.setNotices).toHaveBeenCalledWith(true);
  });

  it("brings a view only for the session on screen", async () => {
    await ask("show_view", { session: "s1", view: "artifacts" });
    await ask("show_view", { session: "s1", view: "settings" });
    expect(ipc.showView).toHaveBeenCalledWith("artifacts");
    expect(ipc.openSettings).toHaveBeenCalled();
    expect((await ask("show_view", { session: "s2", view: "news" })).ok).toBe(false);
  });

  it("tells whether a newer Sens is out", async () => {
    updates.setState({ current: "0.30.0" });
    expect((await ask("check_updates", {})).text).toBe("Sens 0.30.0 is the latest version.");
    ipc.checkUpdates.mockImplementation(async () => updates.setState({ latest: { version: "0.31.0", notes: "", page: "", size: 1 }, installable: true }));
    expect((await ask("check_updates", {})).text).toBe("Sens 0.31.0 is out (this is 0.30.0). The person can install it from the pill in the title bar.");
  });

  it("reloads the capabilities only when they are on screen", async () => {
    await ask("refresh_capabilities", {});
    expect(ipc.loadCapabilities).not.toHaveBeenCalled();
    project.setState({ view: "capabilities" });
    await ask("refresh_capabilities", {});
    expect(ipc.loadCapabilities).toHaveBeenCalled();
  });
});
