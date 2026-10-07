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
import { roveTabs } from "../shared/tabs";
import { useSheet } from "../shared/useSheet";
import { t } from "./copy";
import { closeDock, closeTab, mainTabsIn, shell, showTool, splitDock, toggleTree, TOOLS, visibleIn, widenPaired, type Dock, type Tool } from "./shell";
import { Splitter } from "./Splitter";
import { liftTool, moveByKey, toolDrag } from "./toolDrag";
import { TOOL_ICONS, ToolsMenu } from "./ToolsMenu";

type View = (props: { hidden: boolean }) => ReactNode;

const VIEWS: Record<Tool, View> = { files: FilesView, changes: ChangesView, map: MapView, web: WebView, terminal: TerminalView, tasks: TasksView };

const PAIR_SIZES: Record<Dock, string> = { start: "--pair-start", end: "--pair-end", bottom: "--pair-bottom" };

export function ToolsPanel({ dock, pane }: { dock: Dock; pane: RefObject<HTMLElement | null> }) {
  const open = useStore(shell, (s) => visibleIn(s, dock) !== null);
  const shown = useStore(shell, (s) => s.shown[dock]);
  const paired = useStore(shell, (s) => s.paired[dock]);
  const homes = useStore(shell, (s) => s.homes);
  const size = useStore(shell, (s) => s.sizes[PAIR_SIZES[dock]]);
  const second = useRef<HTMLDivElement>(null);
  const tools = TOOLS.filter((tool) => homes[tool] === dock && tool !== paired);
  const Paired = paired && VIEWS[paired];
  return (
    <aside
      className="code dock"
      id={`dock-${dock}`}
      data-dock={dock}
      data-tool={shown ?? undefined}
      data-paired={paired ?? undefined}
      aria-label={t.dock[dock]}
      ref={pane}
      inert={!open}
      style={{ [PAIR_SIZES[dock]]: size ? `${size}px` : undefined } as CSSProperties}
    >
      <div className="slot" data-slot="main">
        <div className="slot-grid">
          <ToolTabs dock={dock} />
          <MainActs dock={dock} />
          {tools.map((tool) => {
            const View = VIEWS[tool];
            return <View key={tool} hidden={tool !== shown} />;
          })}
        </div>
      </div>
      {Paired && (
        <>
          <Splitter id={`${dock}-pair-split`} className="seam pair-split" label={t.pairSize} name={PAIR_SIZES[dock]} host={pane} pane={second} grow={-1} axis={dock === "bottom" ? "x" : "y"} />
          <div className="slot" data-slot="pair" ref={second}>
            <div className="slot-grid">
              <PairHead dock={dock} tool={paired} />
              <Paired hidden={false} />
            </div>
          </div>
        </>
      )}
    </aside>
  );
}

