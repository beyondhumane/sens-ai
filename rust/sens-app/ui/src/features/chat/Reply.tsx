import { Fragment, memo, useEffect, useState, type ReactNode } from "react";
import { compact, seconds, whole } from "../../shared/format.js";
import { Ask } from "./Ask";
import { Run } from "./Run";
import { Said } from "./Said";
import { t } from "./thread.copy";
import type { Compacted as CompactedPart, Foot as FootPart, Reply as ReplyTurn } from "./turns";
import { FOLD_AT, grouped } from "./work";

const JOIN = " · ";
const MARK = "\u0000";

export const Reply = memo(function Reply({ turn }: { turn: ReplyTurn }) {
  return (
    <div className="turn reply">
      {turn.who && <div className="who">{turn.who}</div>}
      <Flow turn={turn} />
      {turn.working && <Live said={turn.working} began={turn.began} />}
    </div>
  );
});

export function Flow({ turn }: { turn: ReplyTurn }) {
  return (
    <div className="flow">
      {grouped(turn.parts).map((piece) => {
        switch (piece.kind) {
          case "run":
            return <Run key={piece.key} parts={piece.parts} folded={turn.closed && piece.parts.length >= FOLD_AT} />;
          case "said":
            return <Said key={piece.key} part={piece} />;
          case "ask":
            return <Ask key={piece.key} part={piece} reply={turn.key} />;
          case "fault":
            return (
              <p key={piece.key} className="reply-fault">
                {piece.text}
              </p>
            );
          case "foot":
            return <Foot key={piece.key} part={piece} />;
          case "compacted":
            return <Note key={piece.key} part={piece} />;
        }
      })}
    </div>
  );
}

function Foot({ part }: { part: FootPart }) {
  const [before, after] = t.tokens(MARK).split(MARK);
  const pieces: ReactNode[] = [];
  if (part.millis) pieces.push(seconds(part.millis));
  if (part.tokens)
    pieces.push(
      <>
        {before}
        <b className="foot-count">{compact(part.tokens)}</b>
        {after}
      </>,
    );
  if (part.stopped) pieces.push(t.stopped);
  return (
    <div className="reply-foot">
      {pieces.map((piece, at) => (
        <Fragment key={at}>
          {at > 0 && JOIN}
          <span>{piece}</span>
        </Fragment>
      ))}
    </div>
  );
}

function Note({ part }: { part: CompactedPart }) {
  return <p className="reply-note">{[part.auto ? t.compactedByClaude : t.compacted, part.before ? t.before(compact(part.before)) : ""].filter(Boolean).join(JOIN)}</p>;
}

export function Live({ said, began }: { said: string; began: number }) {
  const [now, setNow] = useState(() => performance.now());
  useEffect(() => {
    const clock = setInterval(() => setNow(performance.now()), 1000);
    return () => clearInterval(clock);
  }, []);
  return (
    <div className="live">
      <span className="pulse" />
      <span className="live-said">{said}</span>
      <span className="live-clock">{seconds(whole(now - began))}</span>
    </div>
  );
}
