import { describe, expect, it } from "vitest";
import { DEFAULT_SHORTCUT, onlyModifiers, pressedKeys, sameKeys } from "./shortcut";

const pressed = (over: Partial<KeyboardEvent>) => ({ ctrlKey: false, altKey: false, shiftKey: false, metaKey: false, code: "", key: "", ...over });

describe("the keys pressed for a shortcut", () => {
  it("reads Alt+Space, digits, function keys and letters the way Windows names them", () => {
    expect(pressedKeys(pressed({ altKey: true, code: "Space", key: " " }))).toEqual(DEFAULT_SHORTCUT);
    expect(pressedKeys(pressed({ ctrlKey: true, code: "Digit7", key: "/", shiftKey: true }))?.key).toBe("7");
    expect(pressedKeys(pressed({ metaKey: true, code: "F12", key: "F12" }))).toEqual({ ctrl: false, alt: false, shift: false, win: true, key: "F12" });
    expect(pressedKeys(pressed({ ctrlKey: true, code: "KeyQ", key: "a" }))?.key).toBe("A");
  });

  it("falls back to the key's place when AltGr turns the letter into a symbol", () => {
    expect(pressedKeys(pressed({ ctrlKey: true, altKey: true, code: "KeyE", key: "€" }))?.key).toBe("E");
  });

  it("refuses keys with no Ctrl, Alt or Win, and keys Windows cannot bind", () => {
    expect(pressedKeys(pressed({ shiftKey: true, code: "KeyK", key: "K" }))).toBeNull();
    expect(pressedKeys(pressed({ code: "F5", key: "F5" }))).toBeNull();
    expect(pressedKeys(pressed({ ctrlKey: true, code: "Tab", key: "Tab" }))).toBeNull();
    expect(pressedKeys(pressed({ ctrlKey: true, code: "F25", key: "F25" }))).toBeNull();
  });

  it("waits while only modifiers are held", () => {
    expect(onlyModifiers(pressed({ key: "Control" }))).toBe(true);
    expect(onlyModifiers(pressed({ key: "AltGraph" }))).toBe(true);
    expect(onlyModifiers(pressed({ key: "k" }))).toBe(false);
  });

  it("compares two shortcuts key by key", () => {
    expect(sameKeys(DEFAULT_SHORTCUT, { ...DEFAULT_SHORTCUT })).toBe(true);
    expect(sameKeys(DEFAULT_SHORTCUT, { ...DEFAULT_SHORTCUT, ctrl: true })).toBe(false);
  });
});
