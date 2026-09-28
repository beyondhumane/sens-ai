import { useStore } from "zustand";
import { lineCount } from "../features/composer/Clip";
import { dropFile, dropPicture } from "../features/composer/store";
import { Icon } from "../shared/Icon";
import { ICONS } from "../shared/icons.js";
import { t } from "./copy";
import { bar, dropShot, dropText, own, takeClip, takeShot } from "./store";

function Taken({ icon, name, mono, remove }: { icon: string; name: string; mono?: string; remove: () => void }) {
  return (
    <span className="bar-chip on" role="listitem">
      <Icon svg={icon} />
      <span>{name}</span>
      {mono && <span className="mono">{mono}</span>}
      <button type="button" className="bar-chip-drop" aria-label={t.remove(name)} title={t.remove(name)} onClick={remove}>
        <Icon svg={ICONS.remove} />
      </button>
    </span>
  );
}

export function Chips() {
  const front = useStore(bar, (s) => s.front);
  const clip = useStore(bar, (s) => s.clip);
  const shot = useStore(bar, (s) => s.shot);
  const copied = useStore(bar, (s) => s.copied);
  const pasted = useStore(own.desk, (s) => s.pasted);
  const attached = useStore(own.desk, (s) => s.attached);
  const offerShot = front && (shot === "offered" || shot === "taking");
  const offerClip = clip && (copied === "offered" || copied === "taking");
  if (!offerShot && !offerClip && !pasted.length && !attached.length) return null;
  return (
    <div className="bar-chips" role="list">
      {offerShot && (
        <button type="button" className="bar-chip" role="listitem" aria-busy={shot === "taking"} title={t.shotAdd(front.app)} onClick={takeShot}>
          <Icon svg={ICONS.scan} />
          <span>{t.shot(front.app)}</span>
        </button>
      )}
      {offerClip && (
        <button type="button" className="bar-chip" role="listitem" aria-busy={copied === "taking"} title={t.clipAdd} onClick={takeClip}>
          <Icon svg={ICONS.clipboard} />
          <span>{t.clip}</span>
          <span className="mono">{clip.preview}</span>
        </button>
      )}
      {pasted.map((picture) =>
        picture.name === front?.app ? (
          <Taken key={picture.url} icon={ICONS.scan} name={picture.name} remove={() => dropShot(picture.url)} />
        ) : (
          <Taken key={picture.url} icon={ICONS.image} name={picture.name} remove={() => dropPicture(picture, own)} />
        ),
      )}
      {attached.map((file) =>
        file.kind === "text" && file.text !== undefined ? (
          <Taken key={file.path} icon={ICONS.clipboard} name={t.pastedText(lineCount(file.text))} remove={() => dropText(file.path)} />
        ) : (
          <Taken key={file.path} icon={file.kind === "folder" ? ICONS.folderSmall : ICONS.fileText} name={file.name} remove={() => dropFile(file.path, own)} />
        ),
      )}
    </div>
  );
}
