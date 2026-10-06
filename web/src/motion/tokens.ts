import brand from "../brand/brand.json";

const ms = (value: string): number => Number.parseFloat(value);

export const durations = {
  fast: ms(brand.motion.hover),
  medium: 220,
  scene: 600,
  entrance: 1000,
  confirm: ms(brand.motion.confirm),
  scan: ms(brand.motion.scan),
} as const;

export const curves = {
  control: brand.motion.ease,
  enter: "cubic-bezier(.16,1,.3,1)",
  exit: "cubic-bezier(.7,0,.84,0)",
  move: "cubic-bezier(.65,0,.35,1)",
  linked: "linear",
} as const;

export const distances = {
  shift: "0.3em",
  scale: 1.02,
} as const;

export const stagger = {
  lines: 70,
  steps: 80,
  links: 30,
  rays: 30,
} as const;

export const overlap = 0.4;

export type Curve = keyof typeof curves;

export const bezierOf = (curve: Curve): string => {
  const values = curves[curve].match(/-?\d*\.?\d+/g);
  return values ? values.join(",") : "0,0,1,1";
};

export function motionCss(): string {
  const lines = [
    ...Object.entries(durations).map(([name, value]) => `  --dur-${name}: ${value}ms;`),
    ...Object.entries(curves).map(([name, value]) => `  --ease-${name}: ${value};`),
    `  --shift: ${distances.shift};`,
    `  --scale-in: ${distances.scale};`,
    ...Object.entries(stagger).map(([name, value]) => `  --stagger-${name}: ${value}ms;`),
  ];
  return [":root {", ...lines, "}"].join("\n");
}
