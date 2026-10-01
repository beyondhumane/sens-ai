# SensBench

Modelo: claude-sonnet-5-5 · medium. Ejecuciones: 108.

Claude Code corre aislado (`--safe-mode`): sin el CLAUDE.md, las skills, los plugins, los hooks ni los MCP de la persona.

Las medianas y diferencias usan solo las ejecuciones válidas: aceptadas, sin regresiones y sin error.

## Por tarea

| Tarea | Condición | Válidas | Aceptadas | Regresiones | Código neto | Tests netos | Ficheros nuevos | Dependencias | Duplicación añadida | Reutilizó | Tokens | Segundos |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| py-ansi-sequences | C0 | 3/3 | 3 | 0 | 7 | 0 | 0 | 0 | 0 | _ansi_re 3/3 | 217230 | 59 |
| py-ansi-sequences | C1 | 3/3 | 3 | 0 | 0 | 9 | 0 | 0 | 0 | _ansi_re 3/3 | 276480 | 28 |
| py-ansi-sequences | C2 | 3/3 | 3 | 0 | 0 | 9 | 0 | 0 | 0 | _ansi_re 3/3 | 227927 | 49 |
| py-choice-suggest | C0 | 3/3 | 3 | 0 | 15 | 10 | 0 | 0 | 0 | _format_possibilities 3/3 · get_close_matches 3/3 | 315060 | 56 |
| py-choice-suggest | C1 | 2/3 | 3 | 1 | 12 | 11 | 0 | 0 | 4 | _format_possibilities 3/3 · get_close_matches 3/3 | 414033 | 55 |
| py-choice-suggest | C2 | 2/3 | 3 | 1 | 11 | 9 | 0 | 0 | 0 | _format_possibilities 3/3 · get_close_matches 3/3 | 350170 | 63 |
| py-deprecated-space | C0 | 2/3 | 2 | 0 | 1 | 0 | 0 | 0 | 0 | — | 307459 | 29 |
| py-deprecated-space | C1 | 3/3 | 3 | 0 | 0 | 11 | 0 | 0 | 3 | — | 452220 | 43 |
| py-deprecated-space | C2 | 3/3 | 3 | 0 | 0 | 14 | 0 | 0 | 1 | — | 376391 | 63 |
| py-path-home | C0 | 3/3 | 3 | 0 | 0 | 10 | 0 | 0 | 0 | expanduser 3/3 | 218743 | 22 |
| py-path-home | C1 | 3/3 | 3 | 0 | -1 | 10 | 0 | 0 | 0 | expanduser 3/3 | 221965 | 21 |
| py-path-home | C2 | 3/3 | 3 | 0 | 0 | 10 | 0 | 0 | 0 | expanduser 3/3 | 223874 | 44 |
| py-progress-final | C0 | 0/3 | 0 | 0 | — | — | — | 0 | — | make_step 3/3 | — | — |
| py-progress-final | C1 | 0/3 | 0 | 0 | — | — | — | 0 | — | make_step 1/3 | — | — |
| py-progress-final | C2 | 2/3 | 2 | 0 | 8 | 16 | 0 | 0 | 0 | make_step 2/3 | 258354 | 50 |
| py-prompt-ansi | C0 | 3/3 | 3 | 0 | 6 | 11 | 0 | 0 | 0 | should_strip_ansi 3/3 · strip_ansi 3/3 | 231434 | 37 |
| py-prompt-ansi | C1 | 2/3 | 2 | 0 | 6 | 17 | 0 | 0 | 0 | should_strip_ansi 3/3 · strip_ansi 3/3 | 424818 | 65 |
| py-prompt-ansi | C2 | 3/3 | 3 | 0 | 6 | 12 | 0 | 0 | 0 | should_strip_ansi 3/3 · strip_ansi 3/3 | 397254 | 79 |
| py-style-black | C0 | 3/3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | — | 169680 | 13 |
| py-style-black | C1 | 3/3 | 3 | 0 | 0 | 5 | 0 | 0 | 0 | — | 267777 | 25 |
| py-style-black | C2 | 3/3 | 3 | 0 | 0 | 5 | 0 | 0 | 0 | — | 367311 | 43 |
| rs-session-ids | C0 | 3/3 | 3 | 0 | 10 | 12 | 0 | 0 | 0 | is_uuid 3/3 | 377982 | 194 |
| rs-session-ids | C1 | 3/3 | 3 | 0 | 10 | 10 | 0 | 0 | 0 | is_uuid 3/3 | 461423 | 150 |
| rs-session-ids | C2 | 3/3 | 3 | 0 | 6 | 11 | 0 | 0 | 0 | is_uuid 3/3 | 332888 | 94 |
| rs-title-words | C0 | 3/3 | 3 | 0 | 7 | 7 | 0 | 0 | 0 | TITLE_LIMIT 3/3 | 323962 | 61 |
| rs-title-words | C1 | 3/3 | 3 | 0 | 7 | 7 | 0 | 0 | 0 | TITLE_LIMIT 3/3 | 319028 | 59 |
| rs-title-words | C2 | 3/3 | 3 | 0 | 7 | 8 | 0 | 0 | 1 | TITLE_LIMIT 3/3 | 276094 | 81 |
| sens-bar-accents | C0 | 3/3 | 3 | 0 | 0 | 5 | 0 | 0 | 0 | plain 0/3 | 270482 | 27 |
| sens-bar-accents | C1 | 3/3 | 3 | 0 | -1 | 8 | 0 | 0 | 0 | plain 3/3 | 499220 | 42 |
| sens-bar-accents | C2 | 3/3 | 3 | 0 | -1 | 8 | 0 | 0 | 0 | plain 3/3 | 461948 | 82 |
| sens-file-language | C0 | 3/3 | 3 | 0 | 31 | 31 | 0 | 0 | 0 | languageOf 3/3 · titleOf 1/3 | 1153472 | 171 |
| sens-file-language | C1 | 3/3 | 3 | 0 | 13 | 26 | 0 | 0 | 0 | languageOf 3/3 · titleOf 1/3 | 805944 | 149 |
| sens-file-language | C2 | 3/3 | 3 | 0 | 12 | 7 | 0 | 0 | 0 | languageOf 3/3 · titleOf 3/3 | 870550 | 272 |
| sens-shelf-size | C0 | 3/3 | 3 | 0 | 0 | 11 | 0 | 0 | 0 | weigh 3/3 | 843108 | 121 |
| sens-shelf-size | C1 | 3/3 | 3 | 0 | 0 | 10 | 0 | 0 | 0 | weigh 3/3 | 774809 | 109 |
| sens-shelf-size | C2 | 3/3 | 3 | 0 | 2 | 10 | 0 | 0 | 0 | weigh 3/3 | 546000 | 137 |

