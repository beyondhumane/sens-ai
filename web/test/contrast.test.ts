import { describe, expect, it } from "vitest";
import brand from "../src/brand/brand.json";

type Token = keyof typeof brand.palette;

const channel = (value: number) => {
  const unit = value / 255;
  return unit <= 0.04045 ? unit / 12.92 : ((unit + 0.055) / 1.055) ** 2.4;
};

const luminance = (token: Token) => {
  const hex = brand.palette[token].slice(1);
  const [r, g, b] = [0, 2, 4].map((at) => channel(Number.parseInt(hex.slice(at, at + 2), 16)));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
};

const ratio = (text: Token, ground: Token) => {
  const [light, dark] = [luminance(text), luminance(ground)].sort((a, b) => b - a);
  return (light + 0.05) / (dark + 0.05);
};

const light: { text: [Token, Token][]; marks: [Token, Token][] } = {
  text: [
    ["carbon-950", "paper"],
    ["alloy-600", "paper"],
    ["carbon-950", "bone-100"],
    ["alloy-600", "bone-100"],
    ["carbon-950", "signal-500"],
    ["carbon-950", "signal-200"],
    ["paper", "carbon-950"],
    ["paper", "carbon-800"],
    ["paper", "carbon-700"],
  ],
  marks: [
    ["signal-800", "paper"],
    ["signal-800", "bone-100"],
    ["signal-800", "signal-500"],
  ],
};

const dark: typeof light = {
  text: [
    ["bone-50", "carbon-950"],
    ["alloy-400", "carbon-950"],
    ["bone-50", "carbon-800"],
    ["alloy-400", "carbon-800"],
    ["carbon-950", "bone-50"],
    ["carbon-950", "bone-200"],
    ["carbon-950", "bone-300"],
  ],
  marks: [
    ["signal-500", "carbon-950"],
    ["signal-500", "carbon-800"],
  ],
};

describe.each([
  ["light", light],
  ["dark", dark],
])("contrast of the pairs the page uses in the %s theme", (_, pairs) => {
  for (const [ink, ground] of pairs.text) {
    it(`${ink} on ${ground} reads as text (4.5:1)`, () => {
      expect(ratio(ink, ground)).toBeGreaterThanOrEqual(4.5);
    });
  }

  for (const [ink, ground] of pairs.marks) {
    it(`${ink} on ${ground} holds as a focus ring or mark (3:1)`, () => {
      expect(ratio(ink, ground)).toBeGreaterThanOrEqual(3);
    });
  }
});

describe("Signal as text", () => {
  it("keeps Signal off paper as text: it only reaches about 1.1:1", () => {
    expect(ratio("signal-500", "paper")).toBeLessThan(1.2);
  });
});
