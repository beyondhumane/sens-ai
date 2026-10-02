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

it("finds by category too", () => {
  expect(said(["buscar", "gastos.csv", "transporte"]).out).toBe("02/03/2026  Transporte  45,20 €  Gasolina\n02/04/2026  Transporte  15,00 €  Metro");
  const files = { "g.csv": "fecha,importe,categoria,descripcion\n2026-03-05,30.00,Educación,Libros\n" };
  expect(said(["buscar", "g.csv", "educacion"], files).out).toBe("05/03/2026  Educación  30,00 €  Libros");
});
