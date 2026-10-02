import { existsSync, readFileSync, writeFileSync } from "node:fs";

export interface Io {
  read(path: string): string;
  write(path: string, text: string): void;
  exists(path: string): boolean;
  today: string;
}

export function memoryIo(files: Record<string, string>, today = "2026-03-20"): Io & { files: Record<string, string> } {
  return {
    files,
    read(path) {
      if (!(path in files)) throw new Error(`No existe ${path}`);
      return files[path];
    },
    write(path, text) {
      files[path] = text;
    },
    exists(path) {
      return path in files;
    },
    today,
  };
}

export function diskIo(): Io {
  return {
    read: (path) => readFileSync(path, "utf8"),
    write: (path, text) => writeFileSync(path, text, "utf8"),
    exists: (path) => existsSync(path),
    today: new Date().toISOString().slice(0, 10),
  };
}
