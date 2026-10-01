Mi banco me deja bajar los movimientos en un CSV como este:

```
Fecha;Concepto;Importe
09/03/2026;Bar Paco;-8,75
10/03/2026;Nómina;1.850,00
```

Los importes negativos son gastos y los positivos, ingresos. Orden nueva: `importar <fichero-del-banco> <fichero>`: añade al final de `<fichero>` los gastos del banco (solo los negativos, en positivo), con el concepto como descripción y la categoría `Sin categoría`, y responde `Importados 1 gastos`. Si alguna línea del banco no se entiende, no importa nada y dice cuál: `Línea 3 del banco no válida` (la cabecera es la línea 1).
