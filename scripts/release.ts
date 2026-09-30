import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { links, releaseApi } from "../src/content/site.ts";
import { releaseFrom, type Release } from "../src/content/release.ts";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const target = path.join(root, "src", "data", "release.json");

const fallback: Release = { version: null, published: null, installer: null, page: links.latest };

function kept(): Release {
  return existsSync(target) ? (JSON.parse(readFileSync(target, "utf8")) as Release) : fallback;
}

async function fetched(): Promise<Release | null> {
  try {
    const response = await fetch(releaseApi, {
      headers: { accept: "application/vnd.github+json", "user-agent": "sens-web" },
      signal: AbortSignal.timeout(15_000),
    });
    if (!response.ok) return null;
    return releaseFrom(await response.json());
  } catch {
    return null;
  }
}

const release = (await fetched()) ?? kept();
mkdirSync(path.dirname(target), { recursive: true });
writeFileSync(target, `${JSON.stringify(release, null, 2)}\n`);
console.log(release.version ? `release · ${release.version} · ${release.installer?.name ?? "no installer"}` : "release · unavailable, links go to releases/latest");
