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

const MESSY = "fecha,importe,categoria,descripcion\n2026-03-10,5.00,Ocio,B\n2026-03-01,7.00,Comida,A\n2026-03-10,9.00,Ocio,C\n";

it("lists by date, oldest first, keeping the file's order within a day", () => {
  expect(said(["lista", "m.csv"], { "m.csv": MESSY })).toMatchObject({ code: 0, out: "01/03/2026  Comida  7,00 €  A\n10/03/2026  Ocio  5,00 €  B\n10/03/2026  Ocio  9,00 €  C" });
});

it("lists the biggest first when asked", () => {
  expect(said(["lista", "m.csv", "--orden", "importe"], { "m.csv": MESSY }).out).toBe("10/03/2026  Ocio  9,00 €  C\n01/03/2026  Comida  7,00 €  A\n10/03/2026  Ocio  5,00 €  B");
  expect(said(["lista", "gastos.csv", "--orden", "importe", "--hasta", "2026-03-01"]).out).toBe("27/02/2026  Ocio  30,00 €  Cine\n01/03/2026  Comida  12,50 €  Pan y fruta");
  expect(said(["lista", "m.csv", "--orden", "color"], { "m.csv": MESSY })).toMatchObject({ code: 1, out: "Orden no válido: color" });
});
