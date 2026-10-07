import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { Bar } from "./Bar";
import { bar, own } from "./store";

const ipc = vi.hoisted(() => ({
  commands: new Proxy({} as Record<string, ReturnType<typeof vi.fn>>, {
    get: (known, name: string) => (known[name] ??= vi.fn(async () => undefined)),
  }),
}));

vi.mock("../ipc/commands", () => ({ commands: ipc.commands, events: {} }));

const handle = () => document.querySelector<HTMLElement>(".bar-handle")!;
const row = () => document.querySelector<HTMLElement>(".bar-row")!;

beforeAll(() => {
  vi.stubGlobal("ResizeObserver", class { observe() {} disconnect() {} });
});

beforeEach(() => {
  for (const command of Object.values(ipc.commands)) command.mockClear();
  own.chat.setState({ turns: [], busy: false });
  own.desk.setState({ root: "C:/nitid", session: "", text: "" });
  bar.setState({ pinned: false, choosing: false, projects: [{ root: "C:/nitid", name: "nitid" }], sessions: {} });
});

afterEach(cleanup);

describe("moving and pinning focus mode", () => {
  it("moves once the S is dragged a few pixels, and a plain click does not", async () => {
    render(<Bar />);
    fireEvent.pointerDown(handle(), { button: 0, clientX: 20, clientY: 20 });
    fireEvent.pointerMove(window, { clientX: 22, clientY: 21 });
    fireEvent.pointerUp(window);
    expect(ipc.commands.barDrag).not.toHaveBeenCalled();

    fireEvent.pointerDown(handle(), { button: 0, clientX: 20, clientY: 20 });
    fireEvent.pointerMove(window, { clientX: 40, clientY: 26 });
    fireEvent.pointerMove(window, { clientX: 60, clientY: 30 });
    expect(ipc.commands.barDrag).toHaveBeenCalledTimes(1);
  });

  it("moves from the empty part of the bar, pinned or not, but not from the field or the buttons", async () => {
    render(<Bar />);
    fireEvent.pointerDown(screen.getByRole("textbox", { name: "Mensaje para Sens" }), { button: 0, clientX: 100, clientY: 20 });
    fireEvent.pointerMove(window, { clientX: 140, clientY: 20 });
    fireEvent.pointerUp(window);
    expect(ipc.commands.barDrag).not.toHaveBeenCalled();

    fireEvent.pointerDown(row(), { button: 0, clientX: 100, clientY: 20 });
    fireEvent.pointerMove(window, { clientX: 140, clientY: 20 });
    expect(ipc.commands.barDrag).toHaveBeenCalledTimes(1);
  });

  it("switches project with Ctrl+Tab, and leaves Tab to reach its buttons", async () => {
    bar.setState({ projects: [{ root: "C:/nitid", name: "nitid" }, { root: "C:/web", name: "web" }] });
    render(<Bar />);
    const field = screen.getByRole("textbox", { name: "Mensaje para Sens" });
    expect(fireEvent.keyDown(field, { key: "Tab" })).toBe(true);
    expect(own.desk.getState().root).toBe("C:/nitid");
    await act(async () => fireEvent.keyDown(field, { key: "Tab", ctrlKey: true }));
    expect(own.desk.getState().root).toBe("C:/web");
    await act(async () => fireEvent.keyDown(field, { key: "Tab", ctrlKey: true, shiftKey: true }));
    expect(own.desk.getState().root).toBe("C:/nitid");
  });

  it("goes back to the center of its screen on a double click", async () => {
    render(<Bar />);
    fireEvent.doubleClick(handle());
    expect(ipc.commands.barRecenter).toHaveBeenCalledTimes(1);
    fireEvent.doubleClick(screen.getByRole("textbox", { name: "Mensaje para Sens" }));
    expect(ipc.commands.barRecenter).toHaveBeenCalledTimes(1);
  });

  it("always offers to pin it, and says it is pinned once it is", async () => {
    render(<Bar />);
    const offer = screen.getByRole("button", { name: "Fijar: se queda visible sobre cualquier ventana de esta pantalla (Ctrl+P)" });
    expect(offer.getAttribute("aria-pressed")).toBe("false");

    await act(async () => fireEvent.click(offer));
    expect(ipc.commands.barPin).toHaveBeenCalledWith(true);
    const pinned = screen.getByRole("button", { name: "Fijada: se queda encima hasta que la sueltes" });
    expect(pinned.getAttribute("aria-pressed")).toBe("true");

    await act(async () => fireEvent.click(pinned));
    expect(ipc.commands.barPin).toHaveBeenLastCalledWith(false);
  });
});
