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

it("lists a range of days, both ends included", () => {
  expect(said(["lista", "gastos.csv", "--desde", "2026-03-02", "--hasta", "2026-03-10"])).toMatchObject({
    code: 0,
    out: "02/03/2026  Transporte  45,20 €  Gasolina\n09/03/2026  Comida  8,75 €  Café con Ana\n10/03/2026  Ocio  60,00 €  Concierto",
  });
  expect(said(["lista", "gastos.csv", "--hasta", "2026-02-28"]).out).toBe("27/02/2026  Ocio  30,00 €  Cine");
  expect(said(["lista", "gastos.csv", "--desde", "2026-03-15"]).out).toBe("15/03/2026  Comida  23,40 €  Supermercado\n02/04/2026  Transporte  15,00 €  Metro");
});

it("refuses a day that does not exist", () => {
  expect(said(["lista", "gastos.csv", "--desde", "2026-03-32"])).toMatchObject({ code: 1, out: "Fecha no válida: 2026-03-32" });
  expect(said(["lista", "gastos.csv", "--hasta", "15/03/2026"])).toMatchObject({ code: 1, out: "Fecha no válida: 15/03/2026" });
});
