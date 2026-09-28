import { act, cleanup, render } from "@testing-library/react";
import { useStore } from "zustand";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import type { ChatEvent } from "../../ipc/types";
import { composer, rereadSettings } from "../composer/store";
import { adopt, focused, newPane } from "../panes/store";
import { project } from "../project/store";
import { Reply } from "./Reply";
import { hearIn, keepQuiet, send } from "./store";
import type { Reply as ReplyTurn } from "./turns";

const ipc = vi.hoisted(() => ({
  commands: {
    openSession: vi.fn(async () => "b1"),
    chatSend: vi.fn(async () => undefined),
    workspaces: vi.fn(async () => []),
    folder: vi.fn(async () => []),
    changes: vi.fn(async () => null),
    repo: vi.fn(async () => null),
  },
}));

vi.mock("../../ipc/commands", () => ({ commands: ipc.commands, events: {} }));

const SETTINGS = { provider: "claude", model: "claude-demo", effort: "", thinking: true, mode: "default" };
const own = newPane("C:/demo");
const tell = (event: ChatEvent) => act(() => hearIn(own, "b1", event));
const settle = () => act(async () => new Promise((done) => setTimeout(done, 50)));

function Last() {
  const reply = useStore(own.chat, (s) => s.turns.findLast((turn): turn is ReplyTurn => turn.kind === "reply"));
  return reply ? <Reply turn={reply} /> : null;
}

beforeAll(() => {
  adopt(own);
  keepQuiet();
});

afterEach(cleanup);

describe("a chat in a window of its own", () => {
  it("is the only pane, in focus, and its project is the one on screen", () => {
    expect(focused()).toBe(own);
    expect(project.getState()).toMatchObject({ root: "C:/demo", work: "C:/demo" });
  });

  it("sends, hears its own session and draws the last reply, and never reads the rail, the files, the changes or the repository", async () => {
    render(<Last />);
    await act(async () => send({ message: { text: "Hola", files: [], images: [] }, shownFiles: [], pictures: [] }, SETTINGS, own));
    expect(ipc.commands.chatSend).toHaveBeenCalledWith("C:/demo", "b1", { text: "Hola", files: [], images: [] }, SETTINGS);
    expect(document.querySelector(".live-said")?.textContent).toBe("Enviando…");

    tell({ kind: "started", model: "claude-demo" });
    tell({ kind: "said", text: "Hecho." });
    tell({ kind: "finished", ok: true, stopped: false, millis: 1000, turns: 1, tokensIn: 1, tokensOut: 20, error: "" });
    await settle();

    expect(document.querySelector(".said")?.textContent).toBe("Hecho.");
    expect(document.querySelector(".reply-foot")?.textContent).toBe("1 s · 20 tokens");
    expect(document.querySelector(".live")).toBeNull();
    expect(own.chat.getState()).toMatchObject({ busy: false, ended: 1 });
    for (const reading of [ipc.commands.workspaces, ipc.commands.folder, ipc.commands.changes, ipc.commands.repo]) expect(reading).not.toHaveBeenCalled();
  });

  it("reads the stored model, effort, thinking and mode again when asked", () => {
    localStorage.setItem("sens.mode", JSON.stringify("plan"));
    localStorage.setItem("sens.choice", JSON.stringify({ provider: "claude", model: "opus" }));
    localStorage.setItem("sens.effort", JSON.stringify("high"));
    localStorage.setItem("sens.thinking", JSON.stringify(false));
    act(() => rereadSettings(own));

    expect(composer.getState().mode).toBe("plan");
    expect(own.desk.getState()).toMatchObject({ choice: { provider: "claude", model: "opus" }, effort: "high", thinking: false });
  });
});
