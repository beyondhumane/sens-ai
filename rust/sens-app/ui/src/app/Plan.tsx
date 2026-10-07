import type { CSSProperties, KeyboardEvent } from "react";
import { useStore } from "zustand";
import { Icon } from "../shared/Icon";
import { t } from "./copy";
import { closeTab, DOCKS, shell, showTool, tabsIn, toggleDock, toggleRail, TOOLS, visibleIn, type Arrangement, type Dock, type Tool } from "./shell";
import { liftTool, moveByKey, toolDrag } from "./toolDrag";
import { DOCK_ICONS, TOOL_ICONS } from "./ToolsMenu";

const AREA: Record<Dock, string> = { start: "s", end: "e", bottom: "b" };

const SHUT = "minmax(0, 0fr)";

export function areasOf({ rail, wide }: Pick<Arrangement, "rail" | "wide">) {
  const row = (cells: string[]) => `"${(rail === "start" ? ["r", ...cells] : [...cells, "r"]).join(" ")}"`;
  return `${row(["s", "c", "e"])} ${row(wide ? ["b", "b", "b"] : ["s", "b", "e"])}`;
}

function gridOf(arrangement: Arrangement, every: boolean): CSSProperties {
  const track = (open: boolean, size: string, folded: string) => (open ? `minmax(0, ${size})` : every ? `minmax(0, ${folded})` : SHUT);
  const shown = (dock: Dock) => visibleIn(arrangement, dock) !== null;
  const rail = track(!arrangement.railClosed, "0.62fr", "0.22fr");
  const middle = [track(shown("start"), "1fr", "0.62fr"), "minmax(0, 1.5fr)", track(shown("end"), "1.15fr", "0.62fr")];
  return {
    gridTemplateAreas: areasOf(arrangement),
    gridTemplateColumns: (arrangement.rail === "start" ? [rail, ...middle] : [...middle, rail]).join(" "),
    gridTemplateRows: `minmax(0, 1fr) ${track(shown("bottom"), "0.62fr", "0.42fr")}`,
  };
}

export function MiniPlan({ arrangement }: { arrangement: Arrangement }) {
  return (
    <span className="plan" data-mini="true" data-calm={arrangement.calm ? "true" : undefined} style={gridOf(arrangement, false)} aria-hidden="true">
      {!arrangement.railClosed && <span className="plan-rail" />}
      <span className="plan-chat" />
      {DOCKS.filter((dock) => visibleIn(arrangement, dock)).map((dock) => (
        <span key={dock} className="plan-slot" style={{ gridArea: AREA[dock] }} />
      ))}
    </span>
  );
}

export function LivePlan() {
  const arrangement = useStore(shell, (s) => s);
  const lifted = useStore(toolDrag, (s) => (s.on === "plan" ? s.tool : null));
  const target = useStore(toolDrag, (s) => (s.on === "plan" ? s.target : null));
  const closed = TOOLS.filter((tool) => !arrangement.tabs.includes(tool));
  const chip = (tool: Tool, current: boolean) => (
    <Chip key={tool} tool={tool} current={current} lifted={lifted === tool} aimed={target?.before === tool} />
  );
  return (
    <div className="plan-live">
      <div className="plan" data-calm={arrangement.calm ? "true" : undefined} style={gridOf(arrangement, true)}>
        <button type="button" className="plan-rail" aria-pressed={!arrangement.railClosed} onClick={toggleRail}>
          <span>{t.sessions}</span>
        </button>
        <span className="plan-chat">{t.chat}</span>
        {DOCKS.map((dock) => {
          const tabs = tabsIn(arrangement, dock);
          const shown = visibleIn(arrangement, dock);
          return (
            <div key={dock} className="plan-slot" style={{ gridArea: AREA[dock] }} data-plan-dock={dock} data-open={shown ? "true" : undefined} data-aimed={target?.dock === dock ? "true" : undefined}>
              <button type="button" className="plan-head" aria-pressed={Boolean(shown)} disabled={!tabs.length} onClick={() => toggleDock(dock)}>
                <Icon svg={DOCK_ICONS[dock]} />
                <span>{t.dock[dock]}</span>
              </button>
              <span className="plan-chips">
                {tabs.map((tool) => chip(tool, tool === arrangement.shown[dock]))}
                {!tabs.length && <span className="plan-empty">{t.emptyDock}</span>}
              </span>
            </div>
          );
        })}
      </div>
      <div className="plan-shelf" data-plan-dock="shelf" data-aimed={target?.dock === "shelf" ? "true" : undefined}>
        <span className="label">{t.closedTools}</span>
        <span className="plan-chips">{closed.map((tool) => chip(tool, false))}</span>
      </div>
    </div>
  );
}

function Chip({ tool, current, lifted, aimed }: { tool: Tool; current: boolean; lifted: boolean; aimed: boolean }) {
  const keys = (event: KeyboardEvent) => {
    if (moveByKey(event, tool, null)) return;
    if (event.key !== "Delete" && event.key !== "Backspace") return;
    event.preventDefault();
    closeTab(tool);
  };
  return (
    <button
      type="button"
      className="plan-chip"
      data-plan-tool={tool}
      aria-current={current ? "true" : undefined}
      data-lifted={lifted ? "true" : undefined}
      data-aimed={aimed ? "true" : undefined}
      title={t.moveHint}
      onPointerDown={(event) => liftTool(event, tool, "plan")}
      onClick={() => showTool(tool)}
      onKeyDown={keys}
    >
      <Icon svg={TOOL_ICONS[tool]} />
      <span>{t.tool[tool]}</span>
    </button>
  );
}
