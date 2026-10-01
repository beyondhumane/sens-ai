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

it("appends an expense to the file", () => {
  const { code, out, files } = said(["añadir", "gastos.csv", "2026-03-19", "4.20", "Comida", "Churros"]);
  expect(code).toBe(0);
  expect(out).toBe("Añadido: 19/03/2026  Comida  4,20 €  Churros");
  expect(files["gastos.csv"]).toBe(GASTOS + "2026-03-19,4.20,Comida,Churros\n");
  expect(said(["lista", "gastos.csv"], files).out.split("\n")).toContain("19/03/2026  Comida  4,20 €  Churros");
  expect(said(["añadir", "gastos.csv", "2026-03-18", "3", "Ocio", "Revista"]).files["gastos.csv"]).toContain("2026-03-18,3.00,Ocio,Revista\n");
});

it("refuses a date or an amount that does not make sense and leaves the file alone", () => {
  for (const [argv, said_] of [
    [["2026-02-30", "4.20"], "Fecha no válida: 2026-02-30"],
    [["19/03/2026", "4.20"], "Fecha no válida: 19/03/2026"],
    [["2026-03-19", "-3"], "Importe no válido: -3"],
    [["2026-03-19", "0"], "Importe no válido: 0"],
    [["2026-03-19", "1.234"], "Importe no válido: 1.234"],
    [["2026-03-19", "doce"], "Importe no válido: doce"],
  ] as const) {
    const { code, out, files } = said(["añadir", "gastos.csv", ...argv, "Comida", "Churros"]);
    expect({ code, out }).toEqual({ code: 1, out: said_ });
    expect(files["gastos.csv"]).toBe(GASTOS);
  }
});
