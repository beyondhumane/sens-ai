import type { Language } from "../shared/i18n";
import type { Look } from "../shared/look";

declare global {
  interface Window {
    __SENS_HIDDEN__?: boolean;
  }
}

export interface Profile {
  name: string;
  checkUpdates: boolean;
  welcomed: boolean;
  seen: string;
  notify: boolean;
  keepInTray?: boolean;
  startWithWindows?: boolean;
}

export interface Front {
  app: string;
  title: string;
}

export interface Copied {
  preview: string;
  chars: number;
}

export interface BarContext {
  front: Front | null;
  clip: Copied | null;
}

export interface Shot {
  mediaType: "image/png";
  data: string;
  width: number;
  height: number;
}

export interface BarProject {
  root: string;
  name: string;
}

export interface HandOver {
  root: string;
  session: string;
  text: string;
}

export interface BarOpened {
  look: Look;
  language: Language | null;
  front: Front | null;
  pinned: boolean;
}

export interface Watching {
  session: string | null;
}

export interface Shortcut {
  keys: string;
  taken: boolean;
}

export interface Release {
  version: string;
  notes: string;
  page: string;
  size: number;
}

export type UpdateStage = "downloading" | "verifying" | "installing";

export interface UpdateCheck {
  latest: Release | null;
  installable: boolean;
}

export interface News {
  version: string;
  title: string;
  notes: string;
  page: string;
  published: string;
}

export type Method = "subscription" | "console" | "apiKey";

export interface Account {
  billing: "subscription" | "noPlan" | "elsewhere" | "signedOut";
  plan: string;
  source: string;
  email: string;
}

export interface ProviderState {
  id: string;
  vendor: string;
  label: string;
  method: Method;
  keyHint: string;
  version: string;
  account: Account | null;
  error: string;
  installed: boolean;
}

export interface Skill {
  name: string;
  description: string;
  enabled: boolean;
}

export interface Plugin {
  name: string;
  description: string;
  version: string;
  enabled: boolean;
}

export interface Server {
  name: string;
  command: string;
  args: string[];
  envKeys: string[];
  kind: string;
  url: string;
  enabled: boolean;
}

export interface Provenance {
  listing: string;
  revision: string;
  version: string;
  installedAt: number;
}

export interface Capabilities {
  skills: Skill[];
  servers: Server[];
  plugins: Plugin[];
  origins: Record<string, Provenance>;
}

export interface NewServer {
  name: string;
  command: string;
  args: string[];
  env: Record<string, string>;
}

export interface Listing {
  id: string;
  kind: "plugin" | "skill" | "connector";
  name: string;
  title: string;
  description: string;
  author: string;
  badge: "anthropic" | "partner" | "community" | "skillsSh";
  source: string;
  category: string;
  version: string;
  homepage: string;
  installs: number | null;
  login: boolean;
  tools: string[];
  installable: boolean;
  revision: string;
}

export interface SourceState {
  id: string;
  fetchedAt: number;
  error: string;
}

export interface Market {
  listings: Listing[];
  sources: SourceState[];
}

export interface Part {
  name: string;
  path: string;
  description: string;
}

export interface Parts {
  skills: Part[];
  commands: Part[];
  agents: Part[];
  hooks: { event: string; command: string }[];
  servers: { name: string; launch: string }[];
  lsp: string[];
  bin: string[];
}

export interface Need {
  name: string;
  description: string;
  secret: boolean;
  required: boolean;
  default: string;
}

export interface Detail {
  listing: Listing;
  readme: string;
  license: string;
  files: { path: string; size: number }[];
  parts: Parts;
  needs: Need[];
}

export interface Artifact {
  kind: "image" | "file" | "link";
  root: string;
  project: string;
  name: string;
  target: string;
  session: string | null;
  sessionTitle: string | null;
  at: number;
  bytes: number | null;
}

export interface Entry {
  name: string;
  path: string;
  dir: boolean;
  ignored: boolean;
}

export type Opened =
  | { kind: "text"; text: string }
  | { kind: "picture"; data: string; bytes: number }
  | { kind: "tooBig"; bytes: number; cap: number }
  | { kind: "binary"; bytes: number };

export interface Changes {
  diff: string;
  fresh: string[];
}

export interface AgentEvent {
  kind: string;
  id?: string;
  name?: string;
  input?: Record<string, unknown>;
  output?: string;
  detail?: { backgroundTaskId?: string };
  runner?: string;
  description?: string;
  prompt?: string;
  tool?: string;
  status?: string;
  summary?: string;
  doing?: string;
  last?: string;
  tools?: number;
  tokens?: number;
  millis?: number;
}

export interface ClaudeCodeProgress {
  stage: "downloading" | "verifying" | "installing" | "updating";
  done: number;
  total: number;
}

export interface SessionSummary {
  id: string;
  title: string;
  startedAt: number;
  tasks: number;
  archived: boolean;
}

export interface Workspace {
  root: string;
  name: string;
  activeAt: number;
  sessions: SessionSummary[];
  trusted: boolean;
}

