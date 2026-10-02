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

const QUOTED = `fecha,importe,categoria,descripcion
2026-03-05,18.00,Ocio,"Cena, con amigos"
2026-03-06,2.00,Comida,"Dijo ""hola"""
2026-03-07,1.50,Comida,Pan
`;

it("reads quoted descriptions", () => {
  expect(said(["lista", "q.csv"], { "q.csv": QUOTED }).out).toBe(
    '05/03/2026  Ocio  18,00 €  Cena, con amigos\n06/03/2026  Comida  2,00 €  Dijo "hola"\n07/03/2026  Comida  1,50 €  Pan',
  );
  expect(said(["total", "q.csv"], { "q.csv": QUOTED }).out).toBe("Total: 21,50 €");
});

it("writes descriptions with commas or quotes between quotes", () => {
  const comma = said(["añadir", "gastos.csv", "2026-03-19", "10", "Ocio", "Cena, con Ana"]).files["gastos.csv"];
  expect(comma.endsWith('2026-03-19,10.00,Ocio,"Cena, con Ana"\n')).toBe(true);
  const quoted = said(["añadir", "gastos.csv", "2026-03-19", "1", "Ocio", 'El "bueno"']).files["gastos.csv"];
  expect(quoted.endsWith('2026-03-19,1.00,Ocio,"El ""bueno"""\n')).toBe(true);
  expect(said(["buscar", "g.csv", "bueno"], { "g.csv": quoted }).out).toBe('19/03/2026  Ocio  1,00 €  El "bueno"');
});
