import { test } from "node:test";
import assert from "node:assert/strict";

import { listing } from "../src/attachments.ts";
import { quotaLine } from "../src/storage.ts";

test("an empty list says nothing", () => {
  assert.equal(listing([]), "");
});

test("attachments keep their order and start with their name", () => {
  const lines = listing([
    { name: "a.txt", bytes: 1, addedAt: new Date() },
    { name: "b.txt", bytes: 2, addedAt: new Date() },
  ]).split("\n");
  assert.equal(lines.length, 2);
  assert.ok(lines[0].startsWith("a.txt"));
  assert.ok(lines[1].startsWith("b.txt"));
});

test("the quota reads in readable sizes", () => {
  const line = quotaLine({
    usedBytes: 1536,
    limitBytes: 1048576,
    renewsAt: new Date(2026, 0, 31),
  });
  assert.equal(line, "1.5 KB de 1 MB · se renueva el 2026-01-31");
});