## Diferencias frente a C0

Diferencia de medianas, con intervalo al 95 % por bootstrap (10000 remuestreos, semilla fija). «Todas» promedia las diferencias de cada tarea. Con menos de dos ejecuciones válidas en una celda no hay estimación.

| Métrica | Tarea | C1 − C0 | C2 − C0 |
| --- | --- | --- | --- |
| Líneas netas de código | py-ansi-sequences | -7 [-10, -3] | -7 [-10, -5] |
| Líneas netas de código | py-choice-suggest | -4 [-8, +0] | -4 [-9, +0] |
| Líneas netas de código | py-deprecated-space | -1 [-2, +2] | -1 [-2, +0] |
| Líneas netas de código | py-path-home | -1 [-6, -1] | +0 [-5, +0] |
| Líneas netas de código | py-progress-final | — | — |
| Líneas netas de código | py-prompt-ansi | -0 [-4, +1] | +0 [-5, +3] |
| Líneas netas de código | py-style-black | +0 [+0, +0] | +0 [+0, +0] |
| Líneas netas de código | rs-session-ids | +0 [+0, +0] | -4 [-6, -4] |
| Líneas netas de código | rs-title-words | +0 [-1, +0] | +0 [+0, +2] |
| Líneas netas de código | sens-bar-accents | -1 [-1, +1] | -1 [-1, -1] |
| Líneas netas de código | sens-file-language | -18 [-21, +24] | -19 [-24, +17] |
| Líneas netas de código | sens-shelf-size | +0 [+0, +2] | +2 [+0, +2] |
| Líneas netas de código | Todas | -3 [-4, +1] | -3 [-4, +0] |
| Líneas netas de tests | py-ansi-sequences | +9 [-9, +12] | +9 [-9, +10] |
| Líneas netas de tests | py-choice-suggest | +1 [-3, +14] | -1 [-3, +10] |
| Líneas netas de tests | py-deprecated-space | +11 [+11, +13] | +14 [+3, +20] |
| Líneas netas de tests | py-path-home | +0 [+0, +0] | +0 [+0, +0] |
| Líneas netas de tests | py-progress-final | — | — |
| Líneas netas de tests | py-prompt-ansi | +6 [-1, +22] | +1 [-2, +13] |
| Líneas netas de tests | py-style-black | +5 [+5, +5] | +5 [+2, +5] |
| Líneas netas de tests | rs-session-ids | -2 [-3, -1] | -1 [-6, +0] |
| Líneas netas de tests | rs-title-words | +0 [-3, +3] | +1 [-2, +4] |
| Líneas netas de tests | sens-bar-accents | +3 [+0, +13] | +3 [+0, +8] |
| Líneas netas de tests | sens-file-language | -5 [-41, +2] | -24 [-46, -17] |
| Líneas netas de tests | sens-shelf-size | -1 [-1, +3] | -1 [-4, +4] |
| Líneas netas de tests | Todas | +2 [-1, +5] | +1 [-3, +3] |
| Duplicación añadida | py-ansi-sequences | +0 [+0, +0] | +0 [+0, +0] |
| Duplicación añadida | py-choice-suggest | +4 [+0, +7] | +0 [+0, +0] |
| Duplicación añadida | py-deprecated-space | +3 [+3, +3] | +1 [+0, +7] |
| Duplicación añadida | py-path-home | +0 [+0, +0] | +0 [+0, +0] |
| Duplicación añadida | py-progress-final | — | — |
| Duplicación añadida | py-prompt-ansi | +0 [+0, +0] | +0 [+0, +0] |
| Duplicación añadida | py-style-black | +0 [+0, +0] | +0 [+0, +0] |
| Duplicación añadida | rs-session-ids | +0 [+0, +0] | +0 [+0, +0] |
| Duplicación añadida | rs-title-words | +0 [-1, +0] | +1 [+0, +1] |
| Duplicación añadida | sens-bar-accents | +0 [+0, +0] | +0 [+0, +0] |
| Duplicación añadida | sens-file-language | +0 [+0, +0] | +0 [+0, +0] |
| Duplicación añadida | sens-shelf-size | +0 [+0, +0] | +0 [+0, +0] |
| Duplicación añadida | Todas | +1 [+0, +1] | +0 [+0, +1] |
| Tokens | py-ansi-sequences | +59250 [-93659, +201996] | +10697 [-129203, +103631] |
| Tokens | py-choice-suggest | +98973 [-69030, +208641] | +35110 [-165033, +176918] |
| Tokens | py-deprecated-space | +144761 [+11846, +307227] | +68932 [-68483, +684476] |
| Tokens | py-path-home | +3222 [-88061, +5123] | +5131 [-42645, +6589] |
| Tokens | py-progress-final | — | — |
| Tokens | py-prompt-ansi | +193384 [+57045, +232813] | +165820 [+20664, +222105] |
| Tokens | py-style-black | +98097 [-46234, +137257] | +197631 [+7008, +285833] |
| Tokens | rs-session-ids | +83441 [-28438, +132328] | -45094 [-146067, -287] |
| Tokens | rs-title-words | -4934 [-5890, +94412] | -47868 [-49255, -43472] |
| Tokens | sens-bar-accents | +228738 [+107324, +279047] | +191466 [+22458, +224698] |
| Tokens | sens-file-language | -347528 [-369848, +176478] | -282922 [-514000, -73464] |
| Tokens | sens-shelf-size | -68299 [-232720, +206280] | -297108 [-396518, +301112] |
| Tokens | Todas | +44464 [+1793, +103102] | +163 [-51836, +88340] |

