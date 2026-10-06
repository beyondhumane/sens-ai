export interface Scale {
  min: number;
  max: number;
  ticks: number[];
}

export const median = (values: number[]): number => {
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
};

export function niceScale(max: number, count = 5): Scale {
  const rough = max / count;
  const power = 10 ** Math.floor(Math.log10(rough));
  const step = [1, 2, 2.5, 5, 10].map((factor) => factor * power).find((candidate) => candidate >= rough) ?? rough;
  const top = Math.ceil(max / step) * step;
  return { min: 0, max: top, ticks: Array.from({ length: Math.round(top / step) + 1 }, (_, at) => at * step) };
}

export function spread(wanted: number[], gap: number): number[] {
  const order = wanted.map((value, at) => ({ value, at })).sort((a, b) => a.value - b.value);
  for (let at = 1; at < order.length; at += 1) order[at].value = Math.max(order[at].value, order[at - 1].value + gap);
  const placed: number[] = [];
  for (const { value, at } of order) placed[at] = value;
  return placed;
}

export const pathOf = (points: [number, number][]): string =>
  points.map(([x, y], at) => `${at === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`).join(" ");

export const millions = (tokens: number): string => `${(tokens / 1_000_000).toFixed(1)}M`;
