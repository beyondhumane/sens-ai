import { test } from "node:test";
import assert from "node:assert/strict";

import { listing } from "../src/attachments.ts";

const HOUR = 3_600_000;

test("each attachment shows its name, readable size and how long ago it was added", () => {
  const now = Date.now();
  const text = listing([
    { name: "informe.pdf", bytes: 1_572_864, addedAt: new Date(now - 3 * HOUR) },
    { name: "nota.txt", bytes: 512, addedAt: new Date(now - 50 * HOUR) },
  ]);
  assert.equal(text, "informe.pdf · 1.5 MB · hace 3 horas\nnota.txt · 512 B · hace 2 días");
});

test("an empty list still says nothing", () => {
  assert.equal(listing([]), "");
});
