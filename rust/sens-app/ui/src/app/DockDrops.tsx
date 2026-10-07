import { createPortal } from "react-dom";
import { useStore } from "zustand";
import { Icon } from "../shared/Icon";
import { ICONS } from "../shared/icons.js";
import { t } from "./copy";
import { DOCKS, type Dock } from "./shell";
import { pairHalf, toolDrag } from "./toolDrag";
import { DOCK_ICONS, TOOL_ICONS } from "./ToolsMenu";

export function DockDrops() {
  const lifted = useStore(toolDrag, (s) => (s.on === "window" ? s.tool : null));
  const aimed = useStore(toolDrag, (s) => s.target?.dock);
  const pair = useStore(toolDrag, (s) => Boolean(s.target?.pair));
  if (!lifted) return null;
  return (
    <div className="snap dock-drops" data-kind={aimed ? "open" : "none"} aria-hidden="true">
      {pair && aimed !== "shelf" && aimed ? (
        <PairDrop dock={aimed} />
      ) : (
        DOCKS.map((dock) => (
          <div key={dock} className={`${aimed === dock ? "snap-slot" : "snap-stay"} dock-slot`} data-dock={dock} data-shown="true">
            <span className="snap-icon">
              <Icon svg={DOCK_ICONS[dock]} />
            </span>
            <span className="snap-title">{t.dock[dock]}</span>
            {aimed === dock && <span className="snap-hint">{t.dropMove}</span>}
          </div>
        ))
      )}
    </div>
  );
}

function PairDrop({ dock }: { dock: Dock }) {
  const aside = document.getElementById(`dock-${dock}`)?.getBoundingClientRect();
  const body = document.getElementById("body")?.getBoundingClientRect();
  if (!aside || !body) return null;
  const half = pairHalf(dock, aside);
  const style = { left: half.left - body.left + 6, top: half.top - body.top + 6, width: half.width - 12, height: half.height - 12 };
  return (
    <div className="snap-slot dock-slot pair-slot" data-shown="true" style={style}>
      <span className="snap-icon">
        <Icon svg={dock === "bottom" ? ICONS.splitView : ICONS.splitRows} />
      </span>
      <span className="snap-hint">{t.dropPair}</span>
    </div>
  );
}

export function ToolGhost() {
  const tool = useStore(toolDrag, (s) => s.tool);
  const over = useStore(toolDrag, (s) => Boolean(s.target));
  const x = useStore(toolDrag, (s) => s.x);
  const y = useStore(toolDrag, (s) => s.y);
  if (!tool) return null;
  return createPortal(
    <div className="snap-ghost" data-over={over ? "true" : undefined} style={{ transform: `translate(${x + 14}px, ${y + 12}px)` }} aria-hidden="true">
      <Icon svg={TOOL_ICONS[tool]} />
      <span className="snap-ghost-title">{t.tool[tool]}</span>
    </div>,
    document.body,
  );
}
