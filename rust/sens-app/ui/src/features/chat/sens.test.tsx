import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Finding } from "../../ipc/types";
import { showLanguage } from "../../shared/i18n";
import { focused } from "../panes/store";
import { project } from "../project/store";
import { Ask } from "./Ask";
import { Held } from "./Held";
import { Run } from "./Run";
import { SensStep } from "./SensStep";
import { heard, opening, type Sens, type Step } from "./turns";
import { grouped, tally, type Work } from "./work";

const ipc = vi.hoisted(() => ({
  commands: {
    openFile: vi.fn(),
    chatAnswer: vi.fn(),
    canonHeld: vi.fn(),
    canonAccept: vi.fn(),
    canonUndo: vi.fn(),
    canonFix: vi.fn(),
    canonRetry: vi.fn(),
  },
}));
const composer = vi.hoisted(() => ({ sendPlain: vi.fn() }));

vi.mock("../../ipc/commands", () => ({ commands: ipc.commands, events: {} }));
vi.mock("../composer/store", async (original) => ({ ...(await original<typeof import("../composer/store")>()), sendPlain: composer.sendPlain }));

const copy: Finding = {
  rule: "R1",
  severity: "Block",
  file: "src/report.ts",
  line: 1,
  message: "Call `totals` instead of writing it again.",
  target: { symbol: "totals", file: "src/lib/totals.ts", line: 1, signature: "export function totals(rows: Row[])", excerpt: "export function totals(rows: Row[]) {" },
  key: "R1:src/report.ts:summarize~src/lib/totals.ts:totals",
};

const sens = (stage: string, findings: Finding[] = [], suggestions: Sens["suggestions"] = []): Sens => ({ kind: "sens", key: Math.random(), stage, findings, suggestions });

const edit = (content: string): Step => ({
  kind: "step",
  key: Math.random(),
  id: "w1",
  name: "Write",
  input: { file_path: "C:/demo/src/report.ts", content },
  state: "done",
  output: "",
  detail: null,
  links: [],
  began: null,
  ended: null,
});

beforeEach(() => {
  project.setState({ root: "C:/demo", work: "C:/demo" });
  focused().desk.setState({ root: "C:/demo", session: "s1", worktree: null });
  for (const command of Object.values(ipc.commands)) command.mockReset().mockResolvedValue(undefined);
  composer.sendPlain.mockReset().mockResolvedValue(undefined);
});

afterEach(() => {
  cleanup();
  showLanguage("es");
});

describe("what Sens does in a reply", () => {
  it("turns its events into steps that stay with the work, and a held turn into a part of its own", () => {
    let reply = opening();
    reply = heard(reply, { kind: "canon", stage: "anticipated", findings: [], suggestions: [{ name: "totals", file: "src/lib/totals.ts", line: 1, signature: "export function totals(rows)", uses: 4 }] }, true);
    reply = heard(reply, { kind: "tool", id: "w1", name: "Write", input: { file_path: "C:/demo/src/report.ts", content: "totals(rows)" } }, true);
    reply = heard(reply, { kind: "canon", stage: "write", findings: [copy], suggestions: [] }, true);
    reply = heard(reply, { kind: "held", findings: [copy] }, true);
    expect(reply.parts.map((part) => part.kind)).toEqual(["sens", "step", "sens", "held"]);
    expect(grouped(reply.parts).map((piece) => piece.kind)).toEqual(["run", "held"]);
  });

  it("shows a write Sens stopped as stopped, not failed, and leaves out a review with nothing to say", () => {
    let reply = opening();
    reply = heard(reply, { kind: "tool", id: "w1", name: "Write", input: { file_path: "C:/demo/src/report.ts", content: "x" } }, true);
    reply = heard(reply, { kind: "toolDone", id: "w1", output: "Sens stopped this:\n1. Call `totals`", error: true, detail: null }, true);
    reply = heard(reply, { kind: "canon", stage: "reviewed", findings: [], suggestions: [] }, true);
    expect(reply.parts.map((part) => (part.kind === "step" ? part.state : part.kind))).toEqual(["stopped"]);
    expect(tally(reply.parts as Work[]).failed).toBe(0);
  });

  it("counts what it stopped, what was reused from what it offered, and whether it approved", () => {
    const work: Work[] = [
      sens("anticipated", [], [{ name: "totals", file: "src/lib/totals.ts", line: 1, signature: "", uses: 4 }, { name: "never", file: "src/x.ts", line: 1, signature: "", uses: 1 }]),
      sens("write", [copy]),
      edit("import { totals } from './lib/totals';\nexport const report = () => totals([]);"),
      sens("reviewed", [{ ...copy, rule: "S4", severity: "Block", key: "S4:x" }]),
      sens("passed"),
    ];
    expect(tally(work)).toMatchObject({ stops: 1, reused: 1, approved: true, edits: 1 });
  });

  it("says in the folded line what Sens did, lighting only what was reused and approved", () => {
    const work: Work[] = [sens("write", [copy]), edit("totals([])"), edit("a"), edit("b"), sens("passed")];
    const drawn = render(<Run parts={work} folded />).container;
    expect(drawn.querySelector(".run-sens")?.textContent).toBe("Sens: 1 parada · 1 reutilizado · aprobado");
    expect(drawn.querySelector(".run-sens-lit")?.textContent).toBe("1 reutilizado · aprobado");
  });

  it("draws each stage with its own meaning", () => {
    const step = (part: Sens) => render(<SensStep part={part} />).container.querySelector("details.step.sens") as HTMLElement;
    const offered = step(sens("anticipated", [], [{ name: "weigh", file: "src/format.js", line: 3, signature: "export const weigh = (bytes)", uses: 7 }]));
    expect(offered.dataset.tone).toBe("found");
    expect(offered.querySelector(".step-target")?.textContent).toBe("ya tiene weigh");
    fireEvent.click(offered.querySelector("summary")!);
    expect(offered.querySelector(".sens-uses")?.textContent).toBe("usado 7 veces");
    const stopped = step(sens("write", [copy]));
    expect([stopped.dataset.tone, stopped.querySelector(".step-target")?.textContent]).toEqual(["stop", "paró una escritura"]);
    expect(stopped.querySelector(".sens-rule")?.textContent).toContain("Copia código que ya existe");
    expect(step(sens("passed")).dataset.tone).toBe("pass");
    expect(step(sens("reviewed")).querySelector(".step-target")?.textContent).toBe("revisó los cambios");
  });
});

