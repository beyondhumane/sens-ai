import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import { commands } from "../ipc/commands";
import { createPortal } from "react-dom";
import { useStore } from "zustand";
import { focused } from "../features/panes/store";
import { project } from "../features/project/store";
import { runningTasks, tasks } from "../features/tasks/store";
import { openUpdate } from "../features/updates/UpdatePanel";
import { updates } from "../features/updates/store";
import { shared } from "../shared/copy";
import { stem } from "../shared/format.js";
import { Icon } from "../shared/Icon";
import { ICONS } from "../shared/icons.js";
import { Mark } from "../shared/Mark";
import { useSheet } from "../shared/useSheet";
import { ArrangeButton } from "./Arrangement";
import { t } from "./copy";
import { railFolded, shell, toggleRail } from "./shell";
import { ToolsMenu } from "./ToolsMenu";

export function Topbar() {
  const closed = useStore(shell, railFolded);
  const end = useStore(shell, (s) => s.rail === "end");
  const label = closed ? t.showSidebar : t.hideSidebar;
  return (
    <header className="topbar">
      <button
        className="icon-btn rail-toggle"
        id="toggle-rail"
        aria-controls="rail"
        aria-expanded={!closed}
        aria-keyshortcuts="Control+B"
        title={`${label} (${shared.ctrl}+B)`}
        aria-label={label}
        onClick={toggleRail}
      >
        <Icon svg={end ? (closed ? ICONS.panelRightOpen : ICONS.panelRightClose) : closed ? ICONS.panelOpen : ICONS.panelClose} />
      </button>
      <div className="brand">
        <Mark size={18} micro />
        <b>sens</b>
      </div>
      <ProjectTitle />
      <UpdatePill />
      <Window />
    </header>
  );
}

function ProjectTitle() {
  const root = useStore(project, (s) => s.root);
  if (!root) return null;
  return (
    <div className="topbar-title" id="project-title">
      {stem(root)}
    </div>
  );
}

function UpdatePill() {
  const latest = useStore(updates, (s) => s.latest);
  if (!latest) return null;
  return (
    <button
      className="update-pill"
      id="update"
      aria-label={t.updateAvailable(latest.version)}
      title={t.versionAvailable(latest.version)}
      onClick={(event) => openUpdate(event.currentTarget)}
    >
      <span className="update-ping" aria-hidden="true" />
      <Icon svg={ICONS.update} />
      <span>{t.update}</span>
      <span className="update-version">{latest.version}</span>
    </button>
  );
}

export function Window({ tools = true }: { tools?: boolean }) {
  const frame = getCurrentWindow();
  const [wide, setWide] = useState(false);

  useEffect(() => {
    const sync = () => frame.isMaximized().then(setWide, () => {});
    sync();
    const stop = frame.onResized(sync);
    return () => {
      stop.then((unlisten) => unlisten());
    };
  }, []);

  const calm = useStore(shell, (s) => s.calm);
  const grow = wide ? t.restore : t.maximize;
  return (
    <div className="win" id="win" data-max={String(wide)}>
      {tools && <FocusButton />}
      {tools && <ArrangeButton />}
      {tools && !calm && <ToolsButton />}
      <button id="win-min" title={t.minimize} aria-label={t.minimize} onClick={() => frame.minimize()}>
        <Icon svg={ICONS.minimize} />
      </button>
      <button id="win-max" title={grow} aria-label={grow} onClick={() => frame.toggleMaximize().then(() => frame.isMaximized().then(setWide))}>
        <span className="grow">
          <Icon svg={ICONS.maximize} />
        </span>
        <span className="restore">
          <Icon svg={ICONS.restore} />
        </span>
      </button>
      <button className="shut" id="win-close" title={shared.close} aria-label={shared.close} onClick={() => frame.close()}>
        <Icon svg={ICONS.shutWindow} />
      </button>
    </div>
  );
}

function FocusButton() {
  const [keys, setKeys] = useState("");
  const learn = () => void commands.shortcutState().then((shortcut) => setKeys(shortcut.named)).catch(() => {});

  useEffect(learn, []);

  function enter() {
    const { root, session } = focused().desk.getState();
    void commands.barFocus({ root, session, text: "" }).catch(() => {});
  }

  return (
    <button id="focus-mode" title={keys ? t.focusModeKeys(keys) : t.focusMode} aria-label={t.focusMode} onPointerEnter={learn} onClick={enter}>
      <Icon svg={ICONS.focus} />
    </button>
  );
}

function ToolsButton() {
  const sheet = useSheet();
  const running = useStore(tasks, () => runningTasks());
  return (
    <>
      <button
        className="tools-btn"
        id="tools"
        ref={sheet.anchor}
        title={t.tools}
        aria-label={t.tools}
        aria-haspopup="menu"
        aria-expanded={sheet.open}
        data-running={String(running > 0)}
        onClick={sheet.toggle}
      >
        <Icon svg={ICONS.moreVertical} />
      </button>
      {createPortal(<ToolsMenu sheet={sheet} id="tool-menu" />, document.body)}
    </>
  );
}
