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

it("compares each category with its budget", () => {
  const files = { "gastos.csv": GASTOS, "p.csv": "categoria,importe\ncomida,100.00\nOcio,55.00\nTransporte,45.20\n" };
  expect(said(["informe", "gastos.csv", "--mes", "2026-03", "--presupuestos", "p.csv"], files)).toMatchObject({
    code: 0,
    out: "Marzo de 2026\n  Ocio: 60,00 € de 55,00 € (te pasas 5,00 €)\n  Transporte: 45,20 € de 45,20 € (quedan 0,00 €)\n  Comida: 44,65 € de 100,00 € (quedan 55,35 €)\nTotal: 149,85 €",
  });
});

it("leaves categories without a budget as they were", () => {
  const files = { "gastos.csv": GASTOS, "p.csv": "categoria,importe\nOcio,100.00\n" };
  expect(said(["informe", "gastos.csv", "--mes", "2026-03", "--presupuestos", "p.csv"], files).out).toBe(
    "Marzo de 2026\n  Ocio: 60,00 € de 100,00 € (quedan 40,00 €)\n  Transporte: 45,20 €\n  Comida: 44,65 €\nTotal: 149,85 €",
  );
});
