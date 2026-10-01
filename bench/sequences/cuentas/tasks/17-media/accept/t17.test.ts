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

it("divides the month by all its days", () => {
  expect(said(["media", "gastos.csv", "--mes", "2026-03"])).toMatchObject({ code: 0, out: "Media diaria en marzo de 2026: 4,83 €" });
  expect(said(["media", "gastos.csv", "--mes", "2026-02"]).out).toBe("Media diaria en febrero de 2026: 1,07 €");
  const leap = { "g.csv": "fecha,importe,categoria,descripcion\n2024-02-10,29.00,Ocio,Cine\n" };
  expect(said(["media", "g.csv", "--mes", "2024-02"], leap).out).toBe("Media diaria en febrero de 2024: 1,00 €");
  expect(said(["media", "gastos.csv", "--mes", "2026-13"])).toMatchObject({ code: 1, out: "Mes no válido: 2026-13" });
});
