import { describe, expect, it } from "vitest";
import { tokensOf } from "../src/lib/code";

describe("code colours", () => {
  const tokens = tokensOf('export const greeting = "Welcome";', "tsx").flat();
  const colourOf = (content: string) => tokens.find((token) => token.content === content)?.color;

  it("carry the Light+ and the Dark+ colour of every token a grammar names", () => {
    for (const content of ["export", "const", "greeting", '"Welcome"']) {
      expect(colourOf(content)).toMatch(/^light-dark\(#[0-9a-f]{6}, #[0-9a-f]{6}\)$/);
    }
  });

  it("leave plain text to the surface's own text colour", () => {
    expect(colourOf(" ")).toBeNull();
    expect(colourOf(";")).toBeNull();
  });
});
