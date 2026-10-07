import { useLayoutEffect } from "react";
import { useStore } from "zustand";
import { focused, panes } from "../features/panes/store";
import { runningTasks, tasks } from "../features/tasks/store";
import { consoles, runningConsoles } from "../features/terminal/store";
import { anchorMenu } from "../shared/anchorMenu";
import { Icon } from "../shared/Icon";
import { ICONS } from "../shared/icons.js";
import type { Sheet } from "../shared/useSheet";
import { t } from "./copy";
import { moveTool, panelShows, shell, showTool, TOOLS, type Dock, type Tool } from "./shell";

export const TOOL_ICONS: Record<Tool, string> = {
  files: ICONS.files,
  changes: ICONS.compare,
  map: ICONS.map,
  web: ICONS.globe,
  terminal: ICONS.terminal,
  tasks: ICONS.activity,
};

export const DOCK_ICONS: Record<Dock, string> = { start: ICONS.panelLeft, end: ICONS.panelRight, bottom: ICONS.panelBottom };

export function ToolsMenu({ sheet, id, dock }: { sheet: Sheet; id: string; dock?: Dock }) {
  const pane = useStore(panes, () => focused());
  const dirty = useStore(pane.desk, (s) => s.repo?.dirty ?? 0);
  const running = useStore(tasks, () => runningTasks());
  const shells = useStore(consoles, () => runningConsoles());
  useStore(shell, (s) => JSON.stringify([s.calm, s.open, s.shown, s.homes]));

  useLayoutEffect(() => {
    if (sheet.open && sheet.sheet.ref.current && sheet.anchor.current) anchorMenu(sheet.sheet.ref.current, sheet.anchor.current);
  }, [sheet.open]);

  const counts: Partial<Record<Tool, number>> = { changes: dirty, tasks: running, terminal: shells };
  const count = (tool: Tool) => (counts[tool] ? String(counts[tool]) : "");
  return (
    <div className="sheet menu float-menu tool-menu" id={id} role="menu" aria-label={t.tools} {...sheet.sheet}>
      {TOOLS.map((tool) => (
        <button
          key={tool}
          type="button"
          className="menu-item"
          tabIndex={-1}
          role="menuitemradio"
          aria-checked={panelShows(tool)}
          onClick={() => {
            sheet.shut();
            if (dock) moveTool(tool, dock);
            else showTool(tool);
          }}
        >
          <span className="act-icon">
            <Icon svg={TOOL_ICONS[tool]} />
          </span>
          <span className="mode-text">
            <span>{t.tool[tool]}</span>
            <span className="mode-sub">{t.toolSaid[tool]}</span>
          </span>
          <span className="tool-count">{count(tool)}</span>
        </button>
      ))}
    </div>
  );
}
