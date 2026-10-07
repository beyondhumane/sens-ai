export function Tally({ plus, minus }: { plus: number; minus?: number }) {
  return (
    <>
      <span className="plus">{`+${plus}`}</span>
      {minus !== undefined && <span className="minus">{`−${minus}`}</span>}
    </>
  );
}
