import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useStore } from "zustand";
import { anchorMenu } from "../shared/anchorMenu";
import { shared } from "../shared/copy";
import { Icon } from "../shared/Icon";
import { ICONS } from "../shared/icons.js";
import { useSheet, type Sheet } from "../shared/useSheet";
import { applyKept, applyStart, arrangements, forget, forgetUndo, keepNow, MOST_KEPT, signature, startFrom, STARTS, undo } from "./arrangements";
import { t } from "./copy";
import { LivePlan, MiniPlan } from "./Plan";
import { arrangementOf, placeRail, shell, spanBottom, type Side } from "./shell";

const SIDES: Side[] = ["start", "end"];
const SIDE_ICONS: Record<Side, string> = { start: ICONS.panelLeft, end: ICONS.panelRight };

function Keys({ keys }: { keys: string[] }) {
  return (
    <span className="hint-keys">
      {keys.map((key) => (
        <kbd key={key}>{key}</kbd>
      ))}
    </span>
  );
}

export function ArrangeButton() {
  const sheet = useSheet();
  return (
    <>
      <button
        className="tools-btn"
        id="arrange"
        ref={sheet.anchor}
        title={t.arrangeKeys(`${shared.ctrl}+${shared.shift}+L`)}
        aria-label={t.arrange}
        aria-haspopup="dialog"
        aria-expanded={sheet.open}
        aria-keyshortcuts="Control+Shift+L"
        onClick={sheet.toggle}
      >
        <Icon svg={ICONS.arrange} />
      </button>
      {createPortal(<ArrangeSheet sheet={sheet} />, document.body)}
    </>
  );
}

function ArrangeSheet({ sheet }: { sheet: Sheet }) {
  const now = useStore(shell, (s) => signature(arrangementOf(s)));
  const rail = useStore(shell, (s) => s.rail);
  const wide = useStore(shell, (s) => s.wide);
  const kept = useStore(arrangements, (s) => s.kept);
  const before = useStore(arrangements, (s) => s.before);

  useLayoutEffect(() => {
    if (sheet.open && sheet.sheet.ref.current && sheet.anchor.current) anchorMenu(sheet.sheet.ref.current, sheet.anchor.current);
  }, [sheet.open]);

  useEffect(() => {
    if (sheet.open) sheet.sheet.ref.current?.querySelector<HTMLElement>(".start")?.focus();
    else forgetUndo();
  }, [sheet.open]);

  return (
    <div className="sheet float-menu arrange" id="arrange-sheet" role="dialog" aria-label={t.arrange} {...sheet.sheet}>
      <header className="arrange-head">
        <span className="label">{t.arrange}</span>
        <Keys keys={[shared.ctrl, shared.shift, "L"]} />
      </header>
      <p className="arrange-said">{t.arrangeSaid}</p>
      <LivePlan />
      <div className="arrange-row">
        <span>{t.railSide}</span>
        <span className="segmented" role="radiogroup" aria-label={t.railSide}>
          {SIDES.map((side) => (
            <button key={side} type="button" role="radio" aria-checked={rail === side} onClick={() => placeRail(side)}>
              <Icon svg={SIDE_ICONS[side]} />
              <span>{t.dock[side]}</span>
            </button>
          ))}
        </span>
      </div>
      <div className="arrange-row">
        <span id="arrange-wide">{t.wide}</span>
        <button type="button" className="switch" role="switch" aria-checked={wide} aria-labelledby="arrange-wide" onClick={() => spanBottom(!wide)} />
      </div>
      <section className="arrange-part" aria-label={t.starts}>
        <span className="label">{t.starts}</span>
        <div className="starts">
          {STARTS.map((start) => {
            const arrangement = startFrom(start);
            return (
              <button key={start} type="button" className="start" aria-pressed={signature(arrangement) === now} title={t.startSaid[start]} onClick={() => applyStart(start)}>
                <MiniPlan arrangement={arrangement} />
                <span>{t.start[start]}</span>
              </button>
            );
          })}
        </div>
      </section>
      <section className="arrange-part" aria-label={t.kept}>
        {kept.length > 0 && <span className="label">{t.kept}</span>}
        {kept.map((one, at) => (
          <div className="kept-row" key={`${at}-${one.name}`}>
            <button type="button" className="kept-pick" aria-pressed={signature(one.arrangement) === now} onClick={() => applyKept(at)}>
              <MiniPlan arrangement={one.arrangement} />
              <span className="kept-name">{one.name}</span>
              <Keys keys={[shared.ctrl, shared.shift, String(at + 1)]} />
            </button>
            <button type="button" className="icon-btn" title={t.forget(one.name)} aria-label={t.forget(one.name)} onClick={() => forget(at)}>
              <Icon svg={ICONS.trash} />
            </button>
          </div>
        ))}
        {kept.length < MOST_KEPT && <KeepForm count={kept.length} />}
      </section>
      {before && (
        <div className="arrange-undo" role="status">
          <span>{t.changed}</span>
          <button type="button" className="chipbtn" onClick={undo}>
            <Icon svg={ICONS.undo} />
            <span>{t.undo}</span>
          </button>
        </div>
      )}
    </div>
  );
}

function KeepForm({ count }: { count: number }) {
  const [naming, setNaming] = useState(false);
  const field = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (naming) field.current?.focus();
  }, [naming]);

  if (!naming)
    return (
      <button type="button" className="keep-now" onClick={() => setNaming(true)}>
        <Icon svg={ICONS.keep} />
        <span>{t.keepNow}</span>
      </button>
    );

  const save = () => {
    keepNow(field.current?.value.trim() || t.keptName(count + 1));
    setNaming(false);
  };
  return (
    <form
      className="keep-form"
      onSubmit={(event) => {
        event.preventDefault();
        save();
      }}
    >
      <input ref={field} className="field" aria-label={t.keepName} placeholder={t.keptName(count + 1)} maxLength={40} />
      <button type="submit" className="chipbtn">
        <span>{t.keep}</span>
      </button>
    </form>
  );
}
