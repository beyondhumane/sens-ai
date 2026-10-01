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

it("exports one month to another file", () => {
  const { code, out, files } = said(["exportar", "gastos.csv", "marzo.csv", "--mes", "2026-03"]);
  expect({ code, out }).toEqual({ code: 0, out: "Exportados 5 gastos a marzo.csv" });
  expect(files["marzo.csv"]).toBe(
    "fecha,importe,categoria,descripcion\n2026-03-01,12.50,Comida,Pan y fruta\n2026-03-02,45.20,Transporte,Gasolina\n2026-03-09,8.75,Comida,Café con Ana\n2026-03-10,60.00,Ocio,Concierto\n2026-03-15,23.40,Comida,Supermercado\n",
  );
  expect(files["gastos.csv"]).toBe(GASTOS);
});

it("exports everything without a month and keeps quoted descriptions quoted", () => {
  const files = { "g.csv": 'fecha,importe,categoria,descripcion\n2026-03-05,18.00,Ocio,"Cena, con amigos"\n2026-04-01,3.00,Comida,Pan\n' };
  const exported = said(["exportar", "g.csv", "s.csv"], files);
  expect(exported.out).toBe("Exportados 2 gastos a s.csv");
  expect(exported.files["s.csv"]).toBe(files["g.csv"]);
});
