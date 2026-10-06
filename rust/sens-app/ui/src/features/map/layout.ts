import type { AreaLink } from "../../ipc/types";

export const NODE_HEIGHT = 44;
const ROW = 96;
const GAP = 24;
const MARGIN = 40;
const CHAR = 7;
const PAD = 28;
const LEAST = 96;

export interface Placed {
  name: string;
  label: string;
  x: number;
  y: number;
  width: number;
}

export interface Laid {
  nodes: Placed[];
  width: number;
  height: number;
}

export const shortName = (name: string) => name.split("/").slice(-2).join("/");

function layers(names: string[], uses: Map<string, string[]>) {
  const found = new Map<string, number>();
  const visiting = new Set<string>();
  const depth = (name: string): number => {
    const known = found.get(name);
    if (known !== undefined) return known;
    visiting.add(name);
    const below = (uses.get(name) ?? []).filter((used) => !visiting.has(used)).map(depth);
    visiting.delete(name);
    const layer = below.length ? 1 + Math.max(...below) : 0;
    found.set(name, layer);
    return layer;
  };
  for (const name of names) depth(name);
  return found;
}

export function layered(names: string[], links: AreaLink[]): Laid {
  const known = new Set(names);
  const uses = new Map<string, string[]>();
  for (const link of links) {
    if (!known.has(link.from) || !known.has(link.to) || link.from === link.to) continue;
    uses.set(link.from, [...(uses.get(link.from) ?? []), link.to]);
  }
  const layerOf = layers(names, uses);
  const deepest = Math.max(0, ...layerOf.values());
  const rows: string[][] = Array.from({ length: deepest + 1 }, () => []);
  for (const name of names) rows[layerOf.get(name)!].push(name);

  const order = new Map<string, number>();
  rows.forEach((row) => {
    const pull = (name: string) => {
      const below = (uses.get(name) ?? []).map((used) => order.get(used)).filter((at): at is number => at !== undefined);
      return below.length ? below.reduce((sum, at) => sum + at, 0) / below.length : Number.POSITIVE_INFINITY;
    };
    row.sort((a, b) => pull(a) - pull(b) || names.indexOf(a) - names.indexOf(b));
    row.forEach((name, at) => order.set(name, at));
  });

  const widthOf = (name: string) => Math.max(LEAST, shortName(name).length * CHAR + PAD);
  const rowWidth = (row: string[]) => row.reduce((sum, name) => sum + widthOf(name), 0) + GAP * Math.max(0, row.length - 1);
  const width = Math.max(...rows.map(rowWidth)) + MARGIN * 2;
  const nodes: Placed[] = [];
  rows.forEach((row, layer) => {
    let x = (width - rowWidth(row)) / 2;
    const y = MARGIN + (deepest - layer) * ROW;
    for (const name of row) {
      nodes.push({ name, label: shortName(name), x, y, width: widthOf(name) });
      x += widthOf(name) + GAP;
    }
  });
  return { nodes, width, height: MARGIN * 2 + deepest * ROW + NODE_HEIGHT };
}
