import { act, cleanup, render } from "@testing-library/react";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { voice } from "../features/voice/store";
import type { BarOpened, VoiceHeard } from "../ipc/types";
import { Bar } from "./Bar";
import { bar, opened, own } from "./store";

const ipc = vi.hoisted(() => ({
  commands: new Proxy({} as Record<string, ReturnType<typeof vi.fn>>, {
    get: (known, name: string) => (known[name] ??= vi.fn(async () => undefined)),
  }),
  listeners: new Set<(what: VoiceHeard) => void>(),
}));

vi.mock("../ipc/commands", () => ({
  commands: ipc.commands,
  events: {
    voice: (hear: (what: VoiceHeard) => void) => {
      ipc.listeners.add(hear);
      return Promise.resolve(() => void ipc.listeners.delete(hear));
    },
  },
}));

const open = (listen: boolean) =>
  act(() => opened({ look: { mode: "dark", accent: "signal" }, language: "es", front: null, pinned: false, resume: null, listen } as BarOpened));

const hear = (what: VoiceHeard) => act(() => [...ipc.listeners].forEach((listener) => listener(what)));

beforeAll(() => {
  vi.stubGlobal("ResizeObserver", class { observe() {} disconnect() {} });
});

beforeEach(() => {
  for (const command of Object.values(ipc.commands)) command.mockClear();
  ipc.listeners.clear();
  ipc.commands.barProjects.mockResolvedValue([{ root: "C:/nitid", name: "nitid" }]);
  ipc.commands.workspaces.mockResolvedValue([]);
  ipc.commands.voiceStart.mockResolvedValue(7);
  voice.setState({ ready: true, fetching: false });
  own.chat.setState({ turns: [], busy: false });
  own.desk.setState({ root: "C:/nitid", session: "", text: "" });
});

afterEach(cleanup);

describe("saying “Hey Sens”", () => {
  it("opens focus mode already dictating, hands-free, and leaves what was said to send", async () => {
    render(<Bar />);
    await open(true);
    expect(ipc.commands.voiceStart).toHaveBeenCalledWith("es", true);

    hear({ kind: "phrase", id: 7, text: "Revisa el componente del botón." });
    hear({ kind: "ended", id: 7, refusal: null });
    expect(own.desk.getState().text).toBe("Revisa el componente del botón.");
    expect(ipc.commands.chatSend).not.toHaveBeenCalled();
  });

  it("does not start a second dictation over the one that is listening", async () => {
    render(<Bar />);
    await open(true);
    await open(true);
    expect(ipc.commands.voiceStart).toHaveBeenCalledTimes(1);
  });

  it("leaves the microphone alone when focus mode opens from the shortcut", async () => {
    render(<Bar />);
    await open(false);
    expect(ipc.commands.voiceStart).not.toHaveBeenCalled();
  });

  it("does not dictate by itself when focus mode draws again, as when the language changes", async () => {
    bar.setState({ called: 3 });
    render(<Bar />);
    await act(async () => {});
    expect(ipc.commands.voiceStart).not.toHaveBeenCalled();
  });

  it("waits for the voice model instead of dictating without it", async () => {
    voice.setState({ ready: false, fetching: true });
    render(<Bar />);
    await open(true);
    expect(ipc.commands.voiceStart).not.toHaveBeenCalled();
  });
});
