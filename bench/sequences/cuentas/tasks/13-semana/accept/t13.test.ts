import { expect, it } from "vitest";
import { run } from "../../src/cli";
import { memoryIo } from "../../src/io";

const GASTOS = `fecha,importe,categoria,descripcion
2026-02-27,30.00,Ocio,Cine
2026-03-01,12.50,Comida,Pan y fruta
2026-03-02,45.20,Transporte,Gasolina
2026-03-09,8.75,Comida,Café con Ana
2026-03-10,60.00,Ocio,Concierto
2026-03-15,23.40,Comida,Supermercado
2026-04-02,15.00,Transporte,Metro
`;

const said = (argv: string[], files: Record<string, string> = { "gastos.csv": GASTOS }) => {
  const io = memoryIo({ ...files });
  const ran = run(argv, io);
  return { code: ran.code, out: ran.output.replace(/\u00a0/g, " "), files: io.files };
};

it("adds up the week day by day, from Monday to Sunday", () => {
  expect(said(["semana", "gastos.csv", "2026-03-10"])).toMatchObject({
    code: 0,
    out: "Lunes 09/03/2026: 8,75 €\nMartes 10/03/2026: 60,00 €\nDomingo 15/03/2026: 23,40 €\nTotal: 92,15 €",
  });
  expect(said(["semana", "gastos.csv", "2026-03-01"]).out).toBe("Viernes 27/02/2026: 30,00 €\nDomingo 01/03/2026: 12,50 €\nTotal: 42,50 €");
  expect(said(["semana", "gastos.csv", "2026-03-20"]).out).toBe("Total: 0,00 €");
});

it("puts together what was spent on the same day", () => {
  const files = { "g.csv": "fecha,importe,categoria,descripcion\n2026-03-04,1.10,Comida,Pan\n2026-03-04,2.00,Comida,Leche\n" };
  expect(said(["semana", "g.csv", "2026-03-04"], files).out).toBe("Miércoles 04/03/2026: 3,10 €\nTotal: 3,10 €");
});

it("refuses a day that does not exist", () => {
  expect(said(["semana", "gastos.csv", "2026-02-30"])).toMatchObject({ code: 1, out: "Fecha no válida: 2026-02-30" });
});
