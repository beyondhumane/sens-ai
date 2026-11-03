import { createStore } from "zustand/vanilla";

export interface Edits {
  add: Set<number>;
  plus: number;
  minus: number;
}

export type View = "" | "capabilities" | "artifacts" | "news";

export const project = createStore(() => ({
  root: "",
  work: "",
  session: "",
  view: "" as View,
  touched: new Map<string, Edits>(),
}));

export function noteEdit({ path, lines, plus, minus }: { path: string; lines: number[]; plus: number; minus: number }) {
  project.setState(({ touched }) => {
    const known = touched.get(path);
    const edits = {
      add: new Set([...(known?.add ?? []), ...lines]),
      plus: (known?.plus ?? 0) + plus,
      minus: (known?.minus ?? 0) + minus,
    };
    return { touched: new Map(touched).set(path, edits) };
  });
}

export const forgetEdits = () => project.setState({ touched: new Map() });
