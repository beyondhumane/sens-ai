# El mapa del proyecto

Entrega 1 de 3. La 2 (ciclos y acoplamiento nuevo como regla del Canon) y la 3
(vista de grafo en la app) tendrán su propio spec sobre el mismo `Map`.

## Para qué

Que el modelo empiece cada sesión sabiendo cómo está repartido el proyecto
(qué áreas hay, por qué fichero se entra a cada una, de qué depende cada una y
qué ficheros sostienen al resto) y que lo siga sabiendo cuando el proyecto
cambia, sin gastar llamadas en explorar.

## El grafo

`sens-index/src/map.rs`, función pura del `Index`: `Map::of(&index)`. Se
recalcula en cada `Keeper::refresh` junto al `Catalog`; no se guarda en disco.

- **Aristas entre ficheros**: los imports resueltos del índice, más las
  referencias a un símbolo exportado cuyo nombre es único en el proyecto,
  entre ficheros del mismo lenguaje. Las referencias se resuelven por nombre:
  sin ese filtro, cualquier variable `path` o `text` enlazaba ficheros que no
  tienen nada que ver (sobre el repo de Sens, un fichero recién creado salía
  con 104 dependientes; con el filtro, con 7). Los imports solos no bastan:
  entre crates de Rust no se resuelven.
- **Áreas** (híbrido carpeta + grafo): se parte del árbol de carpetas y se
  baja un nivel mientras un área tenga más de 40 ficheros y subcarpetas. Una
  subcarpeta con menos de 3 ficheros se queda en el área de su padre. El nombre
  del área es su carpeta, estable entre ejecuciones.
- **Puertas** de un área: sus ficheros con más dependientes de otras áreas.
- **Centrales**: los ficheros de los que dependen más ficheros.
- **Desubicados**: ficheros cuya relación con otra área es al menos el doble
  que con la suya (y al menos 3 aristas). Se señalan, no se mueven.

Los ficheros de test cuentan para el grafo (los necesita `tests_for`) pero no
para puertas, centrales, desubicados ni dependencias entre áreas.

## Cómo llega al modelo

1. **Al abrir la sesión**: la tarjeta del proyecto (`sens-canon/src/card.rs`),
   que ya viaja en el system prompt, cambia su lista de módulos por carpeta por
   las áreas del mapa, con puertas, dependencias y exports más usados, más los
   ficheros centrales y los desubicados. Al estar en el system prompt
   sobrevive a la compactación y no rompe la caché.
2. **En cada mensaje** (`UserPromptSubmit`): el circuito compara dónde cae cada
   fichero ahora con lo último que el modelo vio y añade solo el delta
   (`+` fichero nuevo y su área, `-` fichero borrado, `~` fichero que cambia
   de área). Sin cambios, nada. Si el delta pasa del 20 % de los ficheros o de
   30 líneas, va la tarjeta entera.
3. Si el índice no estaba listo al abrir la sesión, el primer mensaje con el
   índice listo lleva la tarjeta entera. Tras una compactación, también.
4. Si el árbol cambió desde el último cierre de turno (ediciones fuera de
   Claude, `git pull`), el índice se refresca al recibir el mensaje.

## Herramientas nuevas del Canon

- `project_map` sin carpeta devuelve la tarjeta actual; con carpeta, la lista
  de ficheros de siempre.
- `impact { target }`: fichero o símbolo. Ficheros que dependen de él por
  distancia (hasta 3 pasos), áreas tocadas y tests que lo alcanzan.
- `tests_for { file }`: tests que alcanzan el fichero, del más cercano al más
  lejano, y si el propio fichero lleva tests dentro.
- `where_is { query }`: el `Catalog` (que ya entiende varios idiomas y rutas)
  agrupado por área, con la puerta de cada área encontrada.

## Pruebas

- `map.rs`: áreas, división de carpetas grandes, fusión de pequeñas, puertas,
  centrales, desubicados, dependientes por distancia, determinismo.
- `card.rs`: la tarjeta muestra áreas, centrales y desubicados.
- `tools.rs`: las nueve herramientas responden.
- `circuit.rs`: el delta aparece tras crear un fichero, desaparece sin cambios,
  y la tarjeta entera vuelve tras olvidar el mapa.

## La pestaña Mapa

El panel de herramientas pasa a tener pestañas: cada herramienta abierta queda
en la tira (una vez cada una), con su `×`, y un `+` abre el mismo menú que la
barra superior. Las pestañas abiertas se recuerdan entre arranques.

Mapa lee el mismo `Map` del `Keeper` por IPC (`canon_map`, `canon_reach`), sin
pasar por el modelo: ficheros centrales, áreas desplegables (puertas, lo que
usan, lo más usado, sus ficheros) y desubicados. Pulsar un fichero muestra lo
que depende de él por pasos y los tests que lo alcanzan; `impact` y
`tests_for` responden desde ese mismo cálculo (`canon/view.rs`).