describe("a held turn", () => {
  const held = { kind: "held" as const, key: 9, findings: [copy] };

  it("offers to fix, undo or accept while Sens still holds it, and says what each did", async () => {
    ipc.commands.canonHeld.mockResolvedValue([copy]);
    ipc.commands.canonUndo.mockResolvedValue({ restored: ["src/report.ts", "src/a.ts"], skipped: ["src/b.ts"] });
    await act(async () => void render(<Held part={held} />));
    expect(screen.getByText("Sens no aprueba estos cambios")).toBeTruthy();
    expect(screen.getAllByRole("button").map((button) => button.textContent)).toEqual(expect.arrayContaining(["Pedir que los arregle", "Deshacer", "Aceptar"]));
    await act(async () => void fireEvent.click(screen.getByText("Deshacer")));
    expect(ipc.commands.canonUndo).toHaveBeenCalledWith("C:/demo");
    expect(screen.getByText("Deshecho · 2 ficheros restaurados · 1 fichero se queda porque lo editaste")).toBeTruthy();
    expect(screen.queryByText("Aceptar")).toBeNull();
  });

  it("hands the findings back to Claude when asked to fix them", async () => {
    ipc.commands.canonHeld.mockResolvedValue([copy]);
    ipc.commands.canonFix.mockResolvedValue("Sens held your last turn. Fix what it found");
    await act(async () => void render(<Held part={held} />));
    await act(async () => void fireEvent.click(screen.getByText("Pedir que los arregle")));
    expect(composer.sendPlain).toHaveBeenCalledWith("Sens held your last turn. Fix what it found", focused());
    expect(screen.getByText("Pedido el arreglo a Claude")).toBeTruthy();
  });

  it("offers to judge again when Sens could not, and shows as resolved once it is no longer held", async () => {
    ipc.commands.canonHeld.mockResolvedValue([{ ...copy, key: "unjudged", message: "Sens's reviewer could not judge this turn: no network" }]);
    await act(async () => void render(<Held part={{ kind: "held", key: 3, findings: [{ ...copy, key: "unjudged", message: "Sens's reviewer could not judge this turn: no network" }] }} />));
    expect(screen.getByText("Sens no pudo juzgar estos cambios")).toBeTruthy();
    await act(async () => void fireEvent.click(screen.getByText("Revisar otra vez")));
    expect(ipc.commands.canonRetry).toHaveBeenCalledWith("s1");
    cleanup();
    ipc.commands.canonHeld.mockResolvedValue(null);
    await act(async () => void render(<Held part={held} />));
    expect(screen.getByText("Resuelto")).toBeTruthy();
    expect(screen.queryByText("Aceptar")).toBeNull();
  });
});

describe("a question from Sens", () => {
  it("names the dependency it is about and answers through the chat", async () => {
    const part = { kind: "ask" as const, key: 1, active: true, state: "waiting" as const, answers: null, event: { kind: "asking" as const, request: "sens-canon-0", tool: "sens.dependency", input: { message: "moment is new", file: "package.json", key: "R3:moment" }, suggestions: [] } };
    render(<Ask part={part} reply={1} />);
    expect(screen.getByText("Sens pregunta por una dependencia nueva")).toBeTruthy();
    expect(screen.getByText("moment")).toBeTruthy();
    expect(screen.getByText("Claude la añadió al manifiesto del proyecto.")).toBeTruthy();
    await act(async () => void fireEvent.click(screen.getByText("Rechazar")));
    expect(ipc.commands.chatAnswer).toHaveBeenCalledWith("s1", "sens-canon-0", { allow: false });
  });
});
