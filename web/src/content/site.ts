export const repo = "beyondhumane/sens-ai";

const github = `https://github.com/${repo}`;

export const links = {
  repo: github,
  releases: `${github}/releases`,
  latest: `${github}/releases/latest`,
  issues: `${github}/issues`,
  readme: `${github}#readme`,
  license: `${github}/blob/main/LICENSE`,
  paper: `${github}/blob/main/docs/paper/sens-canon.md`,
} as const;

export const releaseApi = `https://api.github.com/repos/${repo}/releases/latest`;
