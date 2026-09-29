import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { focused } from "../features/panes/store";
import { Window } from "./Topbar";

const ipc = vi.hoisted(() => ({
  commands: new Proxy({} as Record<string, ReturnType<typeof vi.fn>>, {
    get: (known, name: string) => (known[name] ??= vi.fn(async () => undefined)),
  }),
}));

vi.mock("../ipc/commands", () => ({ commands: ipc.commands, events: {} }));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    isMaximized: async () => false,
    onResized: async () => () => {},
    minimize: vi.fn(),
    toggleMaximize: vi.fn(),
    close: vi.fn(),
  }),
}));

beforeEach(() => {
  for (const command of Object.values(ipc.commands)) command.mockClear();
  ipc.commands.shortcutState.mockResolvedValue({ keys: { ctrl: false, alt: true, shift: false, win: false, key: "Space" }, named: "Alt+Espacio", taken: false });
  focused().desk.setState({ root: "C:/Proyectos/web", session: "s7", text: "" });
});

afterEach(cleanup);

describe("the focus mode button", () => {
  it("names its shortcut, and hands focus mode the session Sens has open", async () => {
    await act(async () => render(<Window />));
    const button = screen.getByRole("button", { name: "Modo focus" });
    expect(button.title).toBe("Modo focus · Oculta Sens y pregúntale desde cualquier ventana (Alt+Espacio)");

    await act(async () => fireEvent.click(button));
    expect(ipc.commands.barFocus).toHaveBeenCalledWith({ root: "C:/Proyectos/web", session: "s7", text: "" });
  });

  it("stays out of windows without tools", async () => {
    await act(async () => render(<Window tools={false} />));
    expect(screen.queryByRole("button", { name: "Modo focus" })).toBeNull();
  });
});
