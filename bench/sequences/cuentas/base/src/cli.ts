import { parseExpenses } from "./csv";
import type { Expense } from "./expense";
import type { Io } from "./io";
import { formatMoney } from "./money";

export interface Ran {
  code: number;
  output: string;
}

type Command = (args: string[], io: Io) => string;

function load(path: string | undefined, io: Io): Expense[] {
  if (!path) throw new Error("Falta el fichero de gastos");
  return parseExpenses(io.read(path));
}

const commands: Record<string, Command> = {
  lista: (args, io) =>
    load(args[0], io)
      .map((expense) => [expense.date, expense.category, formatMoney(expense.cents), expense.description].join("  "))
      .join("\n"),
  total: (args, io) => `Total: ${formatMoney(load(args[0], io).reduce((sum, expense) => sum + expense.cents, 0))}`,
};

export function run(argv: string[], io: Io): Ran {
  const [name, ...args] = argv;
  const command = name === undefined ? undefined : commands[name];
  if (!command) return { code: 1, output: `Orden desconocida: ${name ?? ""}`.trim() };
  try {
    return { code: 0, output: command(args, io) };
  } catch (error) {
    return { code: 1, output: error instanceof Error ? error.message : String(error) };
  }
}
