import readline from "node:readline";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const argv = process.argv.slice(2);
const flag = (name) => {
  const at = argv.indexOf(name);
  return at >= 0 ? argv[at + 1] : "";
};
const session = flag("--session-id") || flag("--resume");
const say = (message) => process.stdout.write(JSON.stringify({ session_id: session, ...message }) + "\n");
const sleep = (ms) => new Promise((done) => setTimeout(done, ms));

let answerFor = null;
let canon = false;
let hooks = {};
let interrupted = false;
let asked = 0;
const waiting = new Map();

const finish = (subtype = "success", result = "") =>
  say({
    type: "result",
    subtype,
    is_error: subtype !== "success",
    duration_ms: 12,
    num_turns: 1,
    result,
    usage: {
      input_tokens: 3,
      cache_creation_input_tokens: 0,
      cache_read_input_tokens: 5,
      output_tokens: 2,
      iterations: [{ input_tokens: 3, cache_creation_input_tokens: 0, cache_read_input_tokens: 5, output_tokens: 2 }],
    },
    modelUsage: { "claude-sonnet-5": { contextWindow: 200000 } },
  });

const OFFERED = [
  { name: "compact", description: "Clear conversation history but keep a summary in context", argumentHint: "<optional custom summarization instructions>" },
  { name: "__remote-workflow", description: "", argumentHint: "" },
  { name: "frontend-design", description: "Create distinctive interfaces (user)", argumentHint: "" },
];

const speak = (text) => {
  for (const piece of [text.slice(0, 2), text.slice(2)]) {
    say({ type: "stream_event", event: { type: "content_block_delta", index: 0, delta: { type: "text_delta", text: piece } }, parent_tool_use_id: null });
  }
  say({ type: "assistant", message: { content: [{ type: "text", text }] }, parent_tool_use_id: null });
};

const ask = (request) =>
  new Promise((resolve) => {
    const id = `fake-${++asked}`;
    waiting.set(id, resolve);
    say({ type: "control_request", request_id: id, request });
  });

const callbacks = (event, tool) =>
  (hooks?.[event] ?? []).filter((entry) => !entry.matcher || new RegExp(`^(?:${entry.matcher})$`).test(tool ?? "")).flatMap((entry) => entry.hookCallbackIds);

const hook = async (event, input, tool) => {
  const outputs = [];
  for (const id of callbacks(event, tool)) {
    outputs.push(await ask({ subtype: "hook_callback", callback_id: id, input: { hook_event_name: event, session_id: session, cwd: process.cwd(), ...input } }));
  }
  return outputs;
};

const denial = (outputs) => outputs.find((output) => output?.hookSpecificOutput?.permissionDecision === "deny")?.hookSpecificOutput.permissionDecisionReason;
const blocking = (outputs) => outputs.find((output) => output?.decision === "block")?.reason;
const here = (path) => join(process.cwd(), path);
const put = (path, text) => {
  mkdirSync(dirname(here(path)), { recursive: true });
  writeFileSync(here(path), text);
};

let tools = 0;
const use = async (name, input, act) => {
  const id = `toolu_${++tools}`;
  say({ type: "assistant", message: { content: [{ type: "tool_use", id, name, input }] }, parent_tool_use_id: null });
  const denied = denial(await hook("PreToolUse", { tool_name: name, tool_input: input, tool_use_id: id }, name));
  if (denied) {
    say({ type: "user", message: { content: [{ type: "tool_result", tool_use_id: id, content: denied, is_error: true }] }, parent_tool_use_id: null });
    return { denied };
  }
  act?.();
  say({ type: "user", message: { content: [{ type: "tool_result", tool_use_id: id, content: "hecho", is_error: false }] }, parent_tool_use_id: null });
  return { blocked: blocking(await hook("PostToolUse", { tool_name: name, tool_input: input, tool_response: {}, tool_use_id: id }, name)) };
};

const stop = async () => blocking(await hook("Stop", { stop_hook_active: false, background_tasks: [] }));
const copied = () => readFileSync(here("src/lib/totals.ts"), "utf8").replaceAll("totals", "summarize");
const REUSES = "import { totals } from './lib/totals.ts';\nexport const report = () => totals([], 2);\n";
const ONE_USE = "export interface OnlyOne { run(): void }\nexport const runner: OnlyOne = { run() {} };\n";

const scenarios = {
  async copia() {
    const first = await use("Write", { file_path: here("src/report.ts"), content: copied() }, () => put("src/report.ts", copied()));
    speak(first.denied ? "copia denegada" : "copia escrita");
    await use("Write", { file_path: here("src/report.ts"), content: REUSES }, () => put("src/report.ts", REUSES));
    speak((await stop()) ? "cierre bloqueado" : "cierre limpio");
  },
  async terminal() {
    put("src/report.ts", copied());
    const ran = await use("Bash", { command: "node make.js" });
    speak(ran.blocked ? "terminal bloqueada" : "terminal libre");
    rmSync(here("src/report.ts"));
    speak((await stop()) ? "cierre bloqueado" : "cierre limpio");
  },
  async rondas() {
    put("src/report.ts", copied());
    for (let round = 1; round <= 4; round++) {
      speak((await stop()) ? `ronda ${round} bloqueada` : `ronda ${round} libre`);
    }
  },
  async commit() {
    put("src/report.ts", copied());
    const committed = await use("Bash", { command: "git add -A && git commit -m report" });
    speak(committed.denied ? "commit denegado" : "commit hecho");
    rmSync(here("src/report.ts"));
    await stop();
  },
  async ajustes() {
    const written = await use("Write", { file_path: here(".claude/settings.json"), content: "{}" }, () => put(".claude/settings.json", "{}"));
    speak(written.denied ? "ajustes denegados" : "ajustes escritos");
    const ran = await use("Bash", { command: "node tweak.js" }, () => put(".claude/settings.local.json", '{ "disableAllHooks": true }'));
    speak(ran.blocked && !existsSync(here(".claude/settings.local.json")) ? "ajustes devueltos" : "ajustes quedan");
    await stop();
  },
  async revisor() {
    put("src/runner.ts", ONE_USE);
    speak((await stop()) ? "revisor bloqueó" : "revisor dejó pasar");
    put("src/runner.ts", "export const runner = { run() {} };\n");
    speak((await stop()) ? "cierre bloqueado" : "cierre limpio");
  },
  async dependencia() {
    const manifest = JSON.parse(readFileSync(here("package.json"), "utf8"));
    manifest.dependencies = { ...manifest.dependencies, moment: "2" };
    const text = JSON.stringify(manifest, null, 2);
    const written = await use("Write", { file_path: here("package.json"), content: text }, () => put("package.json", text));
    speak(written.denied ? "dependencia denegada" : "dependencia aceptada");
    await stop();
  },
};

