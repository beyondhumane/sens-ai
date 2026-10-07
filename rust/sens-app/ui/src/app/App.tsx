import { useEffect, useRef, type CSSProperties, type RefObject } from "react";
import { useStore } from "zustand";
import { Shelf } from "../features/artifacts/Shelf";
import { Capabilities } from "../features/capabilities/Capabilities";
import { NewsView } from "../features/news/News";
import { Panes } from "../features/panes/Panes";
import { project } from "../features/project/store";
import { Rail } from "../features/rail/Rail";
import { SettingsDialog } from "../features/settings/Settings";
import { settingsSheet } from "../features/settings/sheet";
import { openSettings } from "../features/settings/store";
import { toggleConsole } from "../features/terminal/store";
import { welcome } from "../features/welcome/store";
import { Welcome } from "../features/welcome/Welcome";
import { sheets } from "../shared/sheets.js";
import { applyKept, MOST_KEPT, toggleCalm } from "./arrangements";
import { t } from "./copy";
import { Dialog } from "./Dialog";
import { DockDrops, ToolGhost } from "./DockDrops";
import { dialog } from "./modal";
import { areasOf } from "./Plan";
import { chooseFolder, fresh } from "./session";
import { DOCKS, railFolded, shell, toggleRail, visibleIn, type Dock } from "./shell";
import { Splitter } from "./Splitter";
import { ToolsPanel } from "./ToolsPanel";
import { Topbar } from "./Topbar";

const HOTKEYS: Record<string, () => unknown> = { n: fresh, o: chooseFolder, b: toggleRail, ",": () => openSettings(), "`": toggleConsole, "ñ": toggleConsole };

const SHIFTED: Record<string, () => unknown> = {
  KeyL: () => document.getElementById("arrange")?.click(),
  Digit0: toggleCalm,
  ...Object.fromEntries(Array.from({ length: MOST_KEPT }, (_, at) => [`Digit${at + 1}`, () => applyKept(at)])),
};

const SIZES: Record<Dock, string> = { start: "--start-width", end: "--end-width", bottom: "--bottom-height" };

const GROWS: Record<Dock, 1 | -1> = { start: 1, end: -1, bottom: -1 };

function hotkey(event: KeyboardEvent) {
  if (!event.ctrlKey || event.altKey || event.metaKey) return undefined;
  return event.shiftKey ? SHIFTED[event.code] : HOTKEYS[event.key.toLowerCase()];
}

export function App() {
  const railClosed = useStore(shell, railFolded);
  const side = useStore(shell, (s) => s.rail);
  const wide = useStore(shell, (s) => s.wide);
  const sizingNow = useStore(shell, (s) => s.sizing);
  const sizes = useStore(shell, (s) => s.sizes);
  const shown = { start: useShown("start"), end: useShown("end"), bottom: useShown("bottom") };
  const view = useStore(project, (s) => s.view);
  const welcoming = useStore(welcome, (s) => s.open);
  const body = useRef<HTMLDivElement>(null);
  const rail = useRef<HTMLElement>(null);
  const docks = { start: useRef<HTMLElement>(null), end: useRef<HTMLElement>(null), bottom: useRef<HTMLElement>(null) };

  useEffect(() => {
    const outside = (event: PointerEvent) => {
      for (const one of sheets) {
        if (!one.sheet || one.sheet.hidden || one.sheet.contains(event.target as Node) || one.anchor?.contains(event.target as Node)) continue;
        one.shut();
      }
    };
    const keys = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        for (const one of sheets) {
          if (!one.sheet || one.sheet.hidden) continue;
          one.shut();
          one.anchor?.focus();
        }
        return;
      }
      const act = hotkey(event);
      if (!act) return;
      event.preventDefault();
      if (!dialog.getState().open && !settingsSheet.getState().open && !welcome.getState().open) act();
    };
    document.addEventListener("pointerdown", outside);
    document.addEventListener("keydown", keys);
    return () => {
      document.removeEventListener("pointerdown", outside);
      document.removeEventListener("keydown", keys);
    };
  }, []);

  const size = (name: string) => (sizes[name] ? `${sizes[name]}px` : undefined);
  const style = Object.fromEntries([["--rail-width", size("--rail-width")], ...DOCKS.map((dock) => [SIZES[dock], size(SIZES[dock])])]);
  return (
    <>
      <div className="app" inert={welcoming}>
        <Topbar />
        <div
          className="body"
          id="body"
          ref={body}
          data-rail={railClosed ? "closed" : "open"}
          data-rail-side={side}
          data-start={shown.start ? "open" : "closed"}
          data-end={shown.end ? "open" : "closed"}
          data-bottom={shown.bottom ? "open" : "closed"}
          data-sizing={sizingNow || undefined}
          style={{ ...style, gridTemplateAreas: areasOf({ rail: side, wide }) } as CSSProperties}
        >
          <nav className="rail" id="rail" ref={rail} inert={railClosed}>
            <Rail />
          </nav>
          {!railClosed && <Splitter id="rail-split" className="seam" label={t.sidebarWidth} name="--rail-width" host={body} pane={rail} grow={side === "start" ? 1 : -1} />}
          <section className="chat" hidden={Boolean(view)}>
            <Panes />
          </section>
          <section className="view" id="shelf" aria-label={t.artifacts} hidden={view !== "artifacts"}>
            <div className="view-inner" id="shelf-body">
              <Shelf />
            </div>
          </section>
          <section className="view" id="capabilities-view" aria-label={t.capabilities} hidden={view !== "capabilities"}>
            <div className="view-inner" id="capabilities-body">
              <Capabilities />
            </div>
          </section>
          <section className="view" id="news-view" aria-label={t.news} hidden={view !== "news"}>
            <div className="view-inner news">
              <NewsView />
            </div>
          </section>
          {DOCKS.map((dock) => (
            <DockArea key={dock} dock={dock} shown={shown[dock]} host={body} pane={docks[dock]} />
          ))}
          <DockDrops />
        </div>
      </div>
      <ToolGhost />
      <SettingsDialog />
      <Dialog />
      <Welcome />
    </>
  );
}

function useShown(dock: Dock) {
  return useStore(shell, (s) => visibleIn(s, dock) !== null);
}

function DockArea({ dock, shown, host, pane }: { dock: Dock; shown: boolean; host: RefObject<HTMLElement | null>; pane: RefObject<HTMLElement | null> }) {
  return (
    <>
      <ToolsPanel dock={dock} pane={pane} />
      {shown && <Splitter id={`${dock}-split`} className="seam" label={t.dockSize[dock]} name={SIZES[dock]} host={host} pane={pane} grow={GROWS[dock]} axis={dock === "bottom" ? "y" : "x"} />}
    </>
  );
}
