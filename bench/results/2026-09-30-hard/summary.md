# SensBench

Modelo: claude-sonnet-5-5 · medium. Ejecuciones: 45.

Claude Code corre aislado (`--safe-mode`): sin el CLAUDE.md, las skills, los plugins, los hooks ni los MCP de la persona.

Las medianas y diferencias usan solo las ejecuciones válidas: aceptadas, sin regresiones y sin error.

## Por tarea

| Tarea | Condición | Válidas | Aceptadas | Regresiones | Código neto | Tests netos | Ficheros nuevos | Dependencias | Duplicación añadida | Reutilizó | Tokens | Segundos |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| sens-bar-accents | C0 | 3/3 | 3 | 0 | 0 | 8 | 0 | 0 | 0 | plain 0/3 | 271458 | 25 |
| sens-bar-accents | C1 | 3/3 | 3 | 0 | -1 | 0 | 0 | 0 | 0 | plain 3/3 | 282674 | 19 |
| sens-bar-accents | C2 | 3/3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | plain 0/3 | 235479 | 15 |
| sens-bar-accents | C2·v2 | 3/3 | 3 | 0 | -1 | 0 | 0 | 0 | 0 | plain 3/3 | 239434 | 18 |
| sens-bar-accents | C2·v3 | 3/3 | 3 | 0 | -1 | 0 | 0 | 0 | 0 | plain 3/3 | 237246 | 30 |
| sens-file-language | C0 | 3/3 | 3 | 0 | 35 | 31 | 0 | 0 | 0 | languageOf 3/3 · titleOf 0/3 | 1022245 | 111 |
| sens-file-language | C1 | 3/3 | 3 | 0 | 31 | 30 | 0 | 0 | 0 | languageOf 3/3 · titleOf 1/3 | 935286 | 130 |
| sens-file-language | C2 | 3/3 | 3 | 0 | 31 | 0 | 0 | 0 | 0 | languageOf 3/3 · titleOf 0/3 | 512223 | 75 |
| sens-file-language | C2·v2 | 3/3 | 3 | 0 | 27 | 0 | 0 | 0 | 0 | languageOf 3/3 · titleOf 1/3 | 519841 | 102 |
| sens-file-language | C2·v3 | 3/3 | 3 | 0 | 12 | 0 | 0 | 0 | 0 | languageOf 3/3 · titleOf 1/3 | 438641 | 72 |
| sens-shelf-size | C0 | 3/3 | 3 | 0 | 0 | 10 | 0 | 0 | 0 | weigh 3/3 | 605030 | 58 |
| sens-shelf-size | C1 | 3/3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | weigh 3/3 | 254942 | 15 |
| sens-shelf-size | C2 | 3/3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | weigh 3/3 | 264600 | 17 |
| sens-shelf-size | C2·v2 | 3/3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | weigh 3/3 | 429459 | 49 |
| sens-shelf-size | C2·v3 | 3/3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | weigh 3/3 | 299597 | 35 |

## Diferencias frente a C0

Diferencia de medianas, con intervalo al 95 % por bootstrap (10000 remuestreos, semilla fija). «Todas» promedia las diferencias de cada tarea. Con menos de dos ejecuciones válidas en una celda no hay estimación.

| Métrica | Tarea | C1 − C0 | C2 − C0 | C2·v2 − C0 | C2·v3 − C0 |
| --- | --- | --- | --- | --- | --- |
| Líneas netas de código | sens-bar-accents | -1 [-1, -1] | +0 [+0, +0] | -1 [-1, -1] | -1 [-1, -1] |
| Líneas netas de código | sens-file-language | -4 [-25, +21] | -4 [-26, +20] | -8 [-24, +23] | -23 [-25, +2] |
| Líneas netas de código | sens-shelf-size | +0 [+0, +0] | +0 [+0, +0] | +0 [+0, +0] | +0 [+0, +1] |
| Líneas netas de código | Todas | -2 [-9, +7] | -1 [-9, +7] | -3 [-8, +7] | -8 [-9, +0] |
| Líneas netas de tests | sens-bar-accents | -8 [-8, +5] | -8 [-8, -3] | -8 [-8, -3] | -8 [-8, -3] |
| Líneas netas de tests | sens-file-language | -1 [-6, +22] | -31 [-33, -10] | -31 [-33, -6] | -31 [-33, -6] |
| Líneas netas de tests | sens-shelf-size | -10 [-12, +0] | -10 [-12, +0] | -10 [-12, +0] | -10 [-12, +0] |
| Líneas netas de tests | Todas | -6 [-8, +6] | -16 [-18, -6] | -16 [-18, -5] | -16 [-18, -5] |
| Duplicación añadida | sens-bar-accents | +0 [+0, +0] | +0 [+0, +0] | +0 [+0, +0] | +0 [+0, +0] |
| Duplicación añadida | sens-file-language | +0 [+0, +0] | +0 [+0, +0] | +0 [+0, +0] | +0 [+0, +0] |
| Duplicación añadida | sens-shelf-size | +0 [+0, +0] | +0 [+0, +0] | +0 [+0, +0] | +0 [+0, +0] |
| Duplicación añadida | Todas | +0 [+0, +0] | +0 [+0, +0] | +0 [+0, +0] | +0 [+0, +0] |
| Tokens | sens-bar-accents | +11216 [-168010, +193138] | -35979 [-170362, -32012] | -32024 [-170452, -30999] | -34212 [-219482, -25019] |
| Tokens | sens-file-language | -86959 [-215345, +779505] | -510022 [-642676, +17351] | -502404 [-768088, +180217] | -583604 [-799349, +3099] |
| Tokens | sens-shelf-size | -350088 [-417867, -291038] | -340430 [-430862, -287312] | -175571 [-412605, -98809] | -305433 [-423476, +18139] |
| Tokens | Todas | -141944 [-225343, +163029] | -295477 [-382346, -119686] | -236666 [-381940, -10864] | -307750 [-414323, -92011] |

## Circuito

En cuántas ejecuciones apareció cada etapa del circuito o cada regla que saltó (`etapa:regla`).

| Tarea | Condición | Circuito |
| --- | --- | --- |
| sens-bar-accents | C2 | anticipated 3/3 · passed 3/3 · write 3/3 |
| sens-bar-accents | C2·v2 | anticipated 3/3 · passed 3/3 · write 3/3 |
| sens-bar-accents | C2·v3 | anticipated 3/3 · passed 3/3 · reviewed 3/3 · write 3/3 |
| sens-file-language | C2 | anticipated 3/3 · landed 3/3 · passed 3/3 · write 2/3 |
| sens-file-language | C2·v2 | anticipated 3/3 · landed 3/3 · passed 3/3 · write 3/3 |
| sens-file-language | C2·v3 | anticipated 3/3 · landed 3/3 · passed 3/3 · reviewed 3/3 · write 3/3 |
| sens-shelf-size | C2 | anticipated 3/3 · passed 3/3 · write 3/3 |
| sens-shelf-size | C2·v2 | anticipated 3/3 · landed 2/3 · passed 3/3 · write 3/3 |
| sens-shelf-size | C2·v3 | anticipated 3/3 · blocked:S7 1/3 · passed 3/3 · reviewed 2/3 · reviewed:S7 2/3 · write 3/3 · write:R2 1/3 |
