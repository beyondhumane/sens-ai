# Horizonte

Modelo: claude-sonnet-5-5 · medium. Pasos registrados: 60. Secuencias por brazo: 1.

## Paso a paso

Media de las secuencias de cada brazo, sobre el proyecto entero después de cada paso.

| Paso | Tarea | C0 aceptadas | C0 duplicadas | C0 líneas | C0 tokens | C2 aceptadas | C2 duplicadas | C2 líneas | C2 tokens |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 01-fechas | 1/1 | 0 | 93 | 263440 | 1/1 | 0 | 89 | 265827 |
| 2 | 02-categorias | 1/1 | 0 | 103 | 181771 | 1/1 | 0 | 97 | 222522 |
| 3 | 03-total-mes | 1/1 | 0 | 116 | 228834 | 1/1 | 0 | 103 | 222258 |
| 4 | 04-buscar | 1/1 | 0 | 123 | 321921 | 1/1 | 0 | 108 | 279324 |
| 5 | 05-anadir | 1/1 | 0 | 146 | 388350 | 1/1 | 0 | 122 | 286470 |
| 6 | 06-informe | 1/1 | 0 | 164 | 291104 | 1/1 | 0 | 140 | 280009 |
| 7 | 07-lista-fechas | 1/1 | 0 | 178 | 545037 | 1/1 | 0 | 154 | 480410 |
| 8 | 08-comillas | 1/1 | 0 | 222 | 285412 | 1/1 | 0 | 157 | 282627 |
| 9 | 09-exportar | 1/1 | 0 | 233 | 301886 | 1/1 | 0 | 167 | 234632 |
| 10 | 10-buscar-categoria | 1/1 | 0 | 235 | 224225 | 1/1 | 0 | 169 | 228640 |
| 11 | 11-presupuesto | 1/1 | 0 | 262 | 302464 | 1/1 | 0 | 196 | 559079 |
| 12 | 12-informe-presupuesto | 1/1 | 0 | 290 | 469072 | 1/1 | 0 | 210 | 287827 |
| 13 | 13-semana | 1/1 | 0 | 311 | 297119 | 1/1 | 0 | 225 | 534808 |
| 14 | 14-total-categoria | 1/1 | 0 | 323 | 279886 | 1/1 | 0 | 236 | 237926 |
| 15 | 15-importe-coma | 1/1 | 0 | 330 | 374837 | 1/1 | 0 | 237 | 231123 |
| 16 | 16-importar-banco | 1/1 | 0 | 370 | 366026 | 1/1 | 0 | 271 | 364139 |
| 17 | 17-media | 1/1 | 0 | 379 | 408943 | 1/1 | 0 | 281 | 342813 |
| 18 | 18-informe-anual | 1/1 | 0 | 393 | 305501 | 1/1 | 0 | 299 | 447718 |
| 19 | 19-lista-orden | 1/1 | 0 | 398 | 439292 | 1/1 | 0 | 311 | 344967 |
| 20 | 20-duplicados | 1/1 | 0 | 413 | 302559 | 1/1 | 0 | 322 | 453438 |
| 21 | 21-avisos | 1/1 | 0 | 427 | 310240 | 1/1 | 0 | 339 | 562960 |
| 22 | 22-renombrar | 1/1 | 0 | 441 | 253222 | 1/1 | 0 | 352 | 191435 |
| 23 | 23-exportar-banco | 1/1 | 0 | 458 | 242316 | 1/1 | 0 | 363 | 336432 |
| 24 | 24-buscar-fechas | 1/1 | 0 | 463 | 224112 | 1/1 | 0 | 366 | 278913 |
| 25 | 25-resumen | 1/1 | 0 | 475 | 755327 | 1/1 | 0 | 378 | 409737 |
| 26 | 26-anadir-futuro | 1/1 | 0 | 476 | 317981 | 1/1 | 0 | 379 | 277556 |
| 27 | 27-comparar | 1/1 | 0 | 499 | 357762 | 1/1 | 0 | 397 | 305103 |
| 28 | 28-informe-csv | 1/1 | 0 | 507 | 361724 | 1/1 | 0 | 410 | 362385 |
| 29 | 29-categorias-mes | 1/1 | 0 | 509 | 250204 | 1/1 | 0 | 410 | 283481 |
| 30 | 30-lista-categoria | 1/1 | 0 | 515 | 292207 | 1/1 | 0 | 411 | 204475 |

## En el paso 30

Diferencia de medianas frente a C0, con intervalo al 95 % por bootstrap sobre las secuencias.

| Medida | C0 | C2 | C2 − C0 |
| --- | --- | --- | --- |
| Líneas duplicadas (jscpd) | 0 | 0 | — |
| Funciones con casi-copia (Sens) | 0 | 2 | — |
| Líneas de código | 515 | 411 | — |
| Funciones | 24 | 37 | — |
| Código muerto | 0 | 0 | — |
| Tests ocultos rotos | 0 | 0 | — |
| Sitios que hacen «acentos» | 1 | 1 | — |
| Sitios que hacen «dias» | 1 | 1 | — |
| Sitios que hacen «dinero» | 1 | 1 | — |
| Sitios que hacen «fechas» | 1 | 1 | — |
| Sitios que hacen «fechas_validas» | 3 | 3 | — |
| Sitios que hacen «importes» | 1 | 1 | — |
| Sitios que hacen «meses» | 1 | 2 | — |
| Sitios que hacen «meses_validos» | 2 | 1 | — |
| Sitios que hacen «opciones» | 7 | 8 | — |

Tareas aceptadas hasta el paso 30: C0 30/30 · C2 30/30.

## Coste por tarea

Pendiente de los tokens de cada tarea a lo largo de la secuencia (tokens más por cada tarea que pasa).

| Brazo | Pendiente | Frente a C0 |
| --- | --- | --- |
| C0 | 1912 | — |
| C2 | 1379 | — |
