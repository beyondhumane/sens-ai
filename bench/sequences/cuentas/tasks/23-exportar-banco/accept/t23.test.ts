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

it("exports in the bank's format", () => {
  const { code, out, files } = said(["exportar", "gastos.csv", "b.csv", "--mes", "2026-03", "--formato", "banco"]);
  expect({ code, out }).toEqual({ code: 0, out: "Exportados 5 gastos a b.csv" });
  expect(files["b.csv"]).toBe(
    "Fecha;Concepto;Importe\n01/03/2026;Pan y fruta;-12,50\n02/03/2026;Gasolina;-45,20\n09/03/2026;Café con Ana;-8,75\n10/03/2026;Concierto;-60,00\n15/03/2026;Supermercado;-23,40\n",
  );
  const back = said(["importar", "b.csv", "n.csv"], { "b.csv": files["b.csv"], "n.csv": "fecha,importe,categoria,descripcion\n" });
  expect(back.out).toBe("Importados 5 gastos");
  expect(said(["total", "n.csv"], back.files).out).toBe("Total: 149,85 €");
});

it("refuses a format it does not know", () => {
  expect(said(["exportar", "gastos.csv", "b.pdf", "--formato", "pdf"])).toMatchObject({ code: 1, out: "Formato no válido: pdf" });
});
