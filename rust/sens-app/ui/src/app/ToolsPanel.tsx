import { useRef, type CSSProperties, type ReactNode, type RefObject } from "react";
import { createPortal } from "react-dom";
import { useStore } from "zustand";
import { ChangeTotals, ChangesPanel } from "../features/changes/Changes";
import { loadChanges } from "../features/changes/store";
import { Tree } from "../features/files/Tree";
import { Viewer, ViewerHead, ViewerModes } from "../features/files/Viewer";
import { MapPanel, MapTally } from "../features/map/MapPanel";
import { loadMap } from "../features/map/store";
import { TaskTally, TasksPanel } from "../features/tasks/TasksPanel";
import { ConsolePanel, ConsoleTabs, ConsoleTools } from "../features/terminal/Consoles";
import { Address, Outside, Web } from "../features/web/Web";
import { shared } from "../shared/copy";
import { Icon } from "../shared/Icon";
import { ICONS } from "../shared/icons.js";
import { useSheet } from "../shared/useSheet";
import { t } from "./copy";
import { closeTab, closeTools, shell, showTool, toggleTree, type Tool } from "./shell";
import { Splitter } from "./Splitter";
import { TOOL_ICONS, ToolsMenu } from "./ToolsMenu";

export function ToolsPanel({ pane }: { pane?: RefObject<HTMLElement | null> }) {
  const tool = useStore(shell, (s) => s.tool);
  return (
    <aside className="code" id="code" data-tool={tool} ref={pane}>
      <ToolTabs />
      <Section tool="files" head={<ViewerHead />} tools={<FilesTools />}>
        <Files />
      </Section>
      <Section
        tool="changes"
        head={
          <>
            <span className="tool-name">{t.tool.changes}</span>
            <span className="marks" id="change-marks">
              <ChangeTotals />
            </span>
          </>
        }
        tools={<Refresh id="changes-reload" then={loadChanges} />}
      >
        <div className="tool-body" id="changes" aria-live="polite">
          <ChangesPanel />
        </div>
      </Section>
      <Section
        tool="map"
        head={
          <>
            <span className="tool-name">{t.tool.map}</span>
            <span className="tool-tally" id="map-tally">
              <MapTally />
            </span>
          </>
        }
        tools={<Refresh id="map-reload" then={loadMap} />}
      >
        <div className="tool-body map" id="map" aria-live="polite">
          <MapPanel />
        </div>
      </Section>
      <Section tool="web" head={<Address />} tools={<Outside />}>
        <div className="site" id="site">
          <Web />
        </div>
      </Section>
      <Section tool="terminal" head={<ConsoleTabs />} tools={<ConsoleTools />}>
        <ConsolePanel />
      </Section>
      <Section
        tool="tasks"
        head={
          <>
            <span className="tool-name">{t.tool.tasks}</span>
            <span className="tool-tally" id="task-tally">
              <TaskTally />
            </span>
          </>
        }
      >
        <div className="tool-body" id="tasks">
          <TasksPanel />
        </div>
      </Section>
    </aside>
  );
}

function ToolTabs() {
  const tabs = useStore(shell, (s) => s.tabs);
  const shown = useStore(shell, (s) => s.tool);
  const sheet = useSheet();
  return (
    <div className="tool-tabs">
      <div className="tool-tab-list" role="tablist" aria-label={t.tools}>
        {tabs.map((tool) => (
          <div className="tool-tab" key={tool}>
            <button type="button" className="tool-pick" role="tab" aria-selected={tool === shown} aria-controls={`tool-${tool}`} onClick={() => showTool(tool)}>
              <Icon svg={TOOL_ICONS[tool]} />
              <span>{t.tool[tool]}</span>
            </button>
            <button type="button" className="tool-tab-shut" title={t.closeTab(t.tool[tool])} aria-label={t.closeTab(t.tool[tool])} onClick={() => closeTab(tool)}>
              <Icon svg={ICONS.close} />
            </button>
          </div>
        ))}
        <button type="button" className="icon-btn tool-add" ref={sheet.anchor} title={t.openTool} aria-label={t.openTool} aria-haspopup="menu" aria-expanded={sheet.open} onClick={sheet.toggle}>
          <Icon svg={ICONS.plus} />
        </button>
        {createPortal(<ToolsMenu sheet={sheet} id="tab-menu" />, document.body)}
      </div>
      <button className="icon-btn shut-tool" title={shared.close} aria-label={shared.close} onClick={closeTools}>
        <Icon svg={ICONS.close} />
      </button>
    </div>
  );
}

function Refresh({ id, then }: { id: string; then: () => unknown }) {
  return (
    <button className="icon-btn" id={id} title={t.refresh} aria-label={t.refresh} onClick={() => void then()}>
      <Icon svg={ICONS.refresh} />
    </button>
  );
}

function Section({ tool, head, tools, children }: { tool: Tool; head: ReactNode; tools?: ReactNode; children: ReactNode }) {
  const shown = useStore(shell, (s) => s.tool === tool);
  return (
    <section className="tool" id={`tool-${tool}`} data-tool={tool} role="tabpanel" aria-label={t.tool[tool]} hidden={!shown}>
      <div className="code-head">
        {head}
        {tools && <div className="code-tools">{tools}</div>}
      </div>
      {children}
    </section>
  );
}

function FilesTools() {
  const shown = useStore(shell, (s) => s.treeShown);
  return (
    <>
      <ViewerModes />
      <button className="icon-btn" id="toggle-tree" title={t.fileTree} aria-pressed={shown} onClick={toggleTree}>
        <Icon svg={ICONS.treeLines} />
      </button>
    </>
  );
}

function Files() {
  const shown = useStore(shell, (s) => s.treeShown);
  const width = useStore(shell, (s) => s.sizes["--tree-width"]);
  const body = useRef<HTMLDivElement>(null);
  const tree = useRef<HTMLDivElement>(null);
  return (
    <div className="code-body" id="code-body" data-tree={shown ? "shown" : "hidden"} ref={body} style={{ "--tree-width": width ? `${width}px` : undefined } as CSSProperties}>
      <div className="tree" id="tree" ref={tree}>
        <Tree />
      </div>
      <Splitter id="tree-split" label={t.treeWidth} name="--tree-width" host={body} pane={tree} grow={1} />
      <Viewer />
    </div>
  );
}
