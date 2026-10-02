import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ChangesPanel } from "../features/changes/Changes";
import { changes, loadChanges } from "../features/changes/store";
import { ViewerHead } from "../features/files/Viewer";
import { present, viewer } from "../features/files/view";
import { project } from "../features/project/store";

const ipc = vi.hoisted(() => ({ commands: { changes: vi.fn(), openFile: vi.fn(), folder: vi.fn(), findFiles: vi.fn() } }));

vi.mock("../ipc/commands", () => ({ commands: ipc.commands }));

const DIFF = ["diff --git a/src/app.js b/src/app.js", "--- a/src/app.js", "+++ b/src/app.js", "@@ -1 +1 @@", "-old", "+new"].join("\n");

beforeEach(() => {
  project.setState({ root: "C:/demo", work: "C:/demo", session: "s1", touched: new Map() });
  viewer.setState(viewer.getInitialState(), true);
  changes.setState(changes.getInitialState(), true);
  for (const command of Object.values(ipc.commands)) command.mockReset().mockResolvedValue(undefined);
  ipc.commands.changes.mockResolvedValue({ diff: DIFF, fresh: ["notes.md"] });
});

afterEach(() => {
  cleanup();
  document.body.replaceChildren();
});

const head = (title: string, text: string) => {
  act(() => present(title, text, "C:/demo"));
  return render(<ViewerHead />).container.textContent ?? "";
};

const row = (name: string) => screen.getByText(name, { selector: ".change .name" }).closest("details")?.textContent ?? "";

describe("the language of a file", () => {
  it("shows in the viewer's header", () => {
    expect(head("src/app.ts", "export const app = 1;\n")).toContain("TypeScript");
  });

  it("is found for a script without an extension by the program it runs", () => {
    expect(head("tools/run", "#!/usr/bin/env python3\nprint('hola')\n")).toContain("Python");
  });

  it("is not made up when the file is not recognized", () => {
    expect(head("notas.xyz", "hola\n")).toBe("notas.xyz");
  });

  it("shows on each changed file", async () => {
    render(<ChangesPanel />);
    await act(async () => loadChanges());
    expect(row("app.js")).toContain("JavaScript");
    expect(row("notes.md")).toContain("Markdown");
  });
});
