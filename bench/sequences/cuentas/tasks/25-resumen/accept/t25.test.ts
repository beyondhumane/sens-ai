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

it("sums up the whole file", () => {
  expect(said(["resumen", "gastos.csv"])).toMatchObject({
    code: 0,
    out: "Gastos: 7\nDesde 27/02/2026 hasta 02/04/2026\nTotal: 194,85 €\nMayor: 60,00 € en Concierto (10/03/2026)",
  });
});

it("finds the first and last day even when the file is out of order", () => {
  const files = { "g.csv": "fecha,importe,categoria,descripcion\n2026-03-10,5.00,Ocio,B\n2026-01-01,9.00,Ocio,A\n2026-02-01,9.00,Ocio,C\n" };
  expect(said(["resumen", "g.csv"], files).out).toBe("Gastos: 3\nDesde 01/01/2026 hasta 10/03/2026\nTotal: 23,00 €\nMayor: 9,00 € en A (01/01/2026)");
});

it("says when there is nothing", () => {
  expect(said(["resumen", "g.csv"], { "g.csv": "fecha,importe,categoria,descripcion\n" })).toMatchObject({ code: 0, out: "Sin gastos" });
});