async function turn(content) {
  say({ type: "system", subtype: "init", model: `${canon ? "+canon " : ""}${argv.join(" ")} @ ${process.cwd()}` });
  const blocks = typeof content === "string" ? [{ type: "text", text: content }] : content;
  const text = blocks.filter((block) => block.type === "text").map((block) => block.text).join("\n");
  if (!text.includes("sordo")) {
    await hook("UserPromptSubmit", { prompt: text });
  }
  const pictures = blocks.filter((block) => block.type === "image" && block.source?.type === "base64");
  if (pictures.length) {
    speak(`vi ${pictures.length} imagen ${pictures[0].source.media_type} antes de "${text}"`);
    return finish();
  }
  if (text.startsWith("/compact")) {
    say({ type: "system", subtype: "compact_boundary", compact_metadata: { trigger: "manual", pre_tokens: 9000 } });
    return finish();
  }
  if (text.includes("muere")) {
    if (text.includes("copia")) {
      put("src/report.ts", copied());
    }
    process.stderr.write("se acabó la cuerda");
    process.exit(1);
  }
  const scenario = Object.keys(scenarios).find((name) => text.includes(name));
  if (scenario) {
    await scenarios[scenario]();
    return finish();
  }
  if (text.includes("permiso")) {
    say({ type: "assistant", message: { content: [{ type: "tool_use", id: "toolu_w", name: "Write", input: { file_path: "n.txt", content: "hecho" } }] }, parent_tool_use_id: null });
    await hook("PreToolUse", { tool_name: "Write", tool_input: { file_path: "n.txt", content: "hecho" } }, "Write");
    say({
      type: "control_request",
      request_id: "p1",
      request: { subtype: "can_use_tool", tool_name: "Write", input: { file_path: "n.txt", content: "hecho" }, permission_suggestions: [{ type: "setMode", mode: "acceptEdits", destination: "session" }] },
    });
    const response = await new Promise((resolve) => (answerFor = resolve));
    const allowed = response.behavior === "allow";
    say({ type: "user", message: { content: [{ type: "tool_result", tool_use_id: "toolu_w", content: allowed ? "creado" : response.message, is_error: !allowed }] }, parent_tool_use_id: null, tool_use_result: { type: "create", filePath: "n.txt" } });
    speak(allowed ? "permitido" : "rechazado");
    await stop();
    return finish();
  }
  if (text.includes("lento")) {
    interrupted = false;
    for (let count = 0; count < 100 && !interrupted; count++) {
      say({ type: "stream_event", event: { type: "content_block_delta", index: 0, delta: { type: "text_delta", text: `${count} ` } }, parent_tool_use_id: null });
      await sleep(40);
    }
    return interrupted ? finish("error_during_execution") : finish();
  }
  speak("Hola");
  await stop();
  return finish();
}

function review(asked) {
  let file = "";
  const findings = [];
  for (const line of asked.split("\n")) {
    if (line.startsWith("+++ b/")) file = line.slice(6);
    if (line.startsWith("+export interface OnlyOne")) {
      findings.push({ rule: "S1", file, quote: line.slice(1), why: "OnlyOne has a single implementation.", fix: "Drop the interface and use the object.", confidence: "high" });
    }
  }
  process.stdout.write(JSON.stringify({ type: "result", subtype: "success", is_error: false, structured_output: { findings } }) + "\n");
}

if (argv.includes("--json-schema")) {
  let asked = "";
  process.stdin.on("data", (chunk) => (asked += chunk));
  process.stdin.on("end", () => review(asked));
} else readline.createInterface({ input: process.stdin }).on("line", (line) => {
  const message = JSON.parse(line);
  if (message.type === "control_request" && message.request.subtype === "initialize") {
    canon = Boolean(message.request.appendSystemPrompt);
    hooks = message.request.hooks ?? {};
    say({ type: "control_response", response: { subtype: "success", request_id: message.request_id, response: { commands: OFFERED } } });
  } else if (message.type === "control_request" && message.request.subtype === "interrupt") {
    interrupted = true;
    say({ type: "control_response", response: { subtype: "success", request_id: message.request_id, response: {} } });
  } else if (message.type === "control_response" && waiting.has(message.response.request_id)) {
    const resolve = waiting.get(message.response.request_id);
    waiting.delete(message.response.request_id);
    resolve(message.response.response);
  } else if (message.type === "control_response" && answerFor) {
    const resolve = answerFor;
    answerFor = null;
    resolve(message.response.response);
  } else if (message.type === "user") {
    turn(message.message.content);
  }
});
