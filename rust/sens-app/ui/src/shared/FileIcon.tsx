import { fileIcon } from "./fileIcons";

export function FileIcon({ path }: { path: string }) {
  return (
    <span className="glyph">
      <img src={fileIcon(path)} alt="" draggable={false} />
    </span>
  );
}
