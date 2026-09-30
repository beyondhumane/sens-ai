export interface Chapter {
  n: number;
  title: string;
  text: string;
  replay: string;
}

export const chapters: Chapter[] = [
  {
    n: 1,
    title: "You ask.",
    text: "Type it, or say it. The request stays at the top of the thread, with the project and branch it works in.",
    replay: "In the replay, the request sits at the top of the thread, with the orbit folder and the main branch above the message box.",
  },
  {
    n: 2,
    title: "Sens shows the work.",
    text: "Every tool call is a step, in order: what Claude thought, read, searched, ran and edited, with how long each took.",
    replay: "In the replay, the line Worked opens into nine steps: three thoughts, a read, a search, two test runs and two edits. The first test run comes into focus.",
  },
  {
    n: 3,
    title: "Open any step.",
    text: "A command opens to its output, in its own program's colours. A file and line open the file.",
    replay: "In the replay, the step Run npm test opens to its output: 2 test files and 10 tests passed.",
  },
  {
    n: 4,
    title: "See what changed.",
    text: "Each edit lands in Changes as a diff, file by file, against the last commit.",
    replay: "In the replay, the path src/header.tsx travels from its Edit step to Changes, where header.tsx shows 8 lines added and 2 removed.",
  },
  {
    n: 5,
    title: "Read the result.",
    text: "When the reply is done, the work folds into one line that opens to all of it. A failure is never folded away.",
    replay: "In the replay, the steps fold back into one line and the answer comes forward: the header takes an optional greeting, and the suite passes with 12 tests.",
  },
];

export const numberOf = (n: number): string => String(n).padStart(2, "0");
