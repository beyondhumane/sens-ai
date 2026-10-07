export interface View {
  x: number;
  y: number;
  k: number;
}

export const LEAST_ZOOM = 0.25;
export const MOST_ZOOM = 2.5;
const FIT_MOST = 1.2;
const ROOM = 40;

export const clampZoom = (k: number) => Math.min(MOST_ZOOM, Math.max(LEAST_ZOOM, k));

export function fitted(area: { left: number; top: number; width: number; height: number }, stage: { width: number; height: number }): View {
  const k = clampZoom(Math.min(FIT_MOST, (stage.width - ROOM * 2) / (area.width || 1), (stage.height - ROOM * 2) / (area.height || 1)));
  return { k, x: (stage.width - area.width * k) / 2 - area.left * k, y: (stage.height - area.height * k) / 2 - area.top * k };
}

export function zoomedAt(view: View, at: { x: number; y: number }, factor: number): View {
  const k = clampZoom(view.k * factor);
  const ratio = k / view.k;
  return { k, x: at.x - (at.x - view.x) * ratio, y: at.y - (at.y - view.y) * ratio };
}

export const centeredOn = (view: View, point: { x: number; y: number }, stage: { width: number; height: number }): View => ({
  k: view.k,
  x: stage.width / 2 - point.x * view.k,
  y: stage.height / 2 - point.y * view.k,
});

export function inSight(view: View, box: { x: number; y: number; width: number; height: number }, stage: { width: number; height: number }) {
  const left = box.x * view.k + view.x;
  const top = box.y * view.k + view.y;
  return left >= 0 && top >= 0 && left + box.width * view.k <= stage.width && top + box.height * view.k <= stage.height;
}
