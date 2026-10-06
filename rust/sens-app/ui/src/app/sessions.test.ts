// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";
import { asker } from "../dev/acting";
import { newPane, panes, type Pane } from "../features/panes/store";
import { project } from "../features/project/store";
import { answerSessions } from "./sessions";

const ipc = vi.hoisted(() => ({
  commands: { actAnswer: vi.fn(), chatStop: vi.fn(), repo: vi.fn() },
  send: vi.fn(),
  choose: vi.fn(),
  openBeside: vi.fn(),
  draftBeside: vi.fn(),
  closePane: vi.fn(),
  loadRail: vi.fn(),
}));

vi.mock("../ipc/commands", () => ({ commands: ipc.commands, events: {} }));
vi.mock("./session", () => ({ openBeside: ipc.openBeside, draftBeside: ipc.draftBeside, closePane: ipc.closePane }));
vi.mock("../features/rail/store", () => ({ loadRail: ipc.loadRail }));
vi.mock("../features/composer/store", () => ({
  send: ipc.send,
  switched: vi.fn(),
  effortLevels: (card?: { efforts: string[] }) => card?.efforts ?? [],
}));
vi.mock("../features/models/store", () => {
  const opus = { id: "claude-opus-5-5", label: "Opus 5.5", efforts: ["low", "high", "max"] };
  return {
    models: { getState: () => ({ catalog: [{ id: "anthropic" }] }) },
    offeredBy: () => [opus],
    sameModel: (one: string, other: string) => one === other,
    chosenCard: () => opus,
    choose: ipc.choose,
  };
});

const ask = asker(ipc.commands.actAnswer);

function paneFor(session: string): Pane {
  const pane = newPane("C:/demo");
  pane.desk.setState({ session });
  return pane;
}

const sends = (pane: Pane, id: string) =>
  ipc.send.mockImplementation(async () => {
    pane.desk.setState({ session: id });
    pane.chat.setState({ busy: true });
  });

answerSessions();

let mine: Pane;

beforeEach(() => {
  for (const mock of [...Object.values(ipc.commands), ipc.send, ipc.choose, ipc.openBeside, ipc.draftBeside, ipc.closePane, ipc.loadRail]) mock.mockReset().mockResolvedValue(undefined);
  mine = paneFor("s1");
  panes.setState({ open: [mine], focus: mine.id });
  project.setState({ root: "C:/demo", work: "C:/demo", session: "s1" });
});

describe("Claude working with other sessions", () => {
  it("sends to a session already on screen without moving anything", async () => {
    const other = paneFor("s2");
    panes.setState({ open: [mine, other] });
    sends(other, "s2");
    expect(await ask("send_to_session", { session: "s1", target: "s2", root: "C:/demo", text: "Revisa la rama" })).toEqual({ ok: true, text: "Sent; that session is working on it." });
    expect(ipc.send).toHaveBeenCalledWith("Revisa la rama", other);
    expect(ipc.openBeside).not.toHaveBeenCalled();
  });

  it("opens a session beside before writing to it, but only for the session on screen", async () => {
    const other = paneFor("s2");
    ipc.openBeside.mockImplementation(async () => panes.setState({ open: [mine, other] }));
    sends(other, "s2");
    expect((await ask("send_to_session", { session: "s1", target: "s2", root: "C:/demo", text: "hola" })).ok).toBe(true);
    expect(ipc.openBeside).toHaveBeenCalledWith("C:/demo", "s2");

    panes.setState({ open: [mine] });
    project.setState({ session: "s9" });
    expect((await ask("open_session", { session: "s1", target: "s2", root: "C:/demo" })).text).toContain("looking at another session");
  });

  it("starts a session beside with its first message, and says when nothing could answer", async () => {
    const fresh = paneFor("");
    ipc.draftBeside.mockResolvedValue(fresh);
    sends(fresh, "s3");
    expect(await ask("new_session", { session: "s1", root: "C:/api", prompt: "Escribe las pruebas" })).toEqual({ ok: true, text: "Session s3 started beside this one and is working on it." });
    ipc.send.mockReset().mockResolvedValue(undefined);
    ipc.draftBeside.mockResolvedValue(paneFor(""));
    expect((await ask("new_session", { session: "s1", root: "C:/api", prompt: "otra" })).ok).toBe(false);
    expect((await ask("new_session", { session: "s1", root: "C:/api", prompt: "" })).text).toBe("A new, empty session is open beside this one for the person.");
  });

  it("chooses the model and an effort the model has, and refuses one it lacks", async () => {
    const said = await ask("set_session_model", { target: "s1", model: "Opus 5.5", effort: "max", thinking: true });
    expect(said).toEqual({ ok: true, text: "Next messages use Opus 5.5, max effort, thinking on." });
    expect(ipc.choose).toHaveBeenCalledWith("anthropic", "claude-opus-5-5", mine);
    expect(mine.desk.getState()).toMatchObject({ effort: "max", thinking: true });
    expect((await ask("set_session_model", { target: "s1", effort: "medium" })).text).toBe("Opus 5.5 has no medium effort; it offers low, high, max.");
    expect((await ask("set_session_model", { target: "s1", model: "gpt" })).text).toBe("The model picker offers no gpt.");
  });

  it("closes a half of the window only when there are two", async () => {
    expect((await ask("close_session_pane", { target: "s1" })).text).toBe("Only one chat is on screen; there is nothing to close.");
    const other = paneFor("s2");
    panes.setState({ open: [mine, other] });
    expect((await ask("close_session_pane", { target: "s2" })).ok).toBe(true);
    expect(ipc.closePane).toHaveBeenCalledWith(other);
  });
});
