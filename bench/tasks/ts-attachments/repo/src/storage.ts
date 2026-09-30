import dayjs from "dayjs";

import { formatBytes } from "./lib/format.ts";
import type { Quota } from "./types.ts";

export function quotaLine(quota: Quota): string {
  const used = formatBytes(quota.usedBytes);
  const limit = formatBytes(quota.limitBytes);
  const renews = dayjs(quota.renewsAt).format("YYYY-MM-DD");
  return `${used} de ${limit} · se renueva el ${renews}`;
}
