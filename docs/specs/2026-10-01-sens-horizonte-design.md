# Horizonte: un proyecto después de 30 tareas, con Sens y sin él

Fecha: 2026-10-01 · Estado: aprobada; piloto hecho; confirmación fijada, pendiente de medir (ver *Confirmación*).
Ámbito: `rust/sens-bench`, `bench/sequences/` (nuevo).

## Por qué

La calibración del Canon midió tareas sueltas: cada una parte de un proyecto limpio
y se juzga sola. Ahí Sens mejora, pero poco en lo que más se ve: C2 acierta el 93 %
frente al 89 % de C0, y escribe unas dos líneas menos por tarea, dentro del ruido.

El daño de una IA que no reutiliza no está en una tarea, está en la suma. Cada
función repetida hace que la siguiente tarea encuentre dos versiones de lo mismo,
y el proyecto crece y se enreda. Si Sens sirve, la diferencia debería abrirse
tarea a tarea. Esta spec mide eso.

## La pregunta

Después de 30 tareas encadenadas en el mismo proyecto, ¿cuánto código repetido,
cuánto tamaño y cuántos fallos tiene el proyecto que se hizo con Sens frente al
que se hizo sin él? ¿Y le cuesta más a la IA la tarea 30 que la 1?

## Hipótesis, fijadas antes de medir

Se escriben aquí y no se cambian después de ver los datos.

1. **Duplicación:** en la tarea 30, el proyecto de C2 tiene menos líneas duplicadas
   que el de C0, y la distancia crece con las tareas.
2. **Una sola versión de cada cosa:** en C2 cada concepto sembrado (ver abajo)
   tiene una implementación; en C0, más de una.
3. **Tamaño:** en la tarea 30, el código de C2 es más pequeño que el de C0.
4. **Coste:** los tokens por tarea crecen más despacio en C2 que en C0.
5. **Sin pérdida:** C2 no falla más tareas ni rompe más tests anteriores que C0.

La medida principal es la 1. Si sale y la 5 se cumple, el resultado se puede
contar; las demás lo explican.

## Diseño

### El proyecto

Un proyecto propio y pequeño en TypeScript, **Cuentas**: una librería y una línea
de comandos para llevar gastos personales. Empieza con 100 líneas en
`bench/sequences/cuentas/base`: el gasto, la lectura del CSV, `formatMoney`, la
entrada y salida (`memoryIo` para los tests), las órdenes `lista` y `total` y sus
tests con vitest. TypeScript porque es el lenguaje más usado con Claude Code y
donde el índice de Sens está más probado. Las dependencias se instalan sin red,
con un lockfile sacado del de Sens.

Es propio para poder escribir 30 tareas que dependan unas de otras y juzgarlas
con tests ocultos. El riesgo es diseñarlo a favor de Sens; para evitarlo, las 30
tareas, sus tests y sus soluciones de referencia se escriben y se guardan en un
commit **antes de la primera ejecución**, y no se tocan después.

### Las 30 tareas

Peticiones de producto como las haría una persona, en español, en un orden fijo.
Cada una añade algo que se ve (un informe, un filtro, una exportación, un aviso)
y algunas cambian algo que ya existía.

Por debajo, ocho **conceptos sembrados** que varias tareas necesitan sin decirlo:

| Concepto | Tareas que lo necesitan | ¿Existe al empezar? |
| --- | --- | --- |
| Escribir dinero | Casi todas | Sí, `formatMoney` |
| Escribir fechas como 01/03/2026 | 1, 4, 5, 7, 13, 19, 20, 23, 25, 30 | No |
| Meses, semanas y rangos de fechas | 3, 6, 7, 9, 13, 17, 18, 21, 24, 27, 29 | No |
| Sumar y agrupar | 2, 6, 12, 13, 18, 21, 27, 28, 29 | No |
| Comparar sin mayúsculas ni acentos | 4, 10, 11, 12, 14, 20, 22, 30 | No |
| Validar fechas e importes | 3, 5, 7, 11, 13, 15, 16, 18, 19, 23, 26 | No |
| Leer y escribir CSV, con comillas | 5, 8, 9, 11, 16, 22, 23, 28 | A medias: solo leer, sin comillas |
| Leer las opciones de una orden | 3, 6, 7, 9, 12, 14, 17, 18, 19, 21, 23, 24, 28, 29, 30 | No |

