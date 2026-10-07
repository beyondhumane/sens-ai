import type { AreaLink } from "../../ipc/types";

export const NODE_HEIGHT = 48;
const ROW = 112;
const GAP = 32;
const MARGIN = 40;
const CHAR = 7.2;
const PAD = 64;
const LEAST = 120;
const BEND = 0.14;
const TIP = 3;
const CLEAR = 36;

export interface Point {
  x: number;
  y: number;
}

export interface Placed extends Point {
  name: string;
  label: string;
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

  const widthOf = (name: string) => Math.round(Math.max(LEAST, shortName(name).length * CHAR + PAD));
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

const centerOf = (node: Placed): Point => ({ x: node.x + node.width / 2, y: node.y + NODE_HEIGHT / 2 });

function rim(node: Placed, toward: Point, inset = 0): Point {
  const center = centerOf(node);
  const dx = toward.x - center.x;
  const dy = toward.y - center.y;
  if (!dx && !dy) return center;
  const reach = Math.min(dx ? (node.width / 2 + inset) / Math.abs(dx) : Number.POSITIVE_INFINITY, dy ? (NODE_HEIGHT / 2 + inset) / Math.abs(dy) : Number.POSITIVE_INFINITY);
  return { x: center.x + dx * reach, y: center.y + dy * reach };
}

export function linkPath(from: Placed, to: Placed) {
  const a = centerOf(from);
  const b = centerOf(to);
  const length = Math.hypot(b.x - a.x, b.y - a.y) || 1;
  const swing = Math.abs(b.y - a.y) > ROW * 1.5 ? Math.max(from.width, to.width) / 2 + CLEAR : length * BEND;
  const bend = { x: (a.x + b.x) / 2 - ((b.y - a.y) / length) * swing, y: (a.y + b.y) / 2 + ((b.x - a.x) / length) * swing };
  const start = rim(from, bend);
  const end = rim(to, bend, TIP);
  const middle = { x: (start.x + 2 * bend.x + end.x) / 4, y: (start.y + 2 * bend.y + end.y) / 4 };
  return { d: `M ${start.x} ${start.y} Q ${bend.x} ${bend.y} ${end.x} ${end.y}`, middle };
}

export function boundsOf(nodes: Placed[]) {
  if (!nodes.length) return { left: 0, top: 0, width: 0, height: 0 };
  const left = Math.min(...nodes.map((node) => node.x));
  const top = Math.min(...nodes.map((node) => node.y));
  const right = Math.max(...nodes.map((node) => node.x + node.width));
  const bottom = Math.max(...nodes.map((node) => node.y + NODE_HEIGHT));
  return { left, top, width: right - left, height: bottom - top };
}
