import table from "./file-icons.json";

const names: Record<string, string> = table.names;
const extensions: Record<string, string> = table.extensions;

export function fileIcon(path: string) {
  return `/file-icons/${byName(path, names, extensions) ?? table.file}.svg`;
}

export const knownFile = (path: string) => byName(path, names, extensions) !== undefined;

export function byName<Value>(path: string, names: Record<string, Value>, extensions: Record<string, Value>) {
  const name = (path.split(/[/\\]/).pop() || path).toLowerCase();
  if (Object.hasOwn(names, name)) return names[name];
  for (let dot = name.indexOf("."); dot !== -1; dot = name.indexOf(".", dot + 1)) {
    const extension = name.slice(dot + 1);
    if (Object.hasOwn(extensions, extension)) return extensions[extension];
  }
  return undefined;
}
