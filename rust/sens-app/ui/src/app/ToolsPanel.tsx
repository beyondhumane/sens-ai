import { useRef, type CSSProperties, type ReactNode, type RefObject } from "react";
import { createPortal } from "react-dom";
import { useStore } from "zustand";
import { ChangeTotals, ChangesPanel } from "../features/changes/Changes";
import { loadChanges } from "../features/changes/store";
import { Tree } from "../features/files/Tree";
import { Viewer, ViewerHead, ViewerModes } from "../features/files/Viewer";
import { MapModes, MapPanel, MapTally } from "../features/map/MapPanel";
import { loadMap } from "../features/map/store";
import { TaskTally, TasksPanel } from "../features/tasks/TasksPanel";
import { ConsolePanel, ConsoleTabs, ConsoleTools } from "../features/terminal/Consoles";
import { Address, Outside, Web } from "../features/web/Web";
import { shared } from "../shared/copy";
import { Icon } from "../shared/Icon";
import { ICONS } from "../shared/icons.js";
import { useSheet } from "../shared/useSheet";
import { t } from "./copy";
import { closeDock, closeTab, shell, showTool, tabsIn, toggleTree, TOOLS, visibleIn, type Dock, type Tool } from "./shell";
import { Splitter } from "./Splitter";
import { liftTool, moveByKey, toolDrag } from "./toolDrag";
import { TOOL_ICONS, ToolsMenu } from "./ToolsMenu";

const VIEWS: Record<Tool, () => ReactNode> = { files: FilesView, changes: ChangesView, map: MapView, web: WebView, terminal: TerminalView, tasks: TasksView };

export function ToolsPanel({ dock, pane }: { dock: Dock; pane?: RefObject<HTMLElement | null> }) {
  const open = useStore(shell, (s) => visibleIn(s, dock) !== null);
  const shown = useStore(shell, (s) => s.shown[dock]);
  const homes = useStore(shell, (s) => s.homes);
  return (
    <aside className="code dock" id={`dock-${dock}`} data-dock={dock} data-tool={shown ?? undefined} aria-label={t.dock[dock]} ref={pane} inert={!open}>
      <ToolTabs dock={dock} />
      {TOOLS.filter((tool) => homes[tool] === dock).map((tool) => {
        const View = VIEWS[tool];
        return <View key={tool} />;
      })}
    </aside>
  );
}

function ToolTabs({ dock }: { dock: Dock }) {
  const all = useStore(shell, (s) => s.tabs);
  const homes = useStore(shell, (s) => s.homes);
  const shown = useStore(shell, (s) => s.shown[dock]);
  const lifted = useStore(toolDrag, (s) => s.tool);
  const aimed = useStore(toolDrag, (s) => (s.on === "window" && s.target?.dock === dock ? s.target.before : undefined));
  const tabs = tabsIn({ tabs: all, homes }, dock);
  const sheet = useSheet();
  return (
    <div className="tool-tabs">
      <div className="tool-tab-list" role="tablist" aria-label={t.dock[dock]} data-tab-strip={dock} data-aimed={aimed === null ? "end" : undefined}>
        {tabs.map((tool) => (
          <div className="tool-tab" key={tool} data-tab={tool} data-aimed={aimed === tool ? "true" : undefined} data-lifted={lifted === tool ? "true" : undefined}>
            <button
              type="button"
              className="tool-pick"
              role="tab"
              aria-selected={tool === shown}
              aria-controls={`tool-${tool}`}
              aria-keyshortcuts="Alt+ArrowLeft Alt+ArrowRight Alt+ArrowDown"
              title={t.moveHint}
              onClick={() => showTool(tool)}
              onPointerDown={(event) => liftTool(event, tool, "window")}
              onKeyDown={(event) => moveByKey(event, tool)}
            >
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
        {createPortal(<ToolsMenu sheet={sheet} id={`tab-menu-${dock}`} dock={dock} />, document.body)}
      </div>
      <button className="icon-btn shut-tool" title={shared.close} aria-label={shared.close} onClick={() => closeDock(dock)}>
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
  const shown = useStore(shell, (s) => s.shown[s.homes[tool]] === tool);
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

function Named({ tool, children }: { tool: Tool; children: ReactNode }) {
  return (
    <>
      <span className="tool-name">{t.tool[tool]}</span>
      {children}
    </>
  );
}

function FilesView() {
  return (
    <Section tool="files" head={<ViewerHead />} tools={<FilesTools />}>
      <Files />
    </Section>
  );
}

function ChangesView() {
  return (
    <Section
      tool="changes"
      head={
        <Named tool="changes">
          <span className="marks" id="change-marks">
            <ChangeTotals />
          </span>
        </Named>
      }
      tools={<Refresh id="changes-reload" then={loadChanges} />}
    >
      <div className="tool-body" id="changes" aria-live="polite">
        <ChangesPanel />
      </div>
    </Section>
  );
}

function MapView() {
  return (
    <Section
      tool="map"
      head={
        <Named tool="map">
          <span className="tool-tally" id="map-tally">
            <MapTally />
          </span>
        </Named>
      }
      tools={
        <>
          <MapModes />
          <Refresh id="map-reload" then={loadMap} />
        </>
      }
    >
      <div className="tool-body map" id="map" aria-live="polite">
        <MapPanel />
      </div>
    </Section>
  );
}

function WebView() {
  return (
    <Section tool="web" head={<Address />} tools={<Outside />}>
      <div className="site" id="site">
        <Web />
      </div>
    </Section>
  );
}

function TerminalView() {
  return (
    <Section tool="terminal" head={<ConsoleTabs />} tools={<ConsoleTools />}>
      <ConsolePanel />
    </Section>
  );
}

function TasksView() {
  return (
    <Section
      tool="tasks"
      head={
        <Named tool="tasks">
          <span className="tool-tally" id="task-tally">
            <TaskTally />
          </span>
        </Named>
      }
    >
      <div className="tool-body" id="tasks">
        <TasksPanel />
      </div>
    </Section>
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
