import { useState, type ReactNode } from "react";
import { shared } from "./copy";
import { Markdown } from "./markdown/Markdown";

const LONG_TEXT = 900;
const LONG_LINES = 12;

export const lengthy = (text: string) => text.length > LONG_TEXT || text.split("\n").length > LONG_LINES;

export function Folded({ long, children }: { long: boolean; children: ReactNode }) {
  const [open, setOpen] = useState(false);
  const folded = long && !open;
  return (
    <div className="folded" data-folded={long ? String(folded) : undefined}>
      <div className="inside">{children}</div>
      {folded && (
        <button className="unfold" type="button" onClick={() => setOpen(true)}>
          {shared.showAll}
        </button>
      )}
    </div>
  );
}

export const FoldedText = ({ text }: { text: string }) => (
  <Folded long={lengthy(text)}>
    <Markdown text={text} />
  </Folded>
);
