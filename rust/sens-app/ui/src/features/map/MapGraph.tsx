import { useEffect, useLayoutEffect, useMemo, useRef, useState, type KeyboardEvent, type PointerEvent as ReactPointerEvent } from "react";
import { useStore } from "zustand";
import type { AreaLink, ProjectMap } from "../../ipc/types";
import { Icon } from "../../shared/Icon";
import { ICONS } from "../../shared/icons.js";
import { t } from "./copy";
import { boundsOf, layered, linkPath, NODE_HEIGHT, type Laid, type Placed, type Point } from "./layout";
import { atlas, pickArea } from "./store";
import { centeredOn, fitted, inSight, zoomedAt, type View } from "./view";

const STILL = 4;
const STEP = 1.25;
const NUDGE = 48;
const GLIDE = 260;
const WHEEL = 0.0015;
const DOTS = 24;

const keyOf = (from: string, to: string) => `${from}→${to}`;

function cycleLinks(map: ProjectMap) {
  const areaOf = new Map(map.regions.flatMap((region) => region.files.map((file) => [file, region.name] as const)));
  const found = new Set<string>();
  for (const cycle of map.cycles) {
    cycle.forEach((file, at) => {
      const from = areaOf.get(file);
      const to = areaOf.get(cycle[(at + 1) % cycle.length]);
      if (from && to && from !== to) found.add(keyOf(from, to));
    });
  }
  return found;
}

const loopedAreas = (map: ProjectMap) => {
  const inCycle = new Set(map.cycles.flat());
  return new Set(map.regions.filter((region) => region.files.some((file) => inCycle.has(file))).map((region) => region.name));
};

const sizeOf = (element: HTMLElement | null) => ({ width: element?.clientWidth ?? 0, height: element?.clientHeight ?? 0 });

function useViewport(nodes: Placed[], laid: Laid) {
  const stage = useRef<HTMLDivElement>(null);
  const [view, setView] = useState<View>({ x: 0, y: 0, k: 1 });
  const [gliding, setGliding] = useState(false);
  const touched = useRef(false);
  const timer = useRef(0);
  const area = boundsOf(nodes);
  const fit = () => fitted(area, sizeOf(stage.current));

  const glide = (next: View) => {
    window.clearTimeout(timer.current);
    setGliding(true);
    setView(next);
    timer.current = window.setTimeout(() => setGliding(false), GLIDE);
  };

  const steer = (next: (now: View) => View) => {
    touched.current = true;
    setGliding(false);
    setView(next);
  };

  const refit = () => {
    touched.current = false;
    glide(fit());
  };

  useLayoutEffect(() => {
    const element = stage.current;
    if (!element) return;
    const settle = () => {
      if (!touched.current) setView(fitted(boundsOf(laid.nodes), sizeOf(element)));
    };
    settle();
    if (typeof ResizeObserver !== "function") return;
    const watch = new ResizeObserver(settle);
    watch.observe(element);
    return () => watch.disconnect();
  }, [laid]);

  useEffect(() => {
    const element = stage.current;
    if (!element) return;
    const wheel = (event: WheelEvent) => {
      event.preventDefault();
      const box = element.getBoundingClientRect();
      if (event.shiftKey) return steer((now) => ({ ...now, x: now.x - event.deltaY, y: now.y - event.deltaX }));
      steer((now) => zoomedAt(now, { x: event.clientX - box.left, y: event.clientY - box.top }, Math.exp(-event.deltaY * WHEEL)));
    };
    element.addEventListener("wheel", wheel, { passive: false });
    return () => element.removeEventListener("wheel", wheel);
  }, []);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  const zoomBy = (factor: number) => {
    touched.current = true;
    const { width, height } = sizeOf(stage.current);
    glide(zoomedAt(view, { x: width / 2, y: height / 2 }, factor));
  };

  const reveal = (node: Placed) => {
    const size = sizeOf(stage.current);
    if (!size.width || inSight(view, { ...node, height: NODE_HEIGHT }, size)) return;
    touched.current = true;
    glide(centeredOn(view, { x: node.x + node.width / 2, y: node.y + NODE_HEIGHT / 2 }, size));
  };

  const hold = () => void (touched.current = true);

  return { stage, view, gliding, steer, refit, zoomBy, reveal, hold };
}

