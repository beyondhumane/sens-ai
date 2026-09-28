import { describe, expect, it } from "vitest";
import type { Ask, Part, Reply, Turn, You } from "../features/chat/turns";
import { lastQuestion, phaseOf, spentOf } from "./store";

const you = (text: string): You => ({ kind: "you", key: 1, text, files: [], pictures: [] });

const reply = (parts: Part[], closed = false): Reply => ({ kind: "reply", key: 2, who: "claude", parts, open: null, working: "", began: 0, closed });

const said = (text: string): Part => ({ kind: "said", key: 3, text, streamed: true, settled: false, done: false });

const read = (key: number): Part => ({ kind: "step", key, id: `t${key}`, name: "Read", input: {} as never, state: "done", output: "", detail: null, links: [], began: 0, ended: 10 });

const ask = (active: boolean): Ask => ({ kind: "ask", key: 4, event: {} as never, active, state: active ? "waiting" : "allowed", answers: null });

describe("the phase the seam shows", () => {
  it("rests when there is nothing, and straightens while you type", () => {
    expect(phaseOf([], false, "")).toBe("idle");
    expect(phaseOf([], false, "  ")).toBe("idle");
    expect(phaseOf([], false, "arregla el login")).toBe("typing");
  });

  it("runs while it works, stops on a question, and bends once text arrives", () => {
    const turns: Turn[] = [you("hola"), reply([read(5)])];
    expect(phaseOf(turns, true, "")).toBe("working");
    expect(phaseOf([you("hola"), reply([read(5), ask(true)])], true, "")).toBe("waiting");
    expect(phaseOf([you("hola"), reply([read(5), ask(false), said("Listo")])], true, "")).toBe("answer");
  });

  it("ends as done, or as an error when the reply broke", () => {
    expect(phaseOf([you("hola"), reply([said("Listo")], true)], false, "")).toBe("done");
    expect(phaseOf([you("hola"), reply([{ kind: "fault", key: 9, text: "sin red" }], true)], false, "")).toBe("error");
  });
});

describe("what the footer tells", () => {
  it("counts the files read and the time the turn took", () => {
    const parts: Part[] = [read(5), read(6), said("Listo"), { kind: "foot", key: 7, millis: 3800, tokens: 10, stopped: false }];
    expect(spentOf(reply(parts, true))).toEqual({ reads: 2, millis: 3800 });
    expect(spentOf(undefined)).toEqual({ reads: 0, millis: 0 });
  });

  it("offers the last question you asked as the placeholder", () => {
    expect(lastQuestion([you("primera"), reply([], true), you("segunda"), reply([], true)])).toBe("segunda");
    expect(lastQuestion([you("primera"), you("  ")])).toBe("primera");
    expect(lastQuestion([])).toBe("");
  });
});
