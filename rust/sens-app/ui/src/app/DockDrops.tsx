import { createPortal } from "react-dom";
import { useStore } from "zustand";
import { Icon } from "../shared/Icon";
import { t } from "./copy";
import { DOCKS } from "./shell";
import { toolDrag } from "./toolDrag";
import { DOCK_ICONS, TOOL_ICONS } from "./ToolsMenu";

export function DockDrops() {
  const lifted = useStore(toolDrag, (s) => (s.on === "window" ? s.tool : null));
  const aimed = useStore(toolDrag, (s) => s.target?.dock);
  if (!lifted) return null;
  return (
    <div className="snap dock-drops" data-kind={aimed ? "open" : "none"} aria-hidden="true">
      {DOCKS.map((dock) => (
        <div key={dock} className={`${aimed === dock ? "snap-slot" : "snap-stay"} dock-slot`} data-dock={dock} data-shown="true">
          <span className="snap-icon">
            <Icon svg={DOCK_ICONS[dock]} />
          </span>
          <span className="snap-title">{t.dock[dock]}</span>
          {aimed === dock && <span className="snap-hint">{t.dropMove}</span>}
        </div>
      ))}
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
