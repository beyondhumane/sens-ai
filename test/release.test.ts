import { describe, expect, it } from "vitest";
import { megabytes, releaseFrom } from "../src/content/release";

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
