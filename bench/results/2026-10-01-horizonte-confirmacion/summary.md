# Horizonte

Modelo: claude-sonnet-5-5 · medium. Pasos registrados: 180. Secuencias por brazo: 3.

## Paso a paso

Media de las secuencias de cada brazo, sobre el proyecto entero después de cada paso.

| Paso | Tarea | C0 aceptadas | C0 duplicadas | C0 líneas | C0 tokens | C2 aceptadas | C2 duplicadas | C2 líneas | C2 tokens |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 01-fechas | 3/3 | 0 | 92 | 264154 | 3/3 | 0 | 93 | 298807 |
| 2 | 02-categorias | 3/3 | 0 | 102 | 228929 | 3/3 | 0 | 102 | 261335 |
| 3 | 03-total-mes | 3/3 | 0 | 112 | 369471 | 3/3 | 0 | 110 | 274247 |
| 4 | 04-buscar | 3/3 | 0 | 121 | 263461 | 3/3 | 0 | 118 | 261361 |
| 5 | 05-anadir | 3/3 | 0 | 145 | 394098 | 3/3 | 0 | 136 | 272792 |
| 6 | 06-informe | 3/3 | 0 | 162 | 385999 | 3/3 | 0 | 159 | 318553 |
| 7 | 07-lista-fechas | 3/3 | 0 | 179 | 234639 | 3/3 | 0 | 179 | 298120 |
| 8 | 08-comillas | 3/3 | 0 | 221 | 422461 | 3/3 | 0 | 191 | 398325 |
| 9 | 09-exportar | 3/3 | 0 | 233 | 609700 | 3/3 | 0 | 201 | 376303 |
| 10 | 10-buscar-categoria | 3/3 | 0 | 234 | 255267 | 3/3 | 0 | 202 | 257339 |
| 11 | 11-presupuesto | 3/3 | 0 | 259 | 843053 | 3/3 | 0 | 230 | 384050 |
| 12 | 12-informe-presupuesto | 3/3 | 0 | 279 | 545594 | 3/3 | 0 | 242 | 284111 |
| 13 | 13-semana | 3/3 | 0 | 299 | 409660 | 3/3 | 0 | 263 | 404103 |
| 14 | 14-total-categoria | 3/3 | 0 | 306 | 233596 | 3/3 | 0 | 269 | 237039 |
| 15 | 15-importe-coma | 3/3 | 0 | 316 | 563056 | 3/3 | 0 | 271 | 264964 |
| 16 | 16-importar-banco | 3/3 | 0 | 355 | 409392 | 3/3 | 0 | 304 | 378742 |
| 17 | 17-media | 3/3 | 0 | 365 | 365405 | 3/3 | 0 | 313 | 295130 |
| 18 | 18-informe-anual | 3/3 | 0 | 381 | 308798 | 3/3 | 0 | 332 | 338301 |
| 19 | 19-lista-orden | 3/3 | 0 | 385 | 220934 | 3/3 | 0 | 336 | 330808 |
| 20 | 20-duplicados | 3/3 | 0 | 398 | 326222 | 3/3 | 0 | 350 | 298053 |
| 21 | 21-avisos | 3/3 | 0 | 416 | 454953 | 3/3 | 0 | 366 | 340804 |
| 22 | 22-renombrar | 3/3 | 0 | 436 | 345375 | 3/3 | 0 | 381 | 316753 |
| 23 | 23-exportar-banco | 3/3 | 0 | 445 | 271029 | 3/3 | 0 | 391 | 328641 |
| 24 | 24-buscar-fechas | 3/3 | 0 | 453 | 294505 | 3/3 | 0 | 397 | 279422 |
| 25 | 25-resumen | 3/3 | 0 | 470 | 400626 | 3/3 | 0 | 409 | 285542 |
| 26 | 26-anadir-futuro | 3/3 | 0 | 471 | 336721 | 3/3 | 0 | 410 | 260045 |
| 27 | 27-comparar | 3/3 | 0 | 493 | 381642 | 3/3 | 0 | 425 | 308006 |
| 28 | 28-informe-csv | 3/3 | 0 | 503 | 481997 | 3/3 | 0 | 432 | 365431 |
| 29 | 29-categorias-mes | 3/3 | 0 | 505 | 273856 | 3/3 | 0 | 433 | 256451 |
| 30 | 30-lista-categoria | 3/3 | 4 | 515 | 341868 | 3/3 | 0 | 439 | 256983 |

## En el paso 30

Diferencia de medianas frente a C0, con intervalo al 95 % por bootstrap sobre las secuencias.

| Medida | C0 | C2 | C2 − C0 |
| --- | --- | --- | --- |
| Líneas duplicadas (jscpd) | 6 | 0 | -6.0 [-6.0, +0.0] |
| Funciones con casi-copia (Sens) | 0 | 2 | +2.0 [+0.0, +3.0] |
| Líneas de código | 496 | 436 | -60.0 [-140.0, -28.0] |
| Funciones | 20 | 28 | +8.0 [+2.0, +14.0] |
| Código muerto | 0 | 0 | +0.0 [+0.0, +0.0] |
| Tests ocultos rotos | 0 | 0 | +0.0 [+0.0, +0.0] |
| Sitios que hacen «acentos» | 1 | 1 | +0.0 [+0.0, +0.0] |
| Sitios que hacen «dias» | 1 | 1 | +0.0 [+0.0, +0.0] |
| Sitios que hacen «dinero» | 1 | 1 | +0.0 [+0.0, +0.0] |
| Sitios que hacen «fechas» | 1 | 1 | +0.0 [+0.0, +0.0] |
| Sitios que hacen «fechas_validas» | 3 | 3 | +0.0 [+0.0, +0.0] |
| Sitios que hacen «importes» | 2 | 1 | -1.0 [-1.0, +0.0] |
| Sitios que hacen «meses» | 1 | 1 | +0.0 [+0.0, +1.0] |
| Sitios que hacen «meses_validos» | 2 | 1 | -1.0 [-1.0, +0.0] |
| Sitios que hacen «opciones» | 7 | 7 | +0.0 [-1.0, +8.0] |

Tareas aceptadas hasta el paso 30: C0 90/90 · C2 90/90.

## Coste por tarea

Pendiente de los tokens de cada tarea a lo largo de la secuencia (tokens más por cada tarea que pasa).

| Brazo | Pendiente | Frente a C0 |
| --- | --- | --- |
| C0 | -358 | — |
| C2 | -132 | +274.7 [-1488.8, +1891.4] |
