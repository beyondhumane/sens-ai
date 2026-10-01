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

const DUPS = "fecha,importe,categoria,descripcion\n2026-03-09,8.75,Comida,Café con Ana\n2026-03-09,8.75,Ocio,cafe  con ana\n2026-03-10,8.75,Comida,Café con Ana\n2026-03-11,3.00,Comida,Pan\n2026-03-11,3.00,Comida,PAN \n2026-03-11,3.00,Comida,pan\n";

it("finds expenses that look repeated", () => {
  expect(said(["duplicados", "d.csv"], { "d.csv": DUPS })).toMatchObject({ code: 0, out: "2 veces: 09/03/2026  8,75 €  Café con Ana\n3 veces: 11/03/2026  3,00 €  Pan" });
});

it("says when there are none", () => {
  expect(said(["duplicados", "gastos.csv"])).toMatchObject({ code: 0, out: "Sin duplicados" });
});
