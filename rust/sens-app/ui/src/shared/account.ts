import type { Account } from "../ipc/types";

export const PLANS: Record<string, string> = { pro: "Pro", max: "Max", team: "Team", enterprise: "Enterprise" };
export const API_KEY_SOURCE = "ANTHROPIC_API_KEY";

export const keyed = (found: Account | null | undefined) =>
  found?.billing === "elsewhere" && found.source === API_KEY_SOURCE;
