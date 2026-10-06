import type { ReactNode } from "react";
import { useStore } from "zustand";
import type { ProjectMap, Reached, Region } from "../../ipc/types";
import { FileIcon } from "../../shared/FileIcon";
import { parentOf, stem } from "../../shared/format.js";
import { Icon } from "../../shared/Icon";
import { ICONS } from "../../shared/icons.js";
import { showFile } from "../files/view";
import { project } from "../project/store";
import { t } from "./copy";
import { MapGraph } from "./MapGraph";
import { atlas, inspect, leaveReach, showMapAs, unfoldRegion } from "./store";

export function MapTally() {
  const map = useStore(atlas, (s) => s.map);
  if (!map) return null;
  const files = map.regions.reduce((total, region) => total + region.files.length, 0);
  return <>{t.tally(map.regions.length, files)}</>;
}

export function MapModes() {
  const mode = useStore(atlas, (s) => s.mode);
  return (
    <div className="segment">
      <button type="button" aria-pressed={mode === "list"} onClick={() => showMapAs("list")}>
        {t.list}
      </button>
      <button type="button" aria-pressed={mode === "graph"} onClick={() => showMapAs("graph")}>
        {t.graphMode}
      </button>
    </div>
  );
}

export function MapPanel() {
  const root = useStore(project, (s) => s.work);
  const map = useStore(atlas, (s) => s.map);
  const fault = useStore(atlas, (s) => s.fault);
  const reach = useStore(atlas, (s) => s.reach);
  const mode = useStore(atlas, (s) => s.mode);

  if (fault) return <p className="none fault">{fault}</p>;
  if (!root) return <p className="none">{t.noFolder}</p>;
  if (!map) return <p className="none">{t.indexing}</p>;
  if (reach) return <ReachView />;
  if (!map.regions.length) return <p className="none">{t.empty}</p>;
  if (mode === "graph") return <GraphView map={map} />;
  return (
    <>
      {map.central.length > 0 && (
        <Part title={t.central} said={t.centralSaid}>
          {map.central.map((one) => (
            <FileRow key={one.path} path={one.path} aside={t.dependents(one.dependents)} />
          ))}
        </Part>
      )}
      <Part title={t.areas}>
        {map.regions.map((region) => (
          <RegionRow key={region.name} region={region} />
        ))}
      </Part>
      {map.strays.length > 0 && (
        <Part title={t.strays} said={t.straysSaid}>
          {map.strays.map((one) => (
            <FileRow key={one.path} path={one.path} aside={`→ ${one.area}`} />
          ))}
        </Part>
      )}
      <Cycles map={map} />
    </>
  );
}

function GraphView({ map }: { map: ProjectMap }) {
  const picked = useStore(atlas, (s) => map.regions.find((region) => region.name === s.picked));
  return (
    <>
      <MapGraph map={map} />
      {picked ? (
        <Part title={t.areas}>
          <RegionRow region={picked} />
        </Part>
      ) : (
        <p className="none">{t.pickArea}</p>
      )}
      <Cycles map={map} />
    </>
  );
}

function Cycles({ map }: { map: ProjectMap }) {
  if (!map.cycles.length) return null;
  return (
    <Part title={t.cycles} said={t.cyclesSaid}>
      {map.cycles.map((cycle) => (
        <div className="map-cycle" key={cycle.join("~")}>
          <span className="map-cycle-mark" aria-hidden="true">
            <Icon svg={ICONS.cycle} />
          </span>
          <div className="map-cycle-files">
            {cycle.map((path) => (
              <FileRow key={path} path={path} />
            ))}
          </div>
        </div>
      ))}
    </Part>
  );
}

function Part({ title, said, children }: { title: string; said?: string; children: ReactNode }) {
  return (
    <section className="map-part" aria-label={title}>
      <h3 className="label map-title" title={said}>
        {title}
      </h3>
      {children}
    </section>
  );
}

function RegionRow({ region }: { region: Region }) {
  const open = useStore(atlas, (s) => s.unfolded.has(region.name));
  return (
    <details className="region" open={open} onToggle={(event) => unfoldRegion(region.name, event.currentTarget.open)}>
      <summary>
        <span className="chev">
          <Icon svg={ICONS.shut} />
        </span>
        <span className="region-name">{region.name}</span>
        <span className="region-count">{t.files(region.files.length)}</span>
      </summary>
      {open && (
        <div className="region-body">
          {region.doors.length > 0 && <Facts label={t.doors} items={region.doors.map(stem)} />}
          {region.uses.length > 0 && <Facts label={t.uses} items={region.uses} />}
          {region.exports.length > 0 && <Facts label={t.exports} items={region.exports} />}
          <div className="region-files">
            {region.files.map((path) => (
              <FileRow key={path} path={path} door={region.doors.includes(path)} />
            ))}
          </div>
        </div>
      )}
    </details>
  );
}

function Facts({ label, items }: { label: string; items: string[] }) {
  return (
    <p className="facts">
      <span className="facts-label">{label}</span>
      <span className="facts-items">{items.join(" · ")}</span>
    </p>
  );
}

function FileRow({ path, aside, door = false }: { path: string; aside?: string; door?: boolean }) {
  return (
    <div className="map-file" data-door={String(door)}>
      <button type="button" className="map-pick" title={`${t.inspect}: ${path}`} onClick={() => inspect(path)}>
        <FileIcon path={path} />
        <span className="name">{stem(path)}</span>
        <span className="dirname">{parentOf(path)}</span>
        {aside && <span className="map-aside">{aside}</span>}
      </button>
      <button type="button" className="jump" title={t.openInFiles} aria-label={`${t.openInFiles}: ${path}`} onClick={() => showFile(path)}>
        <Icon svg={ICONS.fileCode} />
      </button>
    </div>
  );
}

function ReachView() {
  const reach = useStore(atlas, (s) => s.reach)!;
  const steps = [...new Set(reach.dependents.map((one) => one.steps))];
  return (
    <>
      <button type="button" className="map-back" onClick={leaveReach}>
        <Icon svg={ICONS.back} />
        <span>{t.back}</span>
      </button>
      <div className="map-subject">
        <FileRow path={reach.file} aside={reach.area} />
      </div>
      <Part title={t.reachOf}>
        {steps.length ? (
          steps.map((step) => <Step key={step} step={step} found={reach.dependents.filter((one) => one.steps === step)} />)
        ) : (
          <p className="none">{t.nothingDepends}</p>
        )}
      </Part>
      <Part title={t.tests}>
        {reach.tests.length ? reach.tests.map((one) => <FileRow key={one.path} path={one.path} aside={t.steps(one.steps)} />) : <p className="none">{t.noTests}</p>}
      </Part>
    </>
  );
}

function Step({ step, found }: { step: number; found: Reached[] }) {
  return (
    <div className="map-step">
      <p className="map-step-label">{t.steps(step)}</p>
      {found.map((one) => (
        <FileRow key={one.path} path={one.path} aside={one.area} />
      ))}
    </div>
  );
}
