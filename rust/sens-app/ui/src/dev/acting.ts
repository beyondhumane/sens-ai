import type { Mock } from "vitest";
import { perform } from "../app/acts";

export const asker = (answered: Mock) => async (act: string, input: Record<string, unknown>) => {
  await perform({ ask: 1, act, input });
  const [, ok, text] = answered.mock.calls.at(-1)!;
  return { ok, text };
};
