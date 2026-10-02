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

it("takes amounts written with a comma", () => {
  expect(said(["añadir", "gastos.csv", "2026-03-19", "4,20", "Comida", "Churros"]).files["gastos.csv"]).toBe(GASTOS + "2026-03-19,4.20,Comida,Churros\n");
  expect(said(["añadir", "gastos.csv", "2026-03-19", "1.234,56", "Casa", "Sofá"]).files["gastos.csv"]).toBe(GASTOS + "2026-03-19,1234.56,Casa,Sofá\n");
  expect(said(["añadir", "gastos.csv", "2026-03-19", "7", "Ocio", "Cine"]).files["gastos.csv"]).toBe(GASTOS + "2026-03-19,7.00,Ocio,Cine\n");
  expect(said(["presupuesto", "p.csv", "Ocio", "100,5"], {}).files["p.csv"]).toBe("categoria,importe\nOcio,100.50\n");
});

it("still refuses what is not an amount", () => {
  for (const amount of ["4,2,1", "12,345", "1.23,45", "1.234", ",5", "-4,20"]) {
    expect(said(["añadir", "gastos.csv", "2026-03-19", amount, "Comida", "Churros"])).toMatchObject({ code: 1, out: `Importe no válido: ${amount}` });
  }
});