## Circuito

En cuántas ejecuciones apareció cada etapa del circuito o cada regla que saltó (`etapa:regla`).

| Tarea | Condición | Circuito |
| --- | --- | --- |
| py-ansi-sequences | C2 | anticipated 3/3 · landed 3/3 · passed 3/3 · reviewed 3/3 · write 3/3 |
| py-choice-suggest | C2 | anticipated 3/3 · landed 2/3 · landed:R2 2/3 · passed 3/3 · reviewed 3/3 · write 3/3 · write:R2 2/3 · write:R8 1/3 |
| py-deprecated-space | C2 | anticipated 3/3 · landed 2/3 · landed:R2 1/3 · passed 3/3 · reviewed 2/3 · reviewed:S4 1/3 · write 3/3 · write:R2 1/3 · write:R8 1/3 |
| py-path-home | C2 | anticipated 3/3 · landed 3/3 · passed 3/3 · reviewed 3/3 · write 3/3 |
| py-progress-final | C2 | anticipated 3/3 · landed 3/3 · passed 3/3 · reviewed 3/3 · write 3/3 |
| py-prompt-ansi | C2 | anticipated 3/3 · landed 3/3 · passed 3/3 · reviewed 3/3 · write 3/3 |
| py-style-black | C2 | anticipated 3/3 · landed 3/3 · passed 3/3 · reviewed 3/3 · write 3/3 |
| rs-session-ids | C2 | anticipated 3/3 · landed 3/3 · passed 3/3 · reviewed 3/3 · write 3/3 |
| rs-title-words | C2 | anticipated 3/3 · landed 3/3 · passed 3/3 · reviewed 3/3 · write 3/3 |
| sens-bar-accents | C2 | anticipated 3/3 · landed 3/3 · passed 3/3 · reviewed 3/3 · write 3/3 |
| sens-file-language | C2 | anticipated 3/3 · landed 3/3 · passed 2/3 · pending 1/3 · reviewed 2/3 · write 3/3 |
| sens-shelf-size | C2 | anticipated 3/3 · landed 3/3 · passed 3/3 · reviewed 3/3 · write 3/3 |
