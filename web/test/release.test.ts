import { describe, expect, it } from "vitest";
import { creditsOf, headlineOf, megabytes, notesOf, releaseFrom } from "../src/content/release";

const payload = {
  tag_name: "v0.29.0",
  published_at: "2026-09-29T19:15:39Z",
  html_url: "https://github.com/beyondhumane/sens-ai/releases/tag/v0.29.0",
  assets: [
    { name: "Sens_0.29.0_x64-setup.exe.sig", browser_download_url: "https://example.test/sig", size: 436 },
    { name: "Sens_0.29.0_x64-setup.exe", browser_download_url: "https://example.test/exe", size: 7836672 },
  ],
};

describe("releaseFrom", () => {
  it("reads the version, the date, the page and the installer", () => {
    expect(releaseFrom(payload)).toEqual({
      version: "0.29.0",
      published: "2026-09-29",
      page: payload.html_url,
      installer: { name: "Sens_0.29.0_x64-setup.exe", url: "https://example.test/exe", bytes: 7836672 },
    });
  });

  it("keeps the release without an installer when none is attached", () => {
    expect(releaseFrom({ ...payload, assets: [payload.assets[0]] })?.installer).toBeNull();
  });

  it("gives nothing for an answer that is not a release", () => {
    expect(releaseFrom({ message: "Not Found" })).toBeNull();
    expect(releaseFrom(null)).toBeNull();
  });
});

describe("megabytes", () => {
  it("writes sizes the way the page shows them", () => {
    expect(megabytes(7836672)).toBe("7.8 MB");
  });
});

describe("notesOf", () => {
  it("keeps the notes up to the first alert or the Install heading, as the app does", () => {
    const body = "Sens now checks.\n\n### New\n- One\n\n> [!WARNING]\n> Not signed\n\n### Install\nRun it";
    expect(notesOf(body)).toBe("Sens now checks.\n\n### New\n- One");
    expect(notesOf("### Fixed\n- Two\n\n### Install\nRun it")).toBe("### Fixed\n- Two");
  });
});

describe("headlineOf", () => {
  it("takes the version off the release's name", () => {
    expect(headlineOf("v0.30.0 — Sens checks what the AI writes", "v0.30.0")).toBe("Sens checks what the AI writes");
    expect(headlineOf("v0.29.0", "v0.29.0")).toBeNull();
  });
});

describe("creditsOf", () => {
  it("credits each author and co-author once, linking those with a GitHub profile", () => {
    const commits = [
      { author: { login: "iiTzSenn", html_url: "https://github.com/iiTzSenn" }, commit: { message: "feat: one\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>" } },
      { author: { login: "iiTzSenn", html_url: "https://github.com/iiTzSenn" }, commit: { message: "fix: two\n\nco-authored-by: Claude Opus 5.5 <noreply@anthropic.com>" } },
      { author: null, commit: { author: { name: "Ada" }, message: "docs: three" } },
    ];
    expect(creditsOf(commits)).toEqual([
      { name: "iiTzSenn", profile: "https://github.com/iiTzSenn" },
      { name: "Claude Opus 5.5", profile: null },
      { name: "Ada", profile: null },
    ]);
  });
});
