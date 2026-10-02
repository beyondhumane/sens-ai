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

it("adds up each category, the biggest first", () => {
  const { code, out } = said(["categorias", "gastos.csv"]);
  expect(code).toBe(0);
  expect(out).toBe("Ocio: 90,00 €\nTransporte: 60,20 €\nComida: 44,65 €");
});

it("breaks ties by name", () => {
  const tied = "fecha,importe,categoria,descripcion\n2026-03-01,10.00,Ropa,Camisa\n2026-03-02,10.00,Casa,Bombillas\n";
  expect(said(["categorias", "g.csv"], { "g.csv": tied }).out).toBe("Casa: 10,00 €\nRopa: 10,00 €");
});