export interface Frame {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type Heard =
  | { kind: "loading" | "loaded"; url: string }
  | { kind: "titled"; title: string }
  | { kind: "said"; level: string; text: string };

export interface TerminalOpened {
  id: number;
  shell: string;
}

export interface TerminalReading {
  ask: number;
  terminal: number | null;
  lines: number;
  within: string[];
}

export type TerminalHeard = { kind: "out"; id: number; data: string } | { kind: "ended"; id: number; code: number | null };

export type VoiceCause = "microphone" | "model" | "other";

export interface VoiceRefusal {
  cause: VoiceCause;
  message: string;
}

export type VoiceHeard =
  | { kind: "level"; id: number; level: number }
  | { kind: "guess"; id: number; text: string }
  | { kind: "phrase"; id: number; text: string }
  | { kind: "ended"; id: number; refusal: VoiceRefusal | null }
  | { kind: "fetching"; done: number; total: number }
  | { kind: "ready" }
  | { kind: "unfetched"; message: string };

export interface VoiceModel {
  ready: boolean;
  fetching: boolean;
  bytes: number;
}

export interface Microphone {
  name: string;
  default: boolean;
}

export interface Todo {
  content: string;
  status: "pending" | "in_progress" | "completed";
  activeForm?: string;
}

export interface Question {
  question: string;
  header?: string;
  multiSelect?: boolean;
  options?: { label: string; description?: string }[];
}

export interface ToolInput {
  file_path?: string;
  notebook_path?: string;
  path?: string;
  command?: string;
  description?: string;
  pattern?: string;
  url?: string;
  query?: string;
  prompt?: string;
  todos?: Todo[];
  subagent_type?: string;
  skill?: string;
  old_string?: string;
  new_string?: string;
  content?: string;
  plan?: string;
  questions?: Question[];
  [field: string]: unknown;
}

export interface ToolDetail {
  stdout?: string;
  stderr?: string;
  structuredPatch?: { oldStart: number; newStart: number; lines: string[] }[];
  filePath?: string;
  type?: string;
  content?: string;
  file?: { numLines?: number };
  numFiles?: number;
  [field: string]: unknown;
}

export interface Link {
  url: string;
  title: string;
}

export interface Suggestion {
  type: string;
  mode?: string;
}

export type Answers = Record<string, string | string[]>;

export interface Asking {
  kind: "asking";
  request: string;
  tool: string;
  input: ToolInput;
  suggestions: Suggestion[] | null;
}

export interface Finished {
  kind: "finished";
  ok: boolean;
  stopped: boolean;
  millis: number;
  turns: number;
  tokensIn: number;
  tokensOut: number;
  context?: number;
  window?: number;
  error: string;
}

export interface LimitWindow {
  utilization: number | null;
  resetsAt: number | null;
}

export interface Overage {
  status: string;
  using: boolean;
  resetsAt: number | null;
  disabled: string;
}

export interface Limits {
  kind: "limits";
  status: string;
  window: string;
  utilization: number | null;
  resetsAt: number | null;
  threshold: number | null;
  overage: Overage | null;
  windows: Record<string, LimitWindow>;
}

export interface Slash {
  name: string;
  description: string;
  hint: string;
}

export type ChatEvent =
  | { kind: "started"; model: string }
  | { kind: "delta"; thinking: boolean; text: string }
  | { kind: "said"; text: string }
  | { kind: "thought"; text: string }
  | { kind: "tool"; id: string; name: string; input: ToolInput }
  | { kind: "toolDone"; id: string; output: string; error: boolean; detail: ToolDetail | null }
  | Asking
  | { kind: "answered"; request: string; allowed: boolean; answers: Answers | null }
  | Limits
  | { kind: "consulted"; tool: string; links: Link[] }
  | { kind: "taskStarted" | "taskProgress" | "taskEnded" }
  | { kind: "compacted"; before: number; auto: boolean }
  | Finished
  | { kind: "failed"; reason: string }
  | { kind: "lockedOut"; reason: Lockout };

export type Lockout = "signIn" | "billing";

export interface Isolation {
  path: string;
  branch: string;
  base: string;
}

export type SessionEntry =
  | { kind: "opened"; at: number; root: string }
  | ({ kind: "isolated"; at: number } & Isolation)
  | { kind: "task"; at: number; text: string; files: string[]; images: string[] }
  | { kind: "agent"; at: number; event: ChatEvent }
  | { kind: "titled"; at: number; title: string };

export interface Decision {
  allow: boolean;
  remember?: boolean;
  mode?: string;
  message?: string;
  answers?: Answers;
}

export interface Message {
  text: string;
  files: string[];
  images: { mediaType: string; data: string }[];
}

export interface Settings {
  provider: string;
  model: string;
  effort: string;
  thinking: boolean;
  mode: string;
}

export interface Provider {
  id: string;
  vendor: string;
  label: string;
}

export interface Card {
  id: string;
  label: string;
  description: string;
  latest: boolean;
  efforts: string[];
  effort: string;
  thinking: "always" | "toggle";
}

export type AttachedFile = { kind: "file"; path: string; name: string; bytes: number; outside: boolean };

export type Attached =
  | AttachedFile
  | { kind: "folder"; path: string; name: string; entries: number; outside: boolean }
  | { kind: "picture"; path: string; name: string; mediaType: string; data: string; bytes: number; outside: boolean };

export interface Refused {
  name: string;
  why: "missing" | "unreadable" | "tooBig" | "project";
}

export interface Attachments {
  items: Attached[];
  refused: Refused[];
}

export interface Repo {
  branch: string;
  detached: boolean;
  dirty: number;
  branches: string[];
}

export interface FoundProject {
  root: string;
  name: string;
  exists: boolean;
  sessions: number;
  already: number;
  last: number;
  suggested: boolean;
}

export interface ForeignServer {
  id: string;
  source: "claude-desktop" | "cursor" | "windsurf" | "vscode" | "codex";
  app: string;
  name: string;
  kind: "stdio" | "http" | "sse";
  command: string;
  args: string[];
  url: string;
  envKeys: string[];
  blocked: string;
}

export interface Found {
  claude: string;
  projects: FoundProject[];
  skills: string[];
  servers: string[];
  plugins: string[];
  foreign: ForeignServer[];
}

export interface Adopted {
  sessions: number;
  projects: number;
  skipped: { root: string; reason: string }[];
}

export interface Imported {
  added: string[];
  skipped: { name: string; reason: string }[];
}
