import type { Expense } from "./expense";

export const HEADER = "fecha,importe,categoria,descripcion";

export function parseExpenses(text: string): Expense[] {
  const [header, ...rows] = text.trim().split(/\r?\n/);
  if (header !== HEADER) throw new Error(`Cabecera desconocida: ${header}`);
  return rows
    .filter((row) => row.trim() !== "")
    .map((row) => {
      const [date, amount, category, ...rest] = row.split(",");
      return { date, cents: Math.round(Number(amount) * 100), category, description: rest.join(",") };
    });
}