Cada tarea tiene una solución de referencia (`reference.patch`) escrita como lo
haría alguien que reutiliza: cada concepto vive en un solo sitio y las órdenes lo
llaman. La referencia termina en 514 líneas repartidas en 12 ficheros.

La primera vez que hace falta un concepto que no existe, la IA lo escribe. Las
siguientes, lo correcto es reutilizar lo que escribió ella misma antes. Así se
mide la reutilización de lo que ya había y la del código que el propio brazo va
dejando, que es lo que de verdad se acumula.

### Brazos y repeticiones

- **C0**, sin Sens, y **C2**, con el circuito entero y el Canon v1.1. C1 no entra:
  la pregunta es por el producto, y cada brazo cuesta 90 ejecuciones.
- **Tres secuencias por brazo.** Cada secuencia empieza en Cuentas limpio y hace
  las 30 tareas en orden, cada una sobre el resultado de la anterior de ese mismo
  brazo.
- **Una sesión nueva de Claude Code por tarea**, como hace una persona con cada
  cosa que pide. En C2 el estado de Sens (índice, excepciones, registro) sigue de
  una tarea a la siguiente, como en un proyecto real.
- Sonnet 5.5 medio y Claude Code aislado (`--safe-mode`), como en la calibración.
- Si una tarea falla, la secuencia sigue con el código como quedó: nadie arregla
  el proyecto a mano. Un proyecto que se degrada es parte de lo que se mide.

## Qué se mide

Después de cada tarea, sobre **todo el proyecto** y no solo sobre lo añadido:

| Medida | Cómo |
| --- | --- |
| Líneas duplicadas | jscpd sobre `src/`, independiente de Sens |
| Casi-copias que ve Sens | Funciones de `src/` con otra muy parecida según las huellas del índice (tipos 1 a 3 y pequeñas) |
| Sitios por concepto | Sondas de `sequence.toml`: cuántas funciones (o líneas sueltas) hacen cada cosa con su primitiva (`Intl.NumberFormat`, la tabla de meses, `normalize(`, el mensaje `Importe no válido`…); lo ideal es 1 |
| Tamaño | Líneas sin blancos de `src/` sin tests, ficheros y funciones |
| Código sin usar | Los candidatos de `dead_code` del índice dentro de `src/` |
| Acierto | Test oculto de la tarea; además, los ocultos de todas las anteriores (regresiones acumuladas) |
| Coste | Tokens y tiempo de la tarea |
| Lo que hizo Sens | Paradas, notas y aprobaciones del circuito (solo C2) |

## Análisis, fijado antes de medir

- Curvas por tarea de cada medida, con la media de las tres secuencias y su rango.
- Medida principal: diferencia C2 − C0 en líneas duplicadas en la tarea 30, con
  intervalo al 95 % por bootstrap sobre las secuencias.
- Pendiente de tokens por tarea en cada brazo (regresión lineal sobre las 30).
- Con tres secuencias por brazo el intervalo será ancho; se publica igual, y si
  la diferencia es grande frente a la variación entre secuencias, se dice así.
- Antes de las 180 ejecuciones va un piloto con una secuencia por brazo (60). Si
  el piloto muestra que el experimento no puede distinguir nada (por ejemplo,
  C0 tampoco duplica), se para y se rediseña en vez de gastar el resto.

## Cambios en `sens-bench`

- **Secuencias:** `bench/sequences/cuentas/` con el proyecto base, `sequence.toml`
  (instalación, tests, sondas de conceptos) y una carpeta por tarea con
  `prompt.md`, `accept/` y `reference.patch`.
