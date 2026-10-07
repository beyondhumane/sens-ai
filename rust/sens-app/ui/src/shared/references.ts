import { knownFile } from "./fileIcons";

export interface Place {
  path: string;
  line: number;
}

export interface Pull {
  ref: string;
  url: string;
}

const PLACE = /^((?:[a-z]:)?[\w.@~+\-\/]+?)(?::(\d+))?(?::\d+)?$/i;
const PULL = /\b([a-z\d][\w.-]*\/[\w.-]+)#(\d+)\b/gi;
const PULL_URL = /^https:\/\/github\.com\/[^/]+\/[^/]+\/(?:pull|issues)\/\d+\/?$/i;

export function placeOf(text: string): Place | null {
  const found = PLACE.exec(text.trim());
  if (!found || found[1].includes("://") || /[\\/]$/.test(found[1]) || !/[.\\/]/.test(found[1]) || !knownFile(found[1])) return null;
  return { path: found[1], line: Number(found[2] ?? 0) };
}

export function pullsIn(text: string): (string | Pull)[] {
  const pieces: (string | Pull)[] = [];
  let from = 0;
  for (const found of text.matchAll(PULL)) {
    if (found.index > from) pieces.push(text.slice(from, found.index));
    pieces.push({ ref: found[0], url: `https://github.com/${found[1]}/issues/${found[2]}` });
    from = found.index + found[0].length;
  }
  if (from < text.length) pieces.push(text.slice(from));
  return pieces;
}

export const isPullUrl = (url: string) => PULL_URL.test(url);

let opener: ((place: Place) => unknown) | null = null;

export const opensPlaces = (open: (place: Place) => unknown) => void (opener = open);

export const placesOpen = () => opener !== null;

export const openPlace = (place: Place) => opener?.(place);