export function MapGraph({ map }: { map: ProjectMap }) {
  const picked = useStore(atlas, (s) => s.picked);
  const laid = useMemo(() => layered(map.regions.map((region) => region.name), map.links), [map]);
  const looped = useMemo(() => loopedAreas(map), [map]);
  const cyclic = useMemo(() => cycleLinks(map), [map]);
  const [moved, setMoved] = useState<Record<string, Point>>({});
  const [hovered, setHovered] = useState("");
  const nodes = useMemo(() => laid.nodes.map((node) => ({ ...node, ...moved[node.name] })), [laid, moved]);
  const { stage, view, gliding, steer, refit, zoomBy, reveal, hold } = useViewport(nodes, laid);
  const dragged = useRef(false);
  const [grabbing, setGrabbing] = useState(false);

  const placed = new Map(nodes.map((node) => [node.name, node]));
  const files = new Map(map.regions.map((region) => [region.name, region.files.length]));
  const most = Math.max(1, ...files.values());
  const heaviest = Math.max(1, ...map.links.map((link) => link.weight));
  const focus = hovered || picked;
  const touches = (link: AreaLink) => link.from === focus || link.to === focus;
  const near = new Set(map.links.filter(touches).flatMap((link) => [link.from, link.to]));
  const drawn = map.links.filter((link) => placed.has(link.from) && placed.has(link.to) && link.from !== link.to);
  const ordered = [...drawn.filter((link) => !touches(link)), ...drawn.filter(touches)];

  useEffect(() => {
    const node = placed.get(picked);
    if (node) reveal(node);
  }, [picked]);

  function grab(event: ReactPointerEvent<HTMLDivElement>) {
    if (event.button !== 0 || (event.target as Element).closest(".map-tools")) return;
    const name = (event.target as Element).closest<SVGGElement>("[data-node]")?.dataset.node;
    const from = { x: event.clientX, y: event.clientY };
    const base = name ? placed.get(name)! : view;
    const surface = event.currentTarget;
    let moving = false;
    dragged.current = false;
    const move = (now: PointerEvent) => {
      const dx = now.clientX - from.x;
      const dy = now.clientY - from.y;
      if (!moving && Math.hypot(dx, dy) < STILL) return;
      if (!moving) {
        surface.setPointerCapture(now.pointerId);
        setGrabbing(true);
        hold();
      }
      moving = true;
      dragged.current = true;
      if (name) setMoved((all) => ({ ...all, [name]: { x: base.x + dx / view.k, y: base.y + dy / view.k } }));
      else steer(() => ({ ...view, x: base.x + dx, y: base.y + dy }));
    };
    const done = () => {
      surface.removeEventListener("pointermove", move);
      surface.removeEventListener("pointerup", done);
      surface.removeEventListener("pointercancel", done);
      setGrabbing(false);
    };
    surface.addEventListener("pointermove", move);
    surface.addEventListener("pointerup", done);
    surface.addEventListener("pointercancel", done);
  }

  function keys(event: KeyboardEvent<HTMLDivElement>) {
    if (event.target !== event.currentTarget) return;
    const nudge = ({ ArrowLeft: [NUDGE, 0], ArrowRight: [-NUDGE, 0], ArrowUp: [0, NUDGE], ArrowDown: [0, -NUDGE] } as Record<string, number[]>)[event.key];
    const zoom = ({ "+": STEP, "=": STEP, "-": 1 / STEP } as Record<string, number>)[event.key];
    if (nudge) steer((now) => ({ ...now, x: now.x + nudge[0], y: now.y + nudge[1] }));
    else if (zoom) zoomBy(zoom);
    else if (event.key === "0") refit();
    else if (event.key === "Escape" && picked) pickArea(picked);
    else return;
    event.preventDefault();
  }

  const pick = (name: string) => {
    if (!dragged.current) pickArea(name);
  };

  const dots = DOTS * view.k;
  return (
    <div
      className="map-stage"
      ref={stage}
      tabIndex={0}
      aria-label={t.graphHelp}
      data-picking={String(Boolean(focus))}
      data-gliding={gliding ? "true" : undefined}
      data-grabbing={grabbing ? "true" : undefined}
      style={{ backgroundSize: `${dots}px ${dots}px`, backgroundPosition: `${view.x}px ${view.y}px` }}
      onPointerDown={grab}
      onDoubleClick={(event) => {
        if (!(event.target as Element).closest("[data-node], .map-tools")) refit();
      }}
      onKeyDown={keys}
    >
      <svg className="map-graph" role="group" aria-label={t.graph}>
        <defs>
          {["rest", "out", "in", "loop"].map((kind) => (
            <marker key={kind} id={`map-tip-${kind}`} viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" markerUnits="userSpaceOnUse" orient="auto-start-reverse">
              <path d="M1,1.5 L8.5,5 L1,8.5 z" className={`map-tip map-tip-${kind}`} />
            </marker>
          ))}
        </defs>
        <g className="map-world" style={{ transform: `translate(${view.x}px, ${view.y}px) scale(${view.k})` }}>
          {ordered.map((link) => {
            const way = link.from === focus ? "out" : link.to === focus ? "in" : "";
            const loop = cyclic.has(keyOf(link.from, link.to));
            const { d, middle } = linkPath(placed.get(link.from)!, placed.get(link.to)!);
            return (
              <g key={keyOf(link.from, link.to)} className="map-edge" data-way={way || undefined} data-loop={loop ? "true" : undefined} data-lit={String(Boolean(way))}>
                <title>{[t.edge(link.from, link.to, link.weight), loop ? t.inLoop : ""].filter(Boolean).join(" · ")}</title>
                <path d={d} strokeWidth={1.25 + (2.25 * link.weight) / heaviest} markerEnd={`url(#map-tip-${way || (loop ? "loop" : "rest")})`} />
                {way && (
                  <text className="map-weight" x={middle.x} y={middle.y} dy="0.35em">
                    {link.weight}
                  </text>
                )}
              </g>
            );
          })}
          {nodes.map((node) => (
            <Node
              key={node.name}
              node={node}
              files={files.get(node.name) ?? 0}
              most={most}
              looped={looped.has(node.name)}
              picked={node.name === picked}
              near={near.has(node.name) || node.name === focus}
              onPick={pick}
              onHover={setHovered}
            />
          ))}
        </g>
      </svg>
      <div className="map-tools" role="toolbar" aria-label={t.graph}>
        <button type="button" className="icon-btn quiet-btn" title={t.zoomOut} aria-label={t.zoomOut} onClick={() => zoomBy(1 / STEP)}>
          <Icon svg={ICONS.zoomOut} />
        </button>
        <span className="map-zoom" aria-live="polite">
          {Math.round(view.k * 100)}%
        </span>
        <button type="button" className="icon-btn quiet-btn" title={t.zoomIn} aria-label={t.zoomIn} onClick={() => zoomBy(STEP)}>
          <Icon svg={ICONS.zoomIn} />
        </button>
        <button type="button" className="icon-btn quiet-btn" title={t.fit} aria-label={t.fit} onClick={refit}>
          <Icon svg={ICONS.fit} />
        </button>
        {Object.keys(moved).length > 0 && (
          <button type="button" className="icon-btn quiet-btn" title={t.relayout} aria-label={t.relayout} onClick={() => setMoved({})}>
            <Icon svg={ICONS.undo} />
          </button>
        )}
      </div>
    </div>
  );
}

