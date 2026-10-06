import { ArrowDownToLine, ArrowRight, ArrowUpRight, Check, Languages, Menu, Moon, RotateCcw, Sun, X } from "lucide";

type Node = [string, Record<string, string | number>][];

const draw = (node: Node): string =>
  node
    .map(([tag, attributes]) => `<${tag} ${Object.entries(attributes).map(([key, value]) => `${key}="${value}"`).join(" ")}/>`)
    .join("");

const CENTER = 12;

const clockwiseFromTop = (path: string): number => {
  const [x = CENTER, y = CENTER] = (path.match(/-?\d*\.?\d+/g) ?? []).map(Number);
  const degrees = (Math.atan2(y - CENTER, x - CENTER) * 180) / Math.PI;
  return (degrees + 90 + 360) % 360;
};

function sunAndMoon(): string {
  const [core, ...rays] = Sun as Node;
  const turn = 360 / rays.length;
  return draw([
    [core[0], { ...core[1], class: "sun-core", pathLength: 1 }],
    ...rays.map(([tag, attributes]): Node[number] => [
      tag,
      { ...attributes, class: "sun-ray", style: `--turn: ${Math.round(clockwiseFromTop(String(attributes.d)) / turn)}` },
    ]),
    ...(Moon as Node).map(([tag, attributes]): Node[number] => [tag, { ...attributes, class: "moon", pathLength: 1 }]),
  ]);
}

export const siteIcons = {
  download: draw(ArrowDownToLine as Node),
  arrow: draw(ArrowRight as Node),
  external: draw(ArrowUpRight as Node),
  menu: draw(Menu as Node),
  close: draw(X as Node),
  replay: draw(RotateCcw as Node),
  languages: draw(Languages as Node),
  check: draw(Check as Node),
  theme: sunAndMoon(),
};

const brain =
  '<path d="M12 18V5"/><path d="M15 13a4.5 4.5 0 0 1-3-4 4.5 4.5 0 0 1-3 4"/><path d="M17.598 6.5a3 3 0 1 0-5.598-1.5 3 3 0 1 0-5.598 1.5"/><path d="M19.967 17.484A4 4 0 0 1 18 18a4 4 0 0 1-4-4"/><path d="M6.003 5.125A3 3 0 0 0 6.401 6.5"/><path d="M6 18a4 4 0 0 1-1.967-.516A4 4 0 0 0 10 14"/><path d="M19.938 10.5a4 4 0 0 1 .585.396 4 4 0 0 1-.585 6.588"/><path d="M4.062 10.5a4 4 0 0 0-.585.396 4 4 0 0 0 .585 6.588"/><path d="M4.062 10.5a4 4 0 0 1 2.526-5.375"/><path d="M19.938 10.5a4 4 0 0 0-2.526-5.375"/>';

const folder =
  '<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>';

const shield =
  '<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/>';

export const appIcons = {
  thought: brain,
  read: '<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><path d="M10 9H8"/><path d="M16 13H8"/><path d="M16 17H8"/>',
  search: '<path d="m21 21-4.34-4.34"/><circle cx="11" cy="11" r="8"/>',
  run: '<path d="M12 19h8"/><path d="m4 17 6-6-6-6"/>',
  edit: '<path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/><path d="m15 5 4 4"/>',
  chevronRight: '<path d="m9 18 6-6-6-6"/>',
  caret: '<path d="M6 9l6 6 6-6"/>',
  copy: '<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>',
  railToggle: '<rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/><path d="m14 9 3 3-3 3"/>',
  focusMode: '<circle cx="12" cy="12" r="3"/><path d="M3 7V5a2 2 0 0 1 2-2h2"/><path d="M17 3h2a2 2 0 0 1 2 2v2"/><path d="M21 17v2a2 2 0 0 1-2 2h-2"/><path d="M7 21H5a2 2 0 0 1-2-2v-2"/>',
  tools: '<circle cx="12" cy="12" r="1"/><circle cx="12" cy="5" r="1"/><circle cx="12" cy="19" r="1"/>',
  minimize: '<path d="M5 12h14"/>',
  maximize: '<rect x="5" y="5" width="14" height="14" rx="2"/>',
  close: '<path d="M18 6 6 18M6 6l12 12"/>',
  folder,
  branch: '<path d="M6 3v12"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/>',
  attach: '<path d="M20 11.5 12.2 19.3a4.5 4.5 0 0 1-6.4-6.4l7.8-7.8a3 3 0 0 1 4.3 4.3l-7.8 7.8a1.5 1.5 0 0 1-2.2-2.1l7.2-7.2"/>',
  dictate: '<rect x="9" y="3" width="6" height="11" rx="3"/><path d="M5 11a7 7 0 0 0 14 0M12 18v3"/>',
  send: '<path d="M12 19V5M5 12l7-7 7 7"/>',
  reload: '<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/>',
  shut: '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
  shieldAlert: `${shield}<path d="M12 8v4"/><path d="M12 16h.01"/>`,
  shieldCheck: `${shield}<path d="m9 12 2 2 4-4"/>`,
};

export type AppIcon = keyof typeof appIcons;
export type SiteIcon = keyof typeof siteIcons;
