import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Artifact } from "../ipc/types";
import { showLanguage } from "../shared/i18n";
import { Shelf } from "../features/artifacts/Shelf";
import { artifacts, loadShelf } from "../features/artifacts/store";
import { project } from "../features/project/store";

const ipc = vi.hoisted(() => ({
  commands: { artifacts: vi.fn(), artifactData: vi.fn(), artifactText: vi.fn(), openExternal: vi.fn() },
}));

vi.mock("../ipc/commands", () => ({ commands: ipc.commands }));
vi.mock("../app/session", () => ({ resume: vi.fn(), draft: vi.fn(async () => {}), fresh: vi.fn(), chooseFolder: vi.fn(), showView: vi.fn() }));

const artifact = (name: string, over: Partial<Artifact> = {}): Artifact => ({
  kind: "file",
  root: "C:/demo",
  project: "demo",
  name,
  target: `C:/demo/.sens/artifacts/${name}`,
  session: "s1",
  sessionTitle: "Arreglo del chat",
  at: Date.now(),
  bytes: null,
  ...over,
});

beforeEach(() => {
  artifacts.setState(artifacts.getInitialState(), true);
  project.setState({ root: "C:/demo" });
  for (const command of Object.values(ipc.commands)) command.mockReset().mockResolvedValue(undefined);
  ipc.commands.artifacts.mockResolvedValue([
    artifact("informe.pdf", { bytes: 1_258_291 }),
    artifact("notas.txt", { bytes: 3_072 }),
    artifact("vacio.txt", { bytes: 0 }),
    artifact("sin-medida.md"),
    artifact("docs", { kind: "link", target: "https://example.com/docs" }),
  ]);
});

afterEach(() => {
  cleanup();
  showLanguage("es");
});

async function open() {
  render(<Shelf />);
  await act(async () => loadShelf());
}

const said = (name: string) => screen.getByText(name, { selector: ".card .name" }).closest(".card")?.textContent ?? "";

describe("the size of each artifact on the shelf", () => {
  it("follows the project when it is known", async () => {
    await open();
    expect(said("informe.pdf")).toContain("demo · 1,2 MB");
    expect(said("notas.txt")).toContain("demo · 3 KB");
    expect(said("vacio.txt")).toContain("demo · 0 B");
  });

  it("is left out when nobody knows it", async () => {
    await open();
    expect(said("sin-medida.md")).toContain("demo");
    expect(said("sin-medida.md")).not.toContain("demo ·");
    expect(said("docs")).not.toContain("demo ·");
  });

  it("reads in the language of the interface", async () => {
    showLanguage("fr");
    await open();
    expect(said("informe.pdf")).toContain("demo · 1,2 Mo");
    expect(said("notas.txt")).toContain("demo · 3 Ko");
  });
});
