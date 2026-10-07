import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Markdown } from "./markdown/Markdown";
import { opensPlaces, placeOf, pullsIn } from "./references";

afterEach(cleanup);

describe("references in a reply", () => {
  it("knows a file by its name, with or without a line, and nothing else", () => {
    expect(placeOf("src/app.tsx:12")).toEqual({ path: "src/app.tsx", line: 12 });
    expect(placeOf("README.md")).toEqual({ path: "README.md", line: 0 });
    expect(placeOf("C:\Proyectos\demo\main.py")).toEqual({ path: "C:\Proyectos\demo\main.py", line: 0 });
    for (const text of ["v0.33.0", "useStore.getState", "npm run dev", "changes", "src/", "https://x.dev/a.js", "sens.arrangement"]) expect(placeOf(text)).toBeNull();
  });

  it("cuts a pull request reference out of the text", () => {
    expect(pullsIn("ver beyondhumane/sens-ai#17 y #3")).toEqual([
      "ver ",
      { ref: "beyondhumane/sens-ai#17", url: "https://github.com/beyondhumane/sens-ai/issues/17" },
      " y #3",
    ]);
  });

  it("draws a file as a chip that opens it, and a pull request as a link", () => {
    const open = vi.fn();
    opensPlaces(open);
    render(<Markdown text="Mira `src/app.tsx:12`, `npm test` y beyondhumane/sens-ai#18." />);
    fireEvent.click(screen.getByRole("button", { name: "src/app.tsx:12" }));
    expect(open).toHaveBeenCalledWith({ path: "src/app.tsx", line: 12 });
    expect(screen.getByText("npm test").tagName).toBe("CODE");
    expect(screen.getByRole("link", { name: "beyondhumane/sens-ai#18" }).getAttribute("href")).toBe("https://github.com/beyondhumane/sens-ai/issues/18");
  });
});
