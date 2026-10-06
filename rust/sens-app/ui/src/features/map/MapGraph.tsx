import { useMemo, type KeyboardEvent } from "react";
import { useStore } from "zustand";
import type { AreaLink, ProjectMap } from "../../ipc/types";
import { Icon } from "../../shared/Icon";
import { ICONS } from "../../shared/icons.js";
import { t } from "./copy";
import { layered, NODE_HEIGHT, type Placed } from "./layout";
import { atlas, pickArea } from "./store";

const ARC = 36;
const SWING = 56;

function curve(from: Placed, to: Placed) {
  const x1 = from.x + from.width / 2;
  const x2 = to.x + to.width / 2;
  if (from.y === to.y) return `M ${x1} ${from.y} C ${x1} ${from.y - ARC}, ${x2} ${to.y - ARC}, ${x2} ${to.y}`;
  const down = to.y > from.y;
  const y1 = down ? from.y + NODE_HEIGHT : from.y;
  const y2 = down ? to.y : to.y + NODE_HEIGHT;
  const bend = (y2 - y1) / 2;
  const skipped = Math.abs(to.y - from.y) > NODE_HEIGHT * 2.5;
  const swing = skipped ? SWING + Math.max(from.width, to.width) / 2 : 0;
  return `M ${x1} ${y1} C ${x1 + swing} ${y1 + bend}, ${x2 + swing} ${y2 - bend}, ${x2} ${y2}`;
}

const loopedAreas = (map: ProjectMap) => {
  const inCycle = new Set(map.cycles.flat());
  return new Set(map.regions.filter((region) => region.files.some((file) => inCycle.has(file))).map((region) => region.name));
};

export function MapGraph({ map }: { map: ProjectMap }) {
  const picked = useStore(atlas, (s) => s.picked);
  const laid = useMemo(() => layered(map.regions.map((region) => region.name), map.links), [map]);
  const looped = useMemo(() => loopedAreas(map), [map]);
  const placed = new Map(laid.nodes.map((node) => [node.name, node]));
  const files = new Map(map.regions.map((region) => [region.name, region.files.length]));
  const heaviest = Math.max(1, ...map.links.map((link) => link.weight));
  const touches = (link: AreaLink) => link.from === picked || link.to === picked;
  const near = new Set(map.links.filter(touches).flatMap((link) => [link.from, link.to]));
  const drawn = map.links.filter((link) => placed.has(link.from) && placed.has(link.to) && link.from !== link.to);
  const ordered = [...drawn.filter((link) => !touches(link)), ...drawn.filter(touches)];

  return (
    <div className="map-graph" data-picking={String(Boolean(picked))}>
      <svg width={laid.width} height={laid.height} viewBox={`0 0 ${laid.width} ${laid.height}`} role="group" aria-label={t.graph}>
        <defs>
          <marker id="map-arrow" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
            <path d="M0,0 L8,4 L0,8 z" className="map-arrow" />
          </marker>
          <marker id="map-arrow-lit" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
            <path d="M0,0 L8,4 L0,8 z" className="map-arrow lit" />
          </marker>
        </defs>
        {ordered.map((link) => {
          const lit = touches(link);
          return (
            <path
              key={`${link.from}→${link.to}`}
              className="map-edge"
              data-lit={String(lit)}
              d={curve(placed.get(link.from)!, placed.get(link.to)!)}
              strokeWidth={1 + (2 * link.weight) / heaviest}
              markerEnd={`url(#${lit ? "map-arrow-lit" : "map-arrow"})`}
            >
              <title>{t.edge(link.from, link.to, link.weight)}</title>
            </path>
          );
        })}
        {laid.nodes.map((node) => (
          <Node key={node.name} node={node} files={files.get(node.name) ?? 0} looped={looped.has(node.name)} picked={node.name === picked} near={near.has(node.name)} />
        ))}
      </svg>
    </div>
  );
}

function Node({ node, files, looped, picked, near }: { node: Placed; files: number; looped: boolean; picked: boolean; near: boolean }) {
  const label = [node.name, t.files(files), looped ? t.inCycle : ""].filter(Boolean).join(" · ");
  const press = (event: KeyboardEvent) => {
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    pickArea(node.name);
  };
  return (
    <g
      className="map-node"
      data-picked={String(picked)}
      data-near={String(near)}
      transform={`translate(${node.x}, ${node.y})`}
      role="button"
      tabIndex={0}
      aria-pressed={picked}
      aria-label={label}
      onClick={() => pickArea(node.name)}
      onKeyDown={press}
    >
      <title>{label}</title>
      <rect width={node.width} height={NODE_HEIGHT} rx={8} />
      <text x={12} y={19} className="map-node-name">
        {node.label}
      </text>
      <text x={12} y={34} className="map-node-count">
        {t.files(files)}
      </text>
      {looped && (
        <g className="map-node-cycle" transform={`translate(${node.width - 22}, 24)`}>
          <Icon svg={ICONS.cycle} />
        </g>
      )}
    </g>
  );
}
