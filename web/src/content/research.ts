export const hooks = ["UserPromptSubmit", "PreToolUse · Write, Edit", "PreToolUse · Bash, PowerShell", "PostToolUse", "Stop · SubagentStop"];

export const changeCodes = ["R1", "R2", "R3", "R4", "R6", "R7", "R8"];

export const reviewCodes = ["S1", "S2", "S3", "S4", "S5", "S6", "S7"];

export const calibration = [
  ["32/36", "31/36", "67/72"],
  ["23/36", "36/36", "72/72"],
  ["0/3", "3/3", "6/6"],
  ["1/3", "1/3", "6/6"],
  ["0/3", "0/3", "3/6"],
];

export const testsWritten = [
  { runs: 18, arms: [14, 9, 2] },
  { runs: 36, arms: [23, 36, 36] },
];

export const blockCounts = ["1", "2", "0", "1", "3"];

export const reproduce = [
  "sens-bench validate --tasks bench/tasks",
  "sens-bench run --tasks bench/tasks --condition C0,C1,C2 --reps 3 --out bench/results/<batch>",
  "sens-bench sequence validate bench/sequences/cuentas",
  "sens-bench sequence run bench/sequences/cuentas --condition C0,C2 --reps 3 --out bench/results/<batch>",
  "sens-bench sequence report bench/results/<batch>",
];
