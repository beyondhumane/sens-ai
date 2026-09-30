const GAP = 12;
const STEP = 18;
const SWAY_X = 18;
const SWAY_Y = 8;
const DRIFT_X = 0.012;
const DRIFT_Y = 0.005;
const REACH = 170;
const TENSION = 0.012;
const FRICTION = 0.9;
const MAX_PUSH = 80;
const LINE = 1.5;

interface Point {
  x: number;
  y: number;
  swayX: number;
  swayY: number;
  pushX: number;
  pushY: number;
  speedX: number;
  speedY: number;
}

interface Pointer {
  x: number;
  y: number;
  smoothX: number;
  smoothY: number;
  lastX: number;
  lastY: number;
  pace: number;
  angle: number;
}

function noiseField(): (x: number, y: number) => number {
  const order = Array.from({ length: 256 }, (_, at) => at);
  for (let at = order.length - 1; at > 0; at -= 1) {
    const other = Math.floor(Math.random() * (at + 1));
    [order[at], order[other]] = [order[other], order[at]];
  }
  const table = [...order, ...order];
  const ease = (t: number) => t * t * t * (t * (t * 6 - 15) + 10);
  const mix = (from: number, to: number, t: number) => from + t * (to - from);
  const slope = (hash: number, x: number, y: number) => (hash & 1 ? -x : x) + (hash & 2 ? -y : y);
  return (x, y) => {
    const cellX = Math.floor(x);
    const cellY = Math.floor(y);
    const fx = x - cellX;
    const fy = y - cellY;
    const left = table[cellX & 255] + (cellY & 255);
    const right = table[(cellX + 1) & 255] + (cellY & 255);
    const top = mix(slope(table[left], fx, fy), slope(table[right], fx - 1, fy), ease(fx));
    const bottom = mix(slope(table[left + 1], fx, fy - 1), slope(table[right + 1], fx - 1, fy - 1), ease(fx));
    return mix(top, bottom, ease(fy));
  };
}

function gridOf(width: number, height: number): Point[][] {
  const lines: Point[][] = [];
  for (let x = -SWAY_X * 2; x <= width + SWAY_X * 2; x += GAP) {
    const line: Point[] = [];
    for (let y = -STEP * 2; y <= height + STEP * 2; y += STEP) {
      line.push({ x, y, swayX: 0, swayY: 0, pushX: 0, pushY: 0, speedX: 0, speedY: 0 });
    }
    lines.push(line);
  }
  return lines;
}

const clampPush = (value: number) => Math.max(-MAX_PUSH, Math.min(MAX_PUSH, value));

function sway(point: Point, noise: (x: number, y: number) => number, time: number): void {
  const turn = noise((point.x + time * DRIFT_X) * 0.002, (point.y + time * DRIFT_Y) * 0.0015) * 12;
  point.swayX = Math.cos(turn) * SWAY_X;
  point.swayY = Math.sin(turn) * SWAY_Y;
}

function push(point: Point, pointer: Pointer): void {
  const reach = Math.max(REACH, pointer.pace);
  const offsetX = point.x - pointer.smoothX;
  const offsetY = point.y - pointer.smoothY;
  const near = Math.abs(offsetX) < reach && Math.abs(offsetY) < reach;
  if (!near && Math.abs(point.pushX) + Math.abs(point.pushY) + Math.abs(point.speedX) + Math.abs(point.speedY) < 0.01) return;
  const distance = near ? Math.hypot(offsetX, offsetY) : reach;
  if (distance < reach) {
    const strength = (1 - distance / reach) * reach * pointer.pace * 0.00065;
    point.speedX += Math.cos(pointer.angle) * strength;
    point.speedY += Math.sin(pointer.angle) * strength;
  }
  point.speedX = (point.speedX - point.pushX * TENSION) * FRICTION;
  point.speedY = (point.speedY - point.pushY * TENSION) * FRICTION;
  point.pushX = clampPush(point.pushX + point.speedX * 2);
  point.pushY = clampPush(point.pushY + point.speedY * 2);
}

