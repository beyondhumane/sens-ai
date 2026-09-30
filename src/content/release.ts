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