- **`sens-bench sequence validate <secuencia>`:** aplica las referencias en orden
  y comprueba, en cada paso, que el test oculto de la tarea falla antes y pasa
  después, y que los de todas las anteriores siguen pasando. Al final mide la
  referencia con las mismas medidas, como punto de comparación.
- **`sens-bench sequence run <secuencia> --condition C0,C2 --reps 3`:** cada paso
  parte del resultado del paso anterior del mismo brazo y repetición. Avanza paso
  a paso en todos los brazos a la vez, para que una tanda cortada deje a todos en
  el mismo punto; guarda las medidas en `steps.jsonl` con el commit de cada paso
  y se reanuda desde ahí sin repetir lo hecho. `--steps` limita los pasos.
- **`sens-bench sequence report <carpeta>`:** la tabla paso a paso, las
  diferencias en el último paso al que llegaron todos y la pendiente del coste.

## Coste

| | Ejecuciones | Tiempo | Tokens (sobre todo lecturas de caché) |
| --- | --- | --- | --- |
| Piloto | 60 | 2,5–3 h | ~20 M |
| Resto | 120 | 5–6 h | ~40 M |

Las ejecuciones gastan cuota del plan: el piloto y el resto se piden antes.

## Orden de trabajo

| Paso | Contenido | Termina cuando |
| --- | --- | --- |
| 1 | Cuentas, las 30 tareas, sus tests ocultos y referencias, en un commit | Hecho: `sequence validate` pasa las 30 |
| 2 | Modo secuencia en `sens-bench`, con sus pruebas | Hecho: pruebas en verde con un modelo falso |
| 3 | Piloto: 1 secuencia por brazo | Curvas del piloto; decisión de seguir o rediseñar |
| 4 | Resto: 2 secuencias más por brazo | Resultado frente a las hipótesis, en esta spec |

## Piloto (2026-10-01)

Una secuencia por brazo, Sonnet 5.5 medio, `--safe-mode`, Canon v1.1: 60 pasos en
una hora. Resultados en `bench/results/2026-10-01-horizonte/`.

| En la tarea 30 | C0 | C2 | Referencia |
| --- | --- | --- | --- |
| Tareas aceptadas | 30/30 | 30/30 | 30/30 |
| Tests ocultos anteriores rotos | 0 | 0 | 0 |
| Líneas de código | 515 | 411 | 455 |
| Funciones | 24 | 37 | 50 |
| Líneas duplicadas (jscpd) | 0 | 0 | 0 |
| Funciones con casi-copia (Sens) | 0 | 2 | 0 |
| Sitios que leen opciones de la orden | 7 | 8 | 1 |
| Tokens en total | 9,94 M | 9,80 M | — |
| Pendiente de tokens por tarea | +1 912 | +1 379 | — |

Lo que dice, contra las hipótesis:

1. **Duplicación: no distingue.** Ninguno de los dos brazos copia bloques en un
   proyecto de este tamaño: jscpd da cero en los dos, y las sondas, una vez
   corregidas, encuentran cada concepto en un solo sitio en ambos, salvo leer las
   opciones de la orden, que los dos reescriben a mano en 7 u 8 sitios. Es la
   regla de parada que se fijó para el piloto: la medida principal no puede
   separar los brazos.
2. **Una sola versión de cada cosa:** igual en los dos.
3. **Tamaño: la diferencia que sí aparece.** Con las mismas 30 funcionalidades y
   todos los tests pasando, C2 termina con 411 líneas y C0 con 515, un 20 % menos,
   y por debajo de la referencia. La distancia se abre pronto (13 líneas en la
   tarea 3, 66 en la 9) y luego se mantiene en torno a 90–100. C2 escribe más
   funciones y más cortas; C0, menos y más largas. Ejemplo en la tarea 8: para
   leer descripciones entre comillas C0 escribió un lector de CSV entero (99
   líneas añadidas) y C2 dos funciones de una línea, porque la descripción es el
   último campo.
