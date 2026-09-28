export function Seam() {
  return (
    <div className="seam" aria-hidden="true">
      <svg viewBox="0 0 600 12" preserveAspectRatio="none">
        <path className="seam-base" pathLength={1} vectorEffect="non-scaling-stroke" />
        <path className="seam-run" pathLength={1} vectorEffect="non-scaling-stroke" />
      </svg>
    </div>
  );
}
