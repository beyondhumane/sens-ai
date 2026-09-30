# SensBench

Modelo: claude-sonnet-5-5 · medium. Ejecuciones: 18.

Claude Code corre aislado (`--safe-mode`): sin el CLAUDE.md, las skills, los plugins, los hooks ni los MCP de la persona.

Las medianas y diferencias usan solo las ejecuciones válidas: aceptadas, sin regresiones y sin error.

## Por tarea

| Tarea | Condición | Válidas | Aceptadas | Regresiones | Código neto | Tests netos | Ficheros nuevos | Dependencias | Duplicación añadida | Reutilizó | Tokens | Segundos |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| py-slugs | C0 | 3/3 | 3 | 0 | 2 | 4 | 0 | 0 | 0 | unicodedata 3/3 | 170800 | 15 |
| py-slugs | C1 | 3/3 | 3 | 0 | 2 | 4 | 0 | 0 | 0 | unicodedata 3/3 | 216532 | 14 |
| rust-quiet | C0 | 3/3 | 3 | 0 | 9 | 0 | 0 | 0 | 0 | config.quiet 3/3 | 214954 | 15 |
| rust-quiet | C1 | 3/3 | 3 | 0 | 9 | 0 | 0 | 0 | 0 | config.quiet 3/3 | 172825 | 12 |
| ts-attachments | C0 | 3/3 | 3 | 0 | 9 | 0 | 0 | 0 | 0 | dayjs 2/3 · formatBytes 3/3 | 266123 | 17 |
| ts-attachments | C1 | 3/3 | 3 | 0 | 9 | 0 | 0 | 0 | 0 | dayjs 3/3 · formatBytes 3/3 | 215502 | 15 |

## Diferencias frente a C0

Diferencia de medianas, con intervalo al 95 % por bootstrap (10000 remuestreos, semilla fija). «Todas» promedia las diferencias de cada tarea. Con menos de dos ejecuciones válidas en una celda no hay estimación.

| Métrica | Tarea | C1 − C0 | C2 − C0 |
| --- | --- | --- | --- |
| Líneas netas de código | py-slugs | +0 [-1, +0] | — |
| Líneas netas de código | rust-quiet | +0 [-6, +0] | — |
| Líneas netas de código | ts-attachments | +0 [-18, +0] | — |
| Líneas netas de código | Todas | +0 [-8, +0] | — |
| Líneas netas de tests | py-slugs | +0 [+0, +0] | — |
| Líneas netas de tests | rust-quiet | +0 [+0, +0] | — |
| Líneas netas de tests | ts-attachments | +0 [+0, +0] | — |
| Líneas netas de tests | Todas | +0 [+0, +0] | — |
| Duplicación añadida | py-slugs | +0 [+0, +0] | — |
| Duplicación añadida | rust-quiet | +0 [+0, +0] | — |
| Duplicación añadida | ts-attachments | +0 [+0, +0] | — |
| Duplicación añadida | Todas | +0 [+0, +0] | — |
| Tokens | py-slugs | +45732 [+45649, +45905] | — |
| Tokens | rust-quiet | -42129 [-42888, +3387] | — |
| Tokens | ts-attachments | -50621 [-81870, +86905] | — |
| Tokens | Todas | -15673 [-26245, +30252] | — |