function follow(pointer: Pointer): void {
  pointer.smoothX += (pointer.x - pointer.smoothX) * 0.1;
  pointer.smoothY += (pointer.y - pointer.smoothY) * 0.1;
  const moved = Math.hypot(pointer.x - pointer.lastX, pointer.y - pointer.lastY);
  pointer.pace = Math.min(100, pointer.pace + (moved - pointer.pace) * 0.1);
  pointer.angle = Math.atan2(pointer.y - pointer.lastY, pointer.x - pointer.lastX);
  pointer.lastX = pointer.x;
  pointer.lastY = pointer.y;
}

export function waves(wordmark: HTMLElement): () => void {
  const svg = wordmark.querySelector("svg");
  const outline = svg?.querySelector("path")?.getAttribute("d");
  const context = document.createElement("canvas").getContext("2d");
  if (!svg || !outline || !context) return () => {};
  const canvas = context.canvas;
  canvas.setAttribute("aria-hidden", "true");
  wordmark.append(canvas);

  const letters = new Path2D(outline);
  const view = svg.viewBox.baseVal;
  const noise = noiseField();
  const paperOf = () => getComputedStyle(canvas).color;
  let paper = paperOf();
  const themed = new MutationObserver(() => {
    paper = paperOf();
  });
  const pointer: Pointer = { x: -1e4, y: -1e4, smoothX: -1e4, smoothY: -1e4, lastX: -1e4, lastY: -1e4, pace: 0, angle: 0 };
  let lines: Point[][] = [];
  let width = 0;
  let height = 0;
  let ratio = 1;
  let frame = 0;

  const size = () => {
    const box = svg.getBoundingClientRect();
    width = box.width;
    height = box.height;
    ratio = window.devicePixelRatio || 1;
    canvas.width = Math.round(width * ratio);
    canvas.height = Math.round(height * ratio);
    lines = gridOf(width, height);
  };

  const draw = (time: number) => {
    follow(pointer);
    const scale = width / view.width;
    context.setTransform(ratio, 0, 0, ratio, 0, 0);
    context.clearRect(0, 0, width, height);
    context.save();
    context.setTransform(ratio * scale, 0, 0, ratio * scale, -view.x * scale * ratio, -view.y * scale * ratio);
    context.clip(letters);
    context.setTransform(ratio, 0, 0, ratio, 0, 0);
    context.beginPath();
    for (const line of lines) {
      line.forEach((point, at) => {
        sway(point, noise, time);
        push(point, pointer);
        const x = point.x + point.swayX + point.pushX;
        const y = point.y + point.swayY + point.pushY;
        if (at === 0) context.moveTo(x, y);
        else context.lineTo(x, y);
      });
    }
    context.strokeStyle = paper;
    context.lineWidth = LINE;
    context.stroke();
    context.restore();
    frame = requestAnimationFrame(draw);
  };

  const track = (event: PointerEvent) => {
    const box = canvas.getBoundingClientRect();
    pointer.x = event.clientX - box.left;
    pointer.y = event.clientY - box.top;
  };

  const start = () => {
    if (!frame) frame = requestAnimationFrame(draw);
  };
  const pause = () => {
    cancelAnimationFrame(frame);
    frame = 0;
  };

  const seen = new IntersectionObserver(([entry]) => (entry?.isIntersecting ? start() : pause()));
  const resized = new ResizeObserver(size);
  size();
  resized.observe(svg);
  seen.observe(wordmark);
  themed.observe(document.documentElement, { attributeFilter: ["data-theme"] });
  window.addEventListener("pointermove", track, { passive: true });

  return () => {
    pause();
    seen.disconnect();
    resized.disconnect();
    themed.disconnect();
    window.removeEventListener("pointermove", track);
    canvas.remove();
  };
}
