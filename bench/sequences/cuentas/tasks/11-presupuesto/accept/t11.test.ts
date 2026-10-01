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

it("keeps a monthly budget per category", () => {
  const first = said(["presupuesto", "p.csv", "Comida", "100"], {});
  expect({ code: first.code, out: first.out }).toEqual({ code: 0, out: "Presupuesto de Comida: 100,00 € al mes" });
  expect(first.files["p.csv"]).toBe("categoria,importe\nComida,100.00\n");
  const second = said(["presupuesto", "p.csv", "Ocio", "55.5"], first.files);
  expect(second.files["p.csv"]).toBe("categoria,importe\nComida,100.00\nOcio,55.50\n");
  const third = said(["presupuesto", "p.csv", "comida", "120"], second.files);
  expect(third.files["p.csv"]).toBe("categoria,importe\ncomida,120.00\nOcio,55.50\n");
  expect(third.out).toBe("Presupuesto de comida: 120,00 € al mes");
});

it("refuses an amount that does not make sense", () => {
  expect(said(["presupuesto", "p.csv", "Comida", "-5"], {})).toMatchObject({ code: 1, out: "Importe no válido: -5" });
});
