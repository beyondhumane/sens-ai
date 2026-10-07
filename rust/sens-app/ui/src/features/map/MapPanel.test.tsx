import { act, cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { panelShows, shell, showTool } from "../../app/shell";
import { project } from "../project/store";
import { MapPanel, MapTally } from "./MapPanel";
import { atlas, loadMap, showMapAs } from "./store";

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
  links: [{ from: "src", to: "lib", weight: 3 }],
  cycles: [],
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
  shell.setState(shell.getInitialState(), true);
  showTool("map");
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

  it("draws the areas as a graph, and picking one lights its links and shows its files", async () => {
    showMapAs("graph");
    render(<MapPanel />);
    await act(() => loadMap());
    const graph = screen.getByRole("group", { name: "Grafo de las áreas" });
    const src = within(graph).getByRole("button", { name: "src · 2 ficheros" });
    expect(screen.getByText("Pulsa un área para ver sus ficheros y lo que usa.")).toBeTruthy();
    fireEvent.keyDown(src, { key: "Enter" });
    expect(src.getAttribute("aria-pressed")).toBe("true");
    expect(graph.querySelector('.map-edge[data-lit="true"] title')?.textContent).toBe("src usa lib · 3 enlaces");
    expect(screen.getByText("runner.ts", { selector: ".name" })).toBeTruthy();
    fireEvent.click(src);
    expect(src.getAttribute("aria-pressed")).toBe("false");
    expect(JSON.parse(localStorage.getItem("sens.map.mode")!)).toBe("graph");
    showMapAs("list");
  });

  it("tells what a picked area uses and what uses it, and walks to either", async () => {
    showMapAs("graph");
    render(<MapPanel />);
    await act(() => loadMap());
    fireEvent.click(screen.getByRole("button", { name: "src · 2 ficheros" }));
    const src = screen.getByRole("region", { name: "src" });
    expect(within(src).getByText("Usa")).toBeTruthy();
    expect(within(src).queryByText("La usan")).toBeNull();
    fireEvent.click(within(src).getByRole("button", { name: "src usa lib · 3 enlaces" }));
    expect(atlas.getState().picked).toBe("lib");
    const lib = screen.getByRole("region", { name: "lib" });
    expect(within(lib).getByText("La usan")).toBeTruthy();
    expect(within(lib).getByRole("button", { name: "src usa lib · 3 enlaces" })).toBeTruthy();
    expect(document.querySelector('.map-edge[data-way="in"]')).toBeTruthy();
    showMapAs("list");
  });

  it("zooms with its buttons and puts moved areas back", async () => {
    showMapAs("graph");
    render(<MapPanel />);
    await act(() => loadMap());
    const zoom = () => document.querySelector(".map-zoom")!.textContent;
    const before = zoom();
    fireEvent.click(screen.getByRole("button", { name: "Acercar" }));
    expect(zoom()).not.toBe(before);
    expect(screen.queryByRole("button", { name: "Recolocar las áreas" })).toBeNull();
    const stage = screen.getByRole("application", { name: /^Flechas o arrastrar para moverte/ });
    const node = screen.getByRole("button", { name: "lib · 1 fichero" });
    fireEvent.pointerDown(node, { button: 0, clientX: 10, clientY: 10, pointerId: 1 });
    stage.setPointerCapture = () => {};
    fireEvent.pointerMove(stage, { clientX: 80, clientY: 60, pointerId: 1 });
    fireEvent.pointerUp(stage, { pointerId: 1 });
    fireEvent.click(node);
    expect(atlas.getState().picked).toBe("");
    fireEvent.click(screen.getByRole("button", { name: "Recolocar las áreas" }));
    expect(screen.queryByRole("button", { name: "Recolocar las áreas" })).toBeNull();
    showMapAs("list");
  });

  it("marks the areas with an import cycle and lists the files in it", async () => {
    ipc.commands.canonMap.mockResolvedValue({ ...MAP, cycles: [["src/app.ts", "src/runner.ts"]] });
    showMapAs("graph");
    render(<MapPanel />);
    await act(() => loadMap());
    expect(screen.getByRole("button", { name: "src · 2 ficheros · tiene un ciclo de imports" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "lib · 1 fichero" })).toBeTruthy();
    const cycles = screen.getByRole("region", { name: "Ciclos de imports" });
    expect(within(cycles).getAllByText(/\.ts$/, { selector: ".name" }).map((name) => name.textContent)).toEqual(["app.ts", "runner.ts"]);
    showMapAs("list");
  });

  it("opens a file in Files and keeps the map as a tab", async () => {
    render(<MapPanel />);
    await act(() => loadMap());
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Abrir en Ficheros: lib/format.ts" })));
    expect(panelShows("files")).toBe(true);
    expect(shell.getState().tabs).toEqual(["map", "files"]);
  });
});
