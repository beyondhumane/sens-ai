import { describe, expect, it } from "vitest";
import headline from "../src/data/headline.json";
import { home } from "../src/i18n/home";
import { locales, pathFor } from "../src/i18n/locales";
import { research } from "../src/i18n/research";

describe.each(locales)("the %s page", (locale) => {
  const page = home(locale);
  const paper = research(locale);

  it("tells every chapter, figure and check the English page tells", () => {
    expect(page.chapters).toHaveLength(home("en").chapters.length);
    expect(page.measured).toHaveLength(home("en").measured.length);
    expect(page.checks).toHaveLength(home("en").checks.length);
  });

  it("keeps the research's lists the same length as the English ones", () => {
    const english = research("en");
    for (const key of ["figures", "stages", "shortcuts", "calibration", "testBatches", "testArms", "changeRules", "reviewRules", "blocks", "limits", "tasks"] as const) {
      expect(paper[key], key).toHaveLength(english[key].length);
    }
  });

  it("has its headline measured, so it fits beside the window", () => {
    expect(headline[locale].line).toBeGreaterThan(0);
    expect(headline[locale].word).toBeGreaterThan(0);
  });
});

describe("pathFor", () => {
  it("keeps English at the root and puts every other language under its code", () => {
    expect(pathFor("en", "/research")).toBe("/research");
    expect(pathFor("ja", "/research")).toBe("/ja/research");
    expect(pathFor("es", "/")).toBe("/es/");
  });
});
