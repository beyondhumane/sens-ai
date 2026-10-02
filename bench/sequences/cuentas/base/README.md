# Cuentas

Una línea de órdenes para llevar los gastos de casa en un CSV.

```
fecha,importe,categoria,descripcion
2026-03-01,12.50,Comida,Pan y fruta
2026-03-02,45.20,Transporte,Gasolina
```

- `fecha`: año-mes-día.
- `importe`: euros con punto decimal.

## Órdenes

| Orden | Qué hace |
| --- | --- |
| `lista <fichero>` | Un gasto por línea: fecha, categoría, importe y descripción, separados por dos espacios |
| `total <fichero>` | `Total: 57,70 €` |

Los errores salen como una línea de texto y la orden termina con código 1.

## Código

- `src/cli.ts`: `run(argv, io)` interpreta la orden y devuelve `{ code, output }`.
- `src/io.ts`: `memoryIo` para los tests y `diskIo` para el uso real; `io.today` es la fecha de hoy.
- `src/csv.ts`, `src/money.ts`, `src/expense.ts`: el fichero, el dinero y el gasto.

`npm test` pasa los tests y `npm run typecheck` comprueba los tipos.
