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

const BANCO = "Fecha;Concepto;Importe\n09/03/2026;Bar Paco;-8,75\n10/03/2026;Nómina;1.850,00\n12/03/2026;Cena, con tapas;-1.234,56\n";

it("imports the bank's expenses and leaves out its income", () => {
  const { code, out, files } = said(["importar", "banco.csv", "gastos.csv"], { "gastos.csv": GASTOS, "banco.csv": BANCO });
  expect({ code, out }).toEqual({ code: 0, out: "Importados 2 gastos" });
  expect(files["gastos.csv"]).toBe(GASTOS + '2026-03-09,8.75,Sin categoría,Bar Paco\n2026-03-12,1234.56,Sin categoría,"Cena, con tapas"\n');
});

it("imports nothing when a line of the bank makes no sense", () => {
  for (const [bank, wrong] of [
    ["Fecha;Concepto;Importe\n09/03/2026;Bar;-8,75\n31/02/2026;Cine;-7,00\n", 3],
    ["Fecha;Concepto;Importe\n09/03/2026;Bar;-8,7x\n", 2],
    ["Fecha;Concepto;Importe\n2026-03-09;Bar;-8,75\n", 2],
  ] as const) {
    const { code, out, files } = said(["importar", "banco.csv", "gastos.csv"], { "gastos.csv": GASTOS, "banco.csv": bank });
    expect({ code, out }).toEqual({ code: 1, out: `Línea ${wrong} del banco no válida` });
    expect(files["gastos.csv"]).toBe(GASTOS);
  }
});