function Node({
  node,
  files,
  most,
  looped,
  picked,
  near,
  onPick,
  onHover,
}: {
  node: Placed;
  files: number;
  most: number;
  looped: boolean;
  picked: boolean;
  near: boolean;
  onPick: (name: string) => void;
  onHover: (name: string) => void;
}) {
  const label = [node.name, t.files(files), looped ? t.inCycle : ""].filter(Boolean).join(" · ");
  const press = (event: KeyboardEvent) => {
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    pickArea(node.name);
  };
  const bar = Math.max(4, ((node.width - 24) * files) / most);
  return (
    <g
      className="map-node"
      data-node={node.name}
      data-picked={String(picked)}
      data-near={String(near)}
      data-looped={looped ? "true" : undefined}
      transform={`translate(${node.x}, ${node.y})`}
      role="button"
      tabIndex={0}
      aria-pressed={picked}
      aria-label={label}
      onClick={() => onPick(node.name)}
      onKeyDown={press}
      onPointerEnter={() => onHover(node.name)}
      onPointerLeave={() => onHover("")}
      onFocus={() => onHover(node.name)}
      onBlur={() => onHover("")}
    >
      <title>{label}</title>
      <rect className="map-node-box" width={node.width} height={NODE_HEIGHT} rx={10} />
      <g className="map-node-icon" transform="translate(12, 16)">
        <Icon svg={ICONS.folderSmall} />
      </g>
      <text x={32} y={21} className="map-node-name">
        {node.label}
      </text>
      <text x={32} y={36} className="map-node-count">
        {t.files(files)}
      </text>
      <rect className="map-node-bar" x={12} y={NODE_HEIGHT - 5} width={bar} height={2} rx={1} />
      {looped && (
        <g className="map-node-cycle" transform={`translate(${node.width - 24}, 17)`}>
          <Icon svg={ICONS.cycle} />
        </g>
      )}
    </g>
  );
}
