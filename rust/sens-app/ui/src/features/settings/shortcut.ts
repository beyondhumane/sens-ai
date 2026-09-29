import type { ShortcutKeys } from "../../ipc/types";

export const DEFAULT_SHORTCUT: ShortcutKeys = { ctrl: false, alt: true, shift: false, win: false, key: "Space" };

type Pressed = Pick<KeyboardEvent, "ctrlKey" | "altKey" | "shiftKey" | "metaKey" | "code" | "key">;

const MODIFIERS = new Set(["Control", "Alt", "AltGraph", "Shift", "Meta", "OS"]);

export const onlyModifiers = (pressed: Pressed) => MODIFIERS.has(pressed.key);

function keyOf({ code, key }: Pressed) {
  if (code === "Space") return "Space";
  if (/^Digit\d$/.test(code)) return code.slice("Digit".length);
  if (/^F([1-9]|1\d|2[0-4])$/.test(code)) return code;
  if (/^[a-z]$/i.test(key)) return key.toUpperCase();
  if (/^Key[A-Z]$/.test(code)) return code.slice("Key".length);
  return null;
}

export function pressedKeys(pressed: Pressed): ShortcutKeys | null {
  const key = keyOf(pressed);
  if (!key || !(pressed.ctrlKey || pressed.altKey || pressed.metaKey)) return null;
  return { ctrl: pressed.ctrlKey, alt: pressed.altKey, shift: pressed.shiftKey, win: pressed.metaKey, key };
}

export const sameKeys = (one: ShortcutKeys, other: ShortcutKeys) =>
  one.ctrl === other.ctrl && one.alt === other.alt && one.shift === other.shift && one.win === other.win && one.key === other.key;