function ToolTabs({ dock }: { dock: Dock }) {
  const tabs = useStore(shell, (s) => mainTabsIn(s, dock).join());
  const shown = useStore(shell, (s) => s.shown[dock]);
  const lifted = useStore(toolDrag, (s) => s.tool);
  const aimed = useStore(toolDrag, (s) => (s.on === "window" && s.target?.dock === dock && !s.target.pair ? s.target.before : undefined));
  const sheet = useSheet();
  const tools = tabs ? (tabs.split(",") as Tool[]) : [];
  const current = shown && tools.includes(shown) ? shown : tools[0];
  return (
    <div className="tool-tabs">
      <div
        className="tool-tab-list"
        role="tablist"
        aria-label={t.dock[dock]}
        data-tab-strip={dock}
        data-aimed={aimed === null ? "end" : undefined}
        onKeyDown={(event) => roveTabs(event, tools, current, showTool)}
      >
        {tools.map((tool) => (
          <div className="tool-tab" key={tool} data-tab={tool} data-aimed={aimed === tool ? "true" : undefined} data-lifted={lifted === tool ? "true" : undefined}>
            <button
              type="button"
              className="tool-pick"
              role="tab"
              aria-selected={tool === shown}
              aria-controls={`tool-${tool}`}
              tabIndex={tool === current ? 0 : -1}
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
        <button type="button" className="icon-btn quiet-btn tool-add" ref={sheet.anchor} title={t.openTool} aria-label={t.openTool} aria-haspopup="menu" aria-expanded={sheet.open} onClick={sheet.toggle}>
          <Icon svg={ICONS.plus} />
        </button>
        {createPortal(<ToolsMenu sheet={sheet} id={`tab-menu-${dock}`} dock={dock} />, document.body)}
      </div>
    </div>
  );
}

function MainActs({ dock }: { dock: Dock }) {
  const splits = useStore(shell, (s) => !s.paired[dock] && mainTabsIn(s, dock).length > 1);
  return (
    <div className="slot-acts">
      {splits && (
        <button className="icon-btn quiet-btn split-dock" title={t.split} aria-label={t.split} onClick={() => splitDock(dock)}>
          <Icon svg={dock === "bottom" ? ICONS.splitView : ICONS.splitRows} />
        </button>
      )}
      <button className="icon-btn quiet-btn shut-tool" title={shared.close} aria-label={shared.close} onClick={() => closeDock(dock)}>
        <Icon svg={ICONS.close} />
      </button>
    </div>
  );
}

function PairHead({ dock, tool }: { dock: Dock; tool: Tool }) {
  const lifted = useStore(toolDrag, (s) => s.tool === tool);
  return (
    <>
      <div className="tool-tabs">
        <div className="tool-tab pair-title" data-lifted={lifted ? "true" : undefined}>
          <button
            type="button"
            className="tool-pick"
            aria-keyshortcuts="Alt+ArrowLeft Alt+ArrowRight Alt+ArrowDown"
            title={t.moveHint}
            onPointerDown={(event) => liftTool(event, tool, "window")}
            onKeyDown={(event) => moveByKey(event, tool)}
          >
            <Icon svg={TOOL_ICONS[tool]} />
            <span>{t.tool[tool]}</span>
          </button>
        </div>
      </div>
      <div className="slot-acts">
        <button className="icon-btn quiet-btn widen-pair" title={t.widen} aria-label={t.widen} onClick={() => widenPaired(dock)}>
          <Icon svg={ICONS.widen} />
        </button>
        <button className="icon-btn quiet-btn" title={t.closeTab(t.tool[tool])} aria-label={t.closeTab(t.tool[tool])} onClick={() => closeTab(tool)}>
          <Icon svg={ICONS.close} />
        </button>
      </div>
    </>
  );
}

function Refresh({ id, then }: { id: string; then: () => unknown }) {
  return (
    <button className="icon-btn quiet-btn" id={id} title={t.refresh} aria-label={t.refresh} onClick={() => void then()}>
      <Icon svg={ICONS.refresh} />
    </button>
  );
}

function Section({ tool, hidden, head, tools, children }: { tool: Tool; hidden: boolean; head?: ReactNode; tools?: ReactNode; children: ReactNode }) {
  return (
    <section className="tool" id={`tool-${tool}`} data-tool={tool} role="tabpanel" aria-label={t.tool[tool]} hidden={hidden}>
      <div className="code-head">
        {head}
        {tools && <div className="code-tools">{tools}</div>}
      </div>
      {children}
    </section>
  );
}

function FilesView({ hidden }: { hidden: boolean }) {
  return (
    <Section tool="files" hidden={hidden} head={<ViewerHead />} tools={<FilesTools />}>
      <Files />
    </Section>
  );
}

function ChangesView({ hidden }: { hidden: boolean }) {
  return (
    <Section
      tool="changes"
      hidden={hidden}
      head={
        <span className="marks" id="change-marks">
          <ChangeTotals />
        </span>
      }
      tools={<Refresh id="changes-reload" then={loadChanges} />}
    >
      <div className="tool-body" id="changes" aria-live="polite">
        <ChangesPanel />
      </div>
    </Section>
  );
}

function MapView({ hidden }: { hidden: boolean }) {
  return (
    <Section
      tool="map"
      hidden={hidden}
      head={
        <span className="tool-tally" id="map-tally">
          <MapTally />
        </span>
      }
      tools={
        <>
          <MapModes />
          <Refresh id="map-reload" then={loadMap} />
        </>
      }
    >
      <MapPanel />
    </Section>
  );
}

function WebView({ hidden }: { hidden: boolean }) {
  return (
    <Section tool="web" hidden={hidden} head={<Address />} tools={<Outside />}>
      <div className="site" id="site">
        <Web />
      </div>
    </Section>
  );
}

function TerminalView({ hidden }: { hidden: boolean }) {
  return (
    <Section tool="terminal" hidden={hidden} head={<ConsoleTabs />} tools={<ConsoleTools />}>
      <ConsolePanel />
    </Section>
  );
}

function TasksView({ hidden }: { hidden: boolean }) {
  return (
    <Section
      tool="tasks"
      hidden={hidden}
      head={
        <span className="tool-tally" id="task-tally">
          <TaskTally />
        </span>
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
      <button className="icon-btn quiet-btn" id="toggle-tree" title={t.fileTree} aria-pressed={shown} onClick={toggleTree}>
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
