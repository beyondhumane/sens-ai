import { act, cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { shell } from "../../app/shell";
import { project } from "../project/store";
import { MapPanel, MapTally } from "./MapPanel";
import { atlas, loadMap } from "./store";

const ipc = vi.hoisted(() => ({
  commands: { canonMap: vi.fn(), canonReach: vi.fn(), openFile: vi.fn() },
}));

vi.mock("../../ipc/commands", () => ({ commands: ipc.commands }));

const MAP = {
  regions: [
    { name: "src", files: ["src/app.ts", "src/runner.ts"], doors: ["src/runner.ts"], uses: ["lib"], exports: ["run"] },
    { name: "lib", files: ["lib/format.ts"], doors: ["lib/format.ts"], uses: [], exports: ["stem"] },
  ],
  central: [{ path: "lib/format.ts", dependents: 4 }],
  strays: [{ path: "src/app.ts", area: "lib" }],
};

const REACH = {
  file: "lib/format.ts",
  area: "lib",
  dependents: [
    { path: "src/runner.ts", area: "src", steps: 1 },
    { path: "src/app.ts", area: "src", steps: 2 },
  ],
  tests: [],
};

beforeEach(() => {
  atlas.setState(atlas.getInitialState(), true);
  project.setState({ root: "C:/demo", work: "C:/demo", session: "s1" });
  for (const command of Object.values(ipc.commands)) command.mockReset().mockResolvedValue(undefined);
  ipc.commands.canonMap.mockResolvedValue(MAP);
  ipc.commands.canonReach.mockResolvedValue(REACH);
  ipc.commands.openFile.mockResolvedValue({ kind: "text", text: "a" });
  shell.setState({ ...shell.getInitialState(), toolsOpen: true, tool: "map", tabs: ["map"] }, true);
});

afterEach(() => {
  cleanup();
  document.body.replaceChildren();
});

describe("map panel", () => {
  it("waits for the index, then shows the central files, the areas and what is out of place", async () => {
    ipc.commands.canonMap.mockResolvedValueOnce(null);
    render(<MapPanel />);
    await act(() => loadMap());
    expect(screen.getByText("Sens está indexando el proyecto…")).toBeTruthy();
    await act(() => loadMap());
    render(<MapTally />);
    expect(screen.getByText("2 áreas · 3 ficheros")).toBeTruthy();
    const central = screen.getByRole("region", { name: "Ficheros centrales" });
    expect(within(central).getByText("4 dependen de él")).toBeTruthy();
    expect(within(screen.getByRole("region", { name: "Fuera de sitio" })).getByText("→ lib")).toBeTruthy();
    const area = screen.getByText("src", { selector: ".region-name" }).closest("details") as HTMLDetailsElement;
    act(() => {
      area.open = true;
      area.dispatchEvent(new Event("toggle"));
    });
    expect(within(area).getByText("runner.ts", { selector: ".name" }).closest(".map-file")?.getAttribute("data-door")).toBe("true");
    expect(within(area).getByText("lib")).toBeTruthy();
  });

  it("shows what depends on a file by steps, and goes back to the map", async () => {
    render(<MapPanel />);
    await act(() => loadMap());
    await act(async () => fireEvent.click(within(screen.getByRole("region", { name: "Ficheros centrales" })).getByTitle("Lo que depende de él: lib/format.ts")));
    expect(ipc.commands.canonReach).toHaveBeenCalledWith("C:/demo", "lib/format.ts");
    const reach = screen.getByRole("region", { name: "Lo que depende de este fichero" });
    expect(within(reach).getByText("1 paso")).toBeTruthy();
    expect(within(reach).getByText("2 pasos")).toBeTruthy();
    expect(screen.getByText("Ningún test lo alcanza.")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Volver al mapa" }));
    expect(screen.getByRole("region", { name: "Áreas" })).toBeTruthy();
  });

  it("opens a file in Files and keeps the map as a tab", async () => {
    render(<MapPanel />);
    await act(() => loadMap());
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Abrir en Ficheros: lib/format.ts" })));
    expect(shell.getState()).toMatchObject({ tool: "files", tabs: ["map", "files"] });
  });
});
