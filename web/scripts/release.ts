import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { links, releaseApi, repo } from "../src/content/site.ts";
import { creditsOf, headlineOf, notesOf, releaseFrom, type Notes, type Release } from "../src/content/release.ts";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const latestFile = path.join(root, "src", "data", "release.json");
const historyFile = path.join(root, "src", "data", "releases.json");
const api = `https://api.github.com/repos/${repo}`;
const token = process.env.GITHUB_TOKEN ?? process.env.GH_TOKEN;

const fallback: Release = { version: null, published: null, installer: null, page: links.latest };

const kept = <T>(file: string, otherwise: T): T => (existsSync(file) ? (JSON.parse(readFileSync(file, "utf8")) as T) : otherwise);

async function github(url: string): Promise<unknown> {
  try {
    const response = await fetch(url, {
      headers: {
        accept: "application/vnd.github+json",
        "user-agent": "sens-web",
        ...(token ? { authorization: `Bearer ${token}` } : {}),
      },
      signal: AbortSignal.timeout(15_000),
    });
    return response.ok ? await response.json() : null;
  } catch {
    return null;
  }
}

interface Listed {
  tag_name?: string;
  name?: string;
  body?: string;
  draft?: boolean;
}

async function history(): Promise<Notes[] | null> {
  const listed = await github(`${api}/releases?per_page=100`);
  if (!Array.isArray(listed)) return null;
  const published = (listed as Listed[]).filter((release) => !release.draft && release.tag_name);
  const notes: Notes[] = [];
  for (const [at, payload] of published.entries()) {
    const release = releaseFrom(payload);
    const tag = payload.tag_name;
    if (!release?.version || !tag) continue;
    const previous = published[at + 1]?.tag_name;
    const commits = previous
      ? ((await github(`${api}/compare/${previous}...${tag}`)) as { commits?: unknown } | null)?.commits
      : await github(`${api}/commits?sha=${tag}&per_page=100`);
    notes.push({
      version: release.version,
      headline: headlineOf(payload.name ?? "", tag),
      published: release.published,
      page: release.page,
      installer: release.installer,
      notes: notesOf(payload.body ?? ""),
      credits: creditsOf(commits),
    });
  }
  return notes;
}

const latest = releaseFrom(await github(releaseApi)) ?? kept(latestFile, fallback);
const releases = (await history()) ?? kept<Notes[]>(historyFile, []);

mkdirSync(path.dirname(latestFile), { recursive: true });
writeFileSync(latestFile, `${JSON.stringify(latest, null, 2)}\n`);
writeFileSync(historyFile, `${JSON.stringify(releases, null, 2)}\n`);
console.log(latest.version ? `release · ${latest.version} · ${latest.installer?.name ?? "no installer"}` : "release · unavailable, links go to releases/latest");
console.log(`releases · ${releases.length} with notes and credits`);
