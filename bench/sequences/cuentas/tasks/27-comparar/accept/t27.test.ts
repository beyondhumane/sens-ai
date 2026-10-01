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

it("compares two months category by category", () => {
  expect(said(["comparar", "gastos.csv", "2026-02", "2026-03"])).toMatchObject({
    code: 0,
    out: "Comida: 0,00 € → 44,65 € (+44,65 €)\nOcio: 30,00 € → 60,00 € (+30,00 €)\nTransporte: 0,00 € → 45,20 € (+45,20 €)",
  });
  expect(said(["comparar", "gastos.csv", "2026-03", "2026-04"]).out).toBe(
    "Comida: 44,65 € → 0,00 € (-44,65 €)\nOcio: 60,00 € → 0,00 € (-60,00 €)\nTransporte: 45,20 € → 15,00 € (-30,20 €)",
  );
  const same = { "g.csv": "fecha,importe,categoria,descripcion\n2026-01-05,10.00,Casa,Luz\n2026-02-05,10.00,Casa,Luz\n" };
  expect(said(["comparar", "g.csv", "2026-01", "2026-02"], same).out).toBe("Casa: 10,00 € → 10,00 € (+0,00 €)");
  expect(said(["comparar", "gastos.csv", "2026-02", "2026-14"])).toMatchObject({ code: 1, out: "Mes no válido: 2026-14" });
});
