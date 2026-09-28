import { useStore } from "zustand";
import { draft } from "../../app/session";
import { Callout } from "../../shared/Callout";
import { compact } from "../../shared/format.js";
import { localeNow } from "../../shared/i18n";
import { Icon } from "../../shared/Icon";
import { ICONS } from "../../shared/icons.js";
import { modelName, models } from "../models/store";
import { usePane } from "../panes/context";
import { t } from "./copy";
import { effortName } from "./knobs.copy";
import { cacheBreak, undoChange, type Changed } from "./store";

type Knobs = { model: string; thinking?: boolean; effort?: string };

function described(changed: Changed[], knobs: Knobs) {
  const said = changed.map((knob) => {
    if (knob === "model") return modelName(knobs.model);
    if (knob === "thinking") return knobs.thinking ? t.change.thinkingOn : t.change.thinkingOff;
    return t.change.effortAt(effortName(knobs.effort ?? ""));
  });
  return new Intl.ListFormat(localeNow(), { type: "conjunction" }).format(said);
}

export function CacheBreakNotice() {
  const pane = usePane();
  useStore(pane.chat, (s) => s.ranOn);
  useStore(pane.chat, (s) => s.ranWith);
  useStore(pane.chat, (s) => s.context);
  useStore(pane.desk, (s) => s.choice);
  useStore(pane.desk, (s) => s.effort);
  useStore(pane.desk, (s) => s.thinking);
  useStore(models, (s) => s.known);
  const root = useStore(pane.desk, (s) => s.root);
  const found = cacheBreak(pane);
  if (!found) return null;
  const { changed, was, now, tokens, undoable } = found;
  const title = changed.length > 1 ? t.change.title.several : t.change.title[changed[0]];

  return (
    <Callout icon={ICONS.info} title={title} said={t.change.said(described(changed, was), described(changed, now), tokens ? compact(tokens) : "")}>
      <button className="primary" onClick={() => draft(root, pane)}>
        <Icon svg={ICONS.messagePlus} />
        {t.change.newSession}
      </button>
      {undoable && (
        <button className="quiet" onClick={() => undoChange(pane)}>
          {t.change.undo}
        </button>
      )}
    </Callout>
  );
}
