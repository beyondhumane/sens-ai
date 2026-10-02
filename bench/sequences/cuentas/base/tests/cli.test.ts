import { describe, expect, it } from "vitest";
import { run } from "../src/cli";
import { memoryIo } from "../src/io";

const GASTOS = `fecha,importe,categoria,descripcion
2026-03-01,12.50,Comida,Pan y fruta
2026-03-02,45.20,Transporte,Gasolina
`;

const shown = (output: string) => output.replace(/ /g, " ");

describe("cuentas", () => {
  it("lists every expense", () => {
    const ran = run(["lista", "gastos.csv"], memoryIo({ "gastos.csv": GASTOS }));
    expect(ran.code).toBe(0);
    expect(shown(ran.output)).toBe("2026-03-01  Comida  12,50 €  Pan y fruta\n2026-03-02  Transporte  45,20 €  Gasolina");
  });

  it("adds everything up", () => {
    expect(shown(run(["total", "gastos.csv"], memoryIo({ "gastos.csv": GASTOS })).output)).toBe("Total: 57,70 €");
  });

  it("says what is missing or unknown", () => {
    expect(run(["lista"], memoryIo({}))).toEqual({ code: 1, output: "Falta el fichero de gastos" });
    expect(run(["borrar"], memoryIo({}))).toEqual({ code: 1, output: "Orden desconocida: borrar" });
    expect(run(["lista", "otro.csv"], memoryIo({}))).toEqual({ code: 1, output: "No existe otro.csv" });
  });
});
