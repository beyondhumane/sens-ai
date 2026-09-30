import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import assert from "node:assert/strict";

const program = process.platform === "win32" ? "target/debug/report.exe" : "target/debug/report";
const report = (...args) => spawnSync(program, args, { encoding: "utf8" });

writeFileSync("data.txt", "10\n12\n20\n");

const loud = report("data.txt");
assert.equal(loud.status, 0, loud.stderr);
assert.equal(loud.stdout.trim(), "3 líneas · total 42 · media 14");

for (const args of [["data.txt", "--quiet"], ["--quiet", "data.txt"]]) {
  const quiet = report(...args);
  assert.equal(quiet.status, 0, quiet.stderr);
  assert.equal(quiet.stdout, "");
}

rmSync("out.csv", { force: true });
const written = report("data.txt", "--quiet", "--output", "out.csv");
assert.equal(written.status, 0, written.stderr);
assert.equal(written.stdout, "");
assert.equal(readFileSync("out.csv", "utf8").trim().split(/\r?\n/)[1], "3,42,14");

rmSync("../escape.csv", { force: true });
const escaping = report("data.txt", "--quiet", "--output", "../escape.csv");
assert.notEqual(escaping.status, 0);
assert.equal(existsSync("../escape.csv"), false);

const broken = report("missing.txt", "--quiet");
assert.notEqual(broken.status, 0);
assert.match(broken.stderr, /missing\.txt/);