4. **Coste: igual en total**, con una pendiente algo menor en C2. Con una sola
   secuencia no se puede decir más.
5. **Sin pérdida: se cumple.**

Lo que hizo el circuito en C2: un bloqueo (R2, `parseBudgets` copia de
`parseExpenses` al 83 %, justo), dos «piénsalo» de R2 sobre lectores de opciones
repetidos (justos) y tres notas del revisor, **las tres equivocadas**: pedir que
`formatDate` se escribiera en línea por tener un solo uso (se reutilizó después),
llamar especulativa a la validación que impide romper el CSV con una coma, y decir
que `normalize` no estaba definida cuando lo estaba fuera del diff. Las notas no
paran nada, pero llegan al modelo y a la persona; el revisor necesita otro ajuste
en S1, S3 y S4.

Correcciones del instrumento, hechas después de mirar los diffs a mano como pedía
el diseño y aplicadas a los dos brazos con `sequence remeasure`: la sonda de
acentos contaba las llamadas a una función que C2 había llamado `normalize`, la de
fechas mezclaba los nombres de días y meses, y las de meses y días no veían las
tablas escritas con mayúscula.

**Una sola secuencia por brazo no basta para afirmar el 20 %.** En la calibración,
las líneas de una misma tarea variaban mucho entre repeticiones. Si se quiere
contar, el tamaño tiene que pasar a ser la medida principal declarada antes de
medir otra vez, con secuencias nuevas: el piloto sirvió para elegirla y no puede
confirmarla.

## Confirmación, fijada antes de medir

Escrita el 2026-10-01, después del piloto y antes de cualquier ejecución nueva.

- **Pregunta:** con las mismas 30 tareas y el mismo modelo, ¿termina el proyecto
  más pequeño con Sens que sin él?
- **Medida principal:** líneas sin blancos de `src/` sin tests en la tarea 30.
- **Criterio:** se afirma que C2 deja el proyecto más pequeño si **las tres
  secuencias de C2 terminan por debajo de las tres de C0**. Sin diferencia real,
  eso pasa por azar una vez de cada 20 (prueba exacta de permutación, p = 0,05 de
  un lado). Se publica también la diferencia de medianas y su porcentaje.
- **Condición:** C2 no acepta menos tareas que C0 ni rompe más tests anteriores,
  sumando las tres secuencias.
- **Secundarias, descriptivas:** tokens en total y su pendiente, funciones,
  duplicación de jscpd, casi-copias de Sens, sondas por concepto y lo que hizo el
  circuito, con cada parada revisada a mano.
- **Secuencias nuevas:** tres por brazo (`--reps 3` en una carpeta nueva). El
  piloto sirvió para elegir la medida y no cuenta.
- **Lo que se mide es Sens tal como queda:** Canon v1.1 y el revisor corregido
  tras el piloto (`1f60112`).
- **Coste:** 180 ejecuciones, unas 3 horas por lo que tardó el piloto.

## Riesgos

| Riesgo | Qué se hace |
| --- | --- |
| Diseñar las tareas a favor de Sens | Tareas, tests y referencias fijados en un commit antes de medir; conceptos sembrados con una primitiva común que no favorece a nadie |
| Una tarea fallida arrastra a las siguientes | Cada test oculto juzga solo su función, y la secuencia sigue; se cuentan aparte los fallos heredados |
| C0 tampoco duplica y no hay nada que ver | El piloto lo detecta antes de gastar el resto |
| Las sondas de conceptos cuentan mal | Se comprueban a mano en el piloto contra los diffs |
| Tres secuencias son pocas | Se dice; si el piloto deja dudas, se amplían antes de publicar nada |

## Después

Las mismas medidas alimentan el resumen que verá cada persona en la app («Sens
evitó 340 líneas, reutilizó 12 funciones de tu proyecto y paró 3 copias»), que
tendrá su propia spec. Con el resultado de Horizonte, ese resumen tiene una cifra
de referencia con la que compararse.
