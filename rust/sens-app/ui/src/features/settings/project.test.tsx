import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { showLanguage } from "../../shared/i18n";
import { project } from "../project/store";
import { ProjectSection } from "./ProjectSection";

const ipc = vi.hoisted(() => ({
  commands: { canonRules: vi.fn(), canonSetRules: vi.fn(), canonExceptions: vi.fn(), canonRetract: vi.fn(), canonAvoided: vi.fn() },
}));

vi.mock("../../ipc/commands", () => ({ commands: ipc.commands, events: {} }));

const nothing = { copies: 0, comments: 0, dependencies: 0, protected: 0, tests: 0, orphans: 0, cycles: 0, judgment: 0, held: 0, accepted: 0, reviews: 0, reviewerCost: 0 };

beforeEach(() => {
  project.setState({ root: "C:/demo", work: "C:/demo/.sens/worktrees/ab12" });
  for (const command of Object.values(ipc.commands)) command.mockReset().mockResolvedValue(undefined);
  ipc.commands.canonRules.mockResolvedValue({ noComments: false });
  ipc.commands.canonExceptions.mockResolvedValue([{ key: "R1:src/a.ts:a~src/b.ts:b", rule: "R1", file: "src/a.ts", since: Date.UTC(2026, 8, 30, 10) }]);
  ipc.commands.canonAvoided.mockResolvedValue({ ...nothing, copies: 3, dependencies: 1, cycles: 2, reviews: 2, reviewerCost: 0.012 });
});

afterEach(() => {
  cleanup();
  showLanguage("es");
});

const shown = async () => {
  await act(async () => void render(<ProjectSection />));
};

describe("the project section", () => {
  it("reads the rules, exceptions and what Sens avoided in the session folder", async () => {
    await shown();
    expect(ipc.commands.canonRules).toHaveBeenCalledWith("C:/demo/.sens/worktrees/ab12");
    expect(screen.getByText("Copia código que ya existe")).toBeTruthy();
    expect(screen.getByText("src/a.ts")).toBeTruthy();
    expect(screen.getByText("3 copias paradas")).toBeTruthy();
    expect(screen.getByText("1 dependencia rechazada")).toBeTruthy();
    expect(screen.getByText("2 ciclos de imports parados")).toBeTruthy();
    expect(screen.getByText("2 revisiones · $0.01")).toBeTruthy();
  });

  it("turns the rule on and retracts an exception", async () => {
    await shown();
    await act(async () => void fireEvent.click(screen.getByRole("switch")));
    expect(ipc.commands.canonSetRules).toHaveBeenCalledWith("C:/demo/.sens/worktrees/ab12", { noComments: true });
    expect(screen.getByRole("switch").getAttribute("aria-checked")).toBe("true");
    await act(async () => void fireEvent.click(screen.getByText("Retirar")));
    expect(ipc.commands.canonRetract).toHaveBeenCalledWith("C:/demo/.sens/worktrees/ab12", "R1:src/a.ts:a~src/b.ts:b");
    expect(screen.getByText("Sin excepciones.")).toBeTruthy();
  });

  it("says so when nothing has been stopped or no project is open", async () => {
    ipc.commands.canonAvoided.mockResolvedValue(nothing);
    await shown();
    expect(screen.getByText("Sens aún no ha parado nada en este proyecto.")).toBeTruthy();
    cleanup();
    project.setState({ root: "", work: "" });
    await shown();
    expect(screen.getByText("Abre un proyecto para ver cómo lo vigila Sens.")).toBeTruthy();
  });
});
