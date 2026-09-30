export const WINDOW = { width: 960, height: 660 } as const;

export const NAV = 72;
const GUTTER = 24;
const CONTENT = 1440;
const MARK = 4;
const MARK_GAP = 14;
const FIELD_BELOW = 40;
const PUSH = 0.85;

export interface Viewport {
  width: number;
  height: number;
}

export interface Placement {
  x: number;
  y: number;
  s: number;
}

export interface Frame {
  hero: Placement;
  stage: Placement;
  field: { x: number; width: number; height: number };
  rail: { x: number; y: number; width: number; height: number };
}

const clamp = (min: number, value: number, max: number): number => Math.min(max, Math.max(min, value));

export function frameOf({ width, height }: Viewport): Frame {
  const margin = clamp(16, width * 0.055, 96);
  const content = Math.min(width - 2 * margin, CONTENT);
  const left = (width - content) / 2;
  const column = (content - 11 * GUTTER) / 12;
  const at = (columns: number) => left + columns * (column + GUTTER);

  const stageX = at(3);
  const stageS = Math.min(1, (left + content - stageX) / WINDOW.width, (height - NAV - 48) / WINDOW.height);
  const stageY = NAV + (height - NAV - WINDOW.height * stageS) / 2;

  const heroY = Math.max(NAV + 32, height * 0.12);
  const heroX = at(5) + GUTTER;
  const heroS = Math.min((width - heroX - margin / 2) / WINDOW.width, stageS * PUSH);
  const fieldX = at(6);

  return {
    hero: { x: heroX, y: heroY, s: heroS },
    stage: { x: stageX, y: stageY, s: stageS },
    field: { x: fieldX, width: width - fieldX, height: heroY + WINDOW.height * heroS + FIELD_BELOW },
    rail: { x: stageX - MARK_GAP, y: stageY, width: MARK, height: WINDOW.height * stageS },
  };
}
