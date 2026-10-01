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

it("refuses expenses in the future", () => {
  const { code, out, files } = said(["añadir", "gastos.csv", "2026-03-21", "4.20", "Comida", "Churros"]);
  expect({ code, out }).toEqual({ code: 1, out: "La fecha 2026-03-21 es futura" });
  expect(files["gastos.csv"]).toBe(GASTOS);
  expect(said(["añadir", "gastos.csv", "2027-01-01", "4.20", "Comida", "Churros"]).out).toBe("La fecha 2027-01-01 es futura");
});

it("takes today", () => {
  expect(said(["añadir", "gastos.csv", "2026-03-20", "4.20", "Comida", "Churros"])).toMatchObject({ code: 0, out: "Añadido: 20/03/2026  Comida  4,20 €  Churros" });
});
