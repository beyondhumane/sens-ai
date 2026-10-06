export interface Installer {
  name: string;
  url: string;
  bytes: number;
}

export interface Release {
  version: string | null;
  published: string | null;
  installer: Installer | null;
  page: string;
}

interface GithubAsset {
  name?: unknown;
  browser_download_url?: unknown;
  size?: unknown;
}

interface GithubRelease {
  tag_name?: unknown;
  published_at?: unknown;
  html_url?: unknown;
  assets?: unknown;
}

const INSTALLER = /_x64-setup\.exe$/;

const text = (value: unknown): string | null => (typeof value === "string" && value.length > 0 ? value : null);

function installerOf(assets: unknown): Installer | null {
  if (!Array.isArray(assets)) return null;
  const asset = (assets as GithubAsset[]).find((candidate) => INSTALLER.test(text(candidate.name) ?? ""));
  const name = text(asset?.name);
  const url = text(asset?.browser_download_url);
  const bytes = typeof asset?.size === "number" ? asset.size : null;
  return name && url && bytes ? { name, url, bytes } : null;
}

export function releaseFrom(payload: unknown): Release | null {
  const release = (payload ?? {}) as GithubRelease;
  const tag = text(release.tag_name);
  const page = text(release.html_url);
  if (!tag || !page) return null;
  return {
    version: tag.replace(/^v/, ""),
    published: text(release.published_at)?.slice(0, 10) ?? null,
    installer: installerOf(release.assets),
    page,
  };
}

export const megabytes = (bytes: number): string => `${(bytes / 1_000_000).toFixed(1)} MB`;

export interface Credit {
  name: string;
  profile: string | null;
}

export interface Notes {
  version: string;
  headline: string | null;
  published: string | null;
  page: string;
  installer: Installer | null;
  notes: string;
  credits: Credit[];
}

interface GithubCommit {
  author?: { login?: unknown; html_url?: unknown } | null;
  commit?: { author?: { name?: unknown }; message?: unknown };
}

const COAUTHOR = /^co-authored-by:\s*(.+?)\s*<[^>]*>\s*$/gim;
const CUT = /^(?:>\s*\[!|#{1,6}\s+install\b)/im;

export function notesOf(body: string): string {
  const cut = body.search(CUT);
  return (cut === -1 ? body : body.slice(0, cut)).trim();
}

export function headlineOf(name: string, tag: string): string | null {
  const headline = name.replace(tag, "").replace(/^[\s—–:-]+/, "").trim();
  return headline.length > 0 ? headline : null;
}

export function creditsOf(commits: unknown): Credit[] {
  if (!Array.isArray(commits)) return [];
  const credits = new Map<string, Credit>();
  const add = (name: string | null, profile: string | null) => {
    if (name && !credits.has(name.toLowerCase())) credits.set(name.toLowerCase(), { name, profile });
  };
  for (const entry of commits as GithubCommit[]) {
    add(text(entry.author?.login) ?? text(entry.commit?.author?.name), text(entry.author?.html_url));
    for (const match of (text(entry.commit?.message) ?? "").matchAll(COAUTHOR)) add(match[1] ?? null, null);
  }
  return [...credits.values()];
}
