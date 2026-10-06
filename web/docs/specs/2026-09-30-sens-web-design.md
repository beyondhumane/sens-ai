# La web de Sens

Fecha: 2026-09-30 · Estado: construida, en la carpeta `web/` del repositorio de Sens y publicada
en `https://sens.beyondhumane.com` con GitHub Pages desde `main` (ver *Publicación*); la etapa 2
está pendiente (ver *Etapas*).
Ámbito: `web/`. Lee del mismo repositorio: `src/brand/{tokens,mark}.ts`,
`rust/sens-app/ui/public/fonts/` y `rust/sens-app/ui/src` como referencia de la interfaz.

Este documento describe la web tal como está construida. Lo que aún no existe lo dice en su
sitio, marcado como pendiente.

## Decisiones

Tomadas el 2026-09-30 en la sesión de diseño.

- **Referencia**: la propuesta 03, «Manifiesto». Fondo blanco cálido, tipografía negra enorme,
  un gesto lima, composición asimétrica, la app como protagonista y `sens` a escala
  extraordinaria.
- **El lima es una excepción con significado** (ver *Excepciones a la identidad*): grande solo en
  la portada y en el cierre, y siempre significa foco.
- **Idioma**: inglés, el de origen de Sens (`docs/i18n.md`), el del README y el de GitHub, y los
  mismos seis idiomas que la app: inglés en `/`, y español, francés, alemán, japonés y chino bajo
  `/es/`, `/fr/`, `/de/`, `/ja/` y `/zh/`.
- **Base**: Astro con salida estática, GSAP (ScrollTrigger y CustomEase) y CSS.
- **Titular a peso 650**, el máximo de `identity.md` §5.2. La contundencia sale del tamaño y del
  espaciado, no de un peso ultranegro.
- **Tipografías**: Space Grotesk, la de la marca en `tokens.ts` y en la app (`identity.md` aún dice
  Geist), y Geist Mono para datos y código.
- **Repositorio que se enlaza**: `github.com/beyondhumane/sens-ai`. `iiTzSenn/Sens` redirige ahí.
- **Fuera de la web**: el logo de Windows del botón de la referencia, porque la identidad solo
  admite iconos Lucide. «for Windows» lo dice en texto. El Canon quedó fuera mientras no estaba
  publicado; desde la 0.30.0 es lo principal de la web (ver *El motor*).
- **Reglas del repositorio**: las de Sens. Ningún comentario en ningún fichero, e `identity.md`
  como fuente de verdad salvo las excepciones escritas aquí.
- **Recorrido**: escritorio con escena fija a partir de 1024 × 640 px; por debajo, tarjetas.

### Cambios durante el prototipo

Tomados el 2026-09-30 al revisar el prototipo en movimiento.

- **Claro y oscuro.** Un botón arriba a la derecha cambia de tema con una transición de escaneo
  (ver *Claro y oscuro*). La página sigue al sistema hasta que se elige.
- **La ventana sigue al tema.** En claro muestra el tema claro de la app y en oscuro, el oscuro.
- **La ventana, sin barra lateral.** El hilo y Changes a tamaño real bastan, y el encuadre no
  tiene que sacar nada.
- **Dos ventanas en la portada.** La de la portada se desplaza con la página; cuando el texto llega
  a la navegación, la ventana fija del escenario ocupa su sitio exacto y hace el viaje.
- **`sens` gigante.** Un trazado de Space Grotesk 700, más ancho y más pesado que el wordmark de la
  marca, de borde a borde. En la portada lo corta la primera vista y se ve entero al bajar, sin
  quitar sitio al titular ni a la ventana. Lleva ondas.
- **Flip no hace falta.** Los viajes se calculan con la geometría de la escena, y la ruta que viaja
  a Changes es una copia que vuela (`motion/ghost.ts`).

### El motor

Tomado el 2026-10-06, cuando la 0.30.0 publicó el Canon. Lo que hace a Sens distinto es el motor
que obliga a Claude Code a reutilizar y a escribir menos, y la web lo cuenta en cuatro sitios:

- **Portada**: la entradilla es *Claude Code that reuses what you have, and writes less.*, con
  «12% less code and 18% fewer tokens in our benchmark» debajo.
- **Demo**: un capítulo nuevo, 04 *Sens checks it.*, en el que Sens para una escritura que copiaba
  `greetingFor` y Claude la reutiliza. La demo pasa a seis capítulos.
- **Sección «What Sens checks»**, tras la demo: *Less code for the same work.*, las tres cifras
  con su condición, cuatro bloques (lo que ya tienes, cada cambio antes de llegar al disco, la
  segunda lectura, la última palabra es tuya), el aviso de turno retenido y los treinta lenguajes.
  Va detrás de la demo porque delante partiría el viaje de la ventana.
- **`/research`**: la investigación como página propia, al estilo de un anuncio de modelo, con las
  gráficas de Horizonte, las tablas de calibración, las reglas, lo que no funcionó, el método y el
  texto del Canon. Sale del paper `docs/paper/sens-canon.md` y de los datos del banco.

Y dos páginas de apoyo: **`/releases`**, con cada versión, sus notas recortadas como en la app y
sus créditos, y una **píldora de versión** junto al logo que lleva allí.

## Lo que la web puede afirmar

Solo hechos comprobados el 2026-09-30 en el README, en el código o en la API de GitHub. Ninguna
cifra de uso, testimonio, premio ni integración que no esté en esta tabla.

| Hecho | Fuente |
| --- | --- |
| App de escritorio para Claude Code sobre la CLI oficial sin modificar, un proceso por sesión | README, «What it is» |
| Versión 0.30.0, publicada el 2026-10-02; instalador `Sens_0.30.0_x64-setup.exe` de 12,1 MB | API de releases |
| Sens ve lo que ya existe y se lo da a Claude con cada mensaje; juzga cada cambio antes de que llegue al disco (copias, también renombradas, código sin usar, comentarios nuevos); pregunta por dependencias nuevas y tests quitados; un revisor deja notas que no paran nada; tras tres intentos el turno queda retenido con *Accept*, *Undo* o *Ask Claude to fix them* | Notas de la 0.30.0 y paper |
| El modelo no puede apagar el circuito ni rodearlo por la terminal, un subagente o un *commit* | Paper, §3.3 |
| Con Claude Sonnet 5.5, tras 30 tareas encadenadas, el proyecto queda un 12 % más pequeño (436 frente a 496 líneas, mediana) con un 18 % menos de tokens y 90 de 90 tareas aceptadas en los dos brazos | Paper, §5.6, y `bench/results` |
| Treinta lenguajes: diecinueve en nivel completo y once en básico | Notas de la 0.30.0 y paper, §3.5 |
| Windows 10 y 11, x64, con WebView2, y una cuenta de Claude: Pro, Max, Console o clave de API | README, «Requirements» |
| Se instala por usuario, sin permisos de administrador | README, «Download» |
| El instalador aún no está firmado: SmartScreen avisa y el SHA-256 va en las notas de la release | README |
| Cada llamada a una herramienta es un paso que se abre; al terminar, el trabajo se pliega en una línea que se abre; un fallo nunca se pliega | README, «A tour» |
| Files, Changes, Web, Terminal y Background junto al chat | README, «The project, beside the chat» |
| Plugins, skills y servidores MCP se instalan una vez, fijados a un commit, y se activan por proyecto; `~/.claude` no cambia | README, «Capabilities» y FAQ |
| Seis idiomas; oscuro, claro o como Windows, con cinco acentos | README |
| Actualizaciones integradas, verificadas con una clave compilada en la app | README |
| Licencia MIT; proyecto independiente, sin relación con Anthropic | LICENSE y README |

La versión, el tamaño y el enlace del instalador no se escriben a mano: los trae `npm run release`
(ver *Arquitectura*).

## Excepciones a la identidad

### El lima como foco

`identity.md` §3.2 prohíbe Signal como gran fondo y lo limita a un 5 % de la composición. La web
se salta esa regla a propósito, con estas condiciones:

- El lima grande es **un único objeto con tres estados**: *campo* en la portada, *marca* de 4 px
  junto a la ventana durante la demo y *campo* otra vez en el cierre. Son los estados `focus` y
  `done` de §8 llevados a la escala de la página.
- Solo es grande **en la portada y en el cierre**. En el resto de la página cumple la guía: foco,
  estado, resultado relevante o finalización.
- Nunca va detrás de texto largo. Sobre lima solo hay carbón: `carbon-950` sobre `signal-500`
  da 16,5:1. En paper el lima no es texto ni línea informativa, porque solo alcanza 1,1:1.
- En oscuro el lima no cambia. El cierre fija el esquema claro, y la parte derecha de la
  navegación también, mientras está sobre el campo de la portada.
- Dentro de la app reconstruida, el lima es exactamente el de la app.

### El `sens` gigante

§5.2 evita la tipografía de display ultranegra y fija 650 como peso máximo. El `sens` gigante no es
texto sino un trazado: Space Grotesk 700 con −0,08em de espaciado, ensanchada ×1,3. Su tamaño exige
ese peso para que la forma no se quede en un contorno fino. El wordmark de la navegación y de la
app no cambia.

### Las ondas

§8 pide que el movimiento explique lo que hace Sens y prohíbe las partículas decorativas. Las ondas
del `sens` gigante son movimiento decorativo continuo: líneas del color de la página que derivan
dentro de las letras y se apartan del puntero. El precedente es el grano dentro de «sens AI» en el
chat vacío de la app. Se limitan así:

- solo en escritorio, nunca con movimiento reducido;
- solo mientras el `sens` está a la vista;
- solo dentro de las letras.

Se mueven solas más de 5 s, así que necesitan un control de pausa (ver *Movimiento reducido*).

### El brillo del escaneo

El anillo del cambio de tema es una línea lima de 2 px con un rastro que se desvanece detrás de
ella, de hasta 18 px. Es el estado `scan` de §8 y dura lo que dura el cambio; la prohibición de
brillo de §4.2 es para el logo.

## Dirección de arte

### Color

Cada rol es `light-dark()` de dos tokens y `color-scheme` en `<html>` elige. Sin elección vale
`light dark`, así que la página sigue al sistema también sin JavaScript; con elección, `data-theme`
lo fija en `light` o en `dark`. Lo que no cambia de tema fija su propio esquema.

| Rol | Variable | Claro | Oscuro | Uso |
| --- | --- | --- | --- | --- |
| Fondo | `--paper` | `paper` | `carbon-950` | la página, la navegación fija, la hoja del menú |
| Superficie | `--bone` | `bone-100` | `carbon-800` | fondo al pasar por el menú, el índice y repetir |
| Tinta | `--ink` | `carbon-950` | `bone-50` | titulares y texto |
| Texto secundario | `--ink-soft` | `alloy-600` | `alloy-400` | entradillas y metadatos |
| Línea | `--hairline` | `bone-200` | `carbon-700` | bajo la navegación fija y entre los enlaces del menú |
| Borde de la ventana | `--frame` | `chalk-300` | `carbon-700` | 1 px alrededor de la ventana |
| Sombra | `--shade` | `carbon-950` | `carbon-950` | con `color-mix()`, la de la ventana y la de la hoja del menú |
| Foco | `--ring` | `signal-800` | `signal-500` | anillo de 2 px, separado 3 px |
| Botón principal | `--button` · `-hover` · `-press` | `carbon-950` · `carbon-800` · `carbon-700` | `bone-50` · `bone-200` · `bone-300` | Download |
| Tinta del botón | `--button-ink` | `paper` | `carbon-950` | |
| Cuerpo de la marca | `--mark-body` | `carbon-900` | `carbon-800` | la marca de la navegación (§4.2) |
| Lima | `--lime` | `signal-500` | `signal-500` | el objeto de foco de la excepción |
| Tinta sobre lima | `--lime-ink` | `carbon-950` | `carbon-950` | el texto seleccionado |
| App | los roles de la app | su tema claro | su tema oscuro | la ventana reconstruida |

Todo sale de `tokens.css`, generado desde `tokens.ts`. Las sombras y transparencias se escriben
con `color-mix()` sobre un token, nunca con un hex nuevo. La ventana usa los roles de la app con
los valores de `[data-mode]` en su `tokens.css`, y cada contraste de la página se calcula en un test
para los dos temas.

### Tipografía

Space Grotesk variable (300–700) y Geist Mono, ambas con licencia OFL y recortadas a latín.

| Rol | Tamaño | Peso · espaciado · interlineado |
| --- | --- | --- |
| Display (`h1`) | `max(4.5rem, min(7vw, 8 % del ancho de contenido))`; en móvil `clamp(3.5rem, 25vw, 7.5rem)` | 650 · −0,05em · 0,88 (0,86 en móvil) |
| `h2` de la demo | `clamp(2.5rem, 3.6vw, 3.5rem)` | 650 · −0,04em · 0,95 |
| `h2` del cierre | `clamp(2.75rem, 6vw, 6rem)` | 650 · −0,045em · 0,92 |
| `h3` | 1,5rem | 600 · −0,02em · 1,2 |
| Entradilla | `clamp(1.375rem, 2vw, 1.75rem)` | 400 · −0,01em · 1,25 |
| Cuerpo | 1,125rem | 400 · 0 · 1,55 |
| Pequeño | 0,9375rem | 400 · 0 · 1,5 |
| Microetiqueta | 0,75rem, mayúsculas | 600 · 0,08em · 1,33 |
| Mono | 0,8125rem, cifras tabulares | 450 · 0 · 1,5 |

El tope del titular sale del ancho de contenido para que en pantallas anchas no pase por debajo de
la ventana.

Los saltos de línea del titular son parte del diseño: *Less noise. / More sense.* en escritorio y
*Less / noise. / More / sense.* en móvil. Están en el HTML, no dejados al azar del ancho.

El `sens` gigante es un trazado en SVG, no texto (ver *Excepciones*). `scripts/display.ts` guarda
la forma y `npm run brand` la ensancha y escribe `wordmark.svg`, con una proporción alto/ancho de
0,21. Ocupa todo el ancho con 12 px de margen.

### Forma

- Retícula de 12 columnas, medianil de 24 px, márgenes `clamp(16px, 5.5vw, 96px)` y contenido de
  1440 px como máximo. El campo lima y el `sens` gigante sangran a sangre.
- Radios: 4 px en teclas y chips de código, 8 px en controles y 12 px en la ventana y las
  tarjetas de móvil.
- Bordes de 1 px. Una sola sombra, larga y suave, la de la ventana. En oscuro, donde esa sombra
  no se ve, la separa su borde.
- Iconos Lucide con trazo de 1,5 por debajo de 24 px. Los iconos de tipo de fichero dentro de la
  app son los de Material Icon Theme, como en la app (§6.5).

## Estructura y texto

Texto en inglés, con la voz de Sens: frases cortas y exactas, sin exclamaciones ni promesas
(`identity.md` §9). Ortografía británica, como el README («colours»).

### 0 · Navegación

Fija arriba. Marca y `sens` (enlace al inicio, «Sens, home»), **Product** (a la demo), **GitHub**
(externo), el botón **Download** y el botón de tema. Al bajar gana el fondo de la página y una
línea fina. En móvil: marca, Download compacto, tema y un botón de menú de 44 × 44.

### 1 · Portada

- `h1`: **Less noise. More sense.**
- Entradilla: *Claude Code, with everything in view.*
- Microetiqueta: `OPEN SOURCE · MIT · WINDOWS 10 & 11`
- Botón: **Download for Windows**, con el enlace directo al `.exe`. Nombre accesible: «Download for
  Windows: Sens 0.29.0 for Windows, 7.8 MB installer».
- Enlace: *View on GitHub*
- En móvil el botón lleva a la release en GitHub, y una línea dice «Sens runs on Windows 10 and
  11. Open this page on your PC to install it.»
- La frase «A desktop app for Claude Code…» abre la demo, y los requisitos en mono van al cierre.
- Composición: titular a la izquierda; campo lima arriba a la derecha, desde la séptima columna,
  sangrando por arriba y por la derecha; la ventana encima, desde la sexta columna, montada sobre
  la esquina inferior izquierda del campo, que baja 40 px más que ella; debajo, `sens` gigante de
  borde a borde, cortado por la primera vista.
- La ventana muestra el hilo y Changes con la sesión terminada y el trabajo plegado en una línea:
  el mismo estado con el que acaba la demo.

### 2 · Demo

- Microetiqueta `HOW IT WORKS`, `h2` **A chat that shows its work.** y la entradilla «A desktop
  app for Claude Code. Every step the agent takes on screen, with the project's files, changes,
  terminal and browser beside it.»
- Cinco capítulos, cada uno con `h3`, una frase y una descripción para lectores de pantalla de lo
  que muestra la escena:

| | `h3` | Texto |
| --- | --- | --- |
| 01 | You ask. | Type it, or say it. The request stays at the top of the thread, with the project and branch it works in. |
| 02 | Sens shows the work. | Every tool call is a step, in order: what Claude thought, read, searched, ran and edited, with how long each took. |
| 03 | Open any step. | A command opens to its output, in its own program's colours. A file and line open the file. |
| 04 | Sens checks it. | Every change is checked before it reaches the disk. Claude wrote a greeting helper the project already had; Sens stopped the write, showed the original, and Claude used it. |
| 05 | See what changed. | Each edit lands in Changes as a diff, file by file, against the last commit. |
| 06 | Read the result. | When the reply is done, the work folds into one line: what Claude did, and what Sens stopped, reused and approved. A failure is never folded away. |

- Etiqueta mono junto a la escena: `Replay · recreated from Sens 0.29`. Distingue la demo de una
  sesión real.
- Índice `01`–`05` para saltar entre capítulos. Se queda fijo bajo la navegación y solo entonces
  lleva fondo, para no cortar la sombra de la ventana mientras sube.

### 3 · Ventajas

Pendiente (etapa 2). Microetiqueta `BUILT FOR REAL WORK`. Tres bloques editoriales, no tarjetas,
cada uno con su prueba en la interfaz:

1. **Know what the agent is doing.** «Thoughts are titled by what they are about, with how long
   they took. Permission requests, questions and plans wait in the thread, where you are reading.»
   Prueba: un detalle ampliado de un razonamiento con su título y su tiempo, una petición de
   permiso y la línea «Worked · …» como botón real que se abre y se cierra.
2. **Keep the project in view.** «Files, changes, the project's pages, a terminal and background
   tasks sit beside the chat, so you check the work where it happens.» Prueba: pestañas **Files ·
   Changes · Web · Terminal · Background**, cada una con su panel reconstruido, sobre la banda
   `--bone`.
3. **Choose what each project loads.** «Install a plugin, skill or MCP server once, pinned to a
   commit. Enable it per project. Each session gets only what its project enables, and your
   `~/.claude` settings stay as they are.» Prueba: la tira real *Install once · Enable per project
   · Use it in sessions* de Capabilities y una lista de activación por proyecto.

Cada panel se reconstruye mirando la interfaz real (`npm run dev -w sens-app-ui` sirve la app con
datos simulados en el navegador), en sus dos temas.

### 4 · Bajo la ventana

Pendiente (etapa 2).

- `h2` **The same Claude Code, in a window.**
- «Under the window is the unmodified `claude` CLI, one process per session, resumed when you come
  back to it. Nothing Claude Code does is reimplemented, and nothing it does is hidden.»
- Lista de definiciones con los datos del README: Engine, Account, Footprint, Platform, Languages,
  Looks, Updates.

### 5 · Cierre

- Campo lima a todo el ancho, con esquema claro en los dos temas. `h2` **Give Claude Code a
  window.**
- **Download for Windows** y **Explore on GitHub**.
- Requisitos en una línea mono, con versión y tamaño de `release.json`: «v0.29.0 · 7.8 MB ·
  Windows 10 or 11 · x64 · WebView2 · A Claude account».
- Aviso: «The installer isn't code-signed yet, so Windows SmartScreen will warn you. Compare it
  with the SHA-256 in the release notes.»
- `sens` gigante entero, encima del pie.

### 6 · Pie

MIT licence · Releases · Issues · README, y la frase del README: «Sens is an independent project,
not affiliated with or endorsed by Anthropic. Claude and Claude Code are trademarks of Anthropic.»

No hay analítica ni cookies, así que no hay banner de consentimiento.

## La sesión de la demo

Es la sesión `orbit`, con la interfaz y la voz de la captura del README y datos que cuadran entre
sí. Vive en `src/content/session.ts` y es la única fuente: la pintan los componentes y la lee la
capa de movimiento.

- **Petición**: «The header should greet people by name, with a greeting that defaults to Welcome.
  Add a test for it.» Chips: carpeta `orbit` y rama `main`.
- **Pasos**, con las filas de Sens tal como las pinta la app (`SensStep.tsx`):
  1. Sens · *already has greetingFor*, con `greetingFor(name: string, greeting = "Welcome")`,
     `src/greeting.ts:1` y «used 3 times»
  2. Thought · *Reading how the header is built* · 0,6 s
  3. Read · `src/header.tsx` · 13 lines · 0,2 s
  4. Search · `<h1>` · 1 result · 0,3 s
  5. Thought · *Checking the tests before changing anything* · 0,8 s
  6. Run · `npm test` · 1,4 s, con salida `✓ src/header.test.tsx (2 tests)`,
     `✓ src/user.test.ts (8 tests)`, `Test Files 2 passed`, `Tests 10 passed`
  7. Thought · *Adding the prop with its default* · 0,7 s
  8. Sens · *stopped a write*: «Copies existing code» en `src/header.tsx:7`, con «Already in the
     project» `src/greeting.ts:1` y el código de `greetingFor`
  9. Thought · *Using greetingFor instead* · 0,4 s
  10. Edit · `src/header.tsx` · +4 −2 · 0,3 s
  11. Edit · `src/header.test.tsx` · +9 · 0,4 s
  12. Run · `npm test` · 1,5 s, con `src/header.test.tsx (4 tests)` y `Tests 12 passed`
  13. Sens · *approved the changes*
- **Pliegue**: «Worked · 2 commands · 1 file read · 2 edits · 1 search · Sens: 1 stop · 1 reused ·
  approved · 6.6 s», con el recuento que hace la app (`work.ts`) y, como en ella, solo «1 reused ·
  approved» en el acento.
- **Respuesta**: «Done. The header greets people by name now, and falls back to **Welcome** when no
  greeting is given:», un bloque TSX con `<Header user={alex} />` y `<Header user={alex}
  greeting="Good morning" />`, y dos viñetas: «`src/header.tsx` takes an optional `greeting` prop
  and reuses `greetingFor`» y «`src/header.test.tsx` covers both cases, and the suite passes:
  **12 tests**». Debajo, `7.6 s · 2.1k tokens`.
- **Changes**: `2 files +13 −2`. `header.tsx` modificado (+4 −2): importa `greetingFor` y el `h1`
  la llama; `header.test.tsx` modificado (+9). En el capítulo 01, antes de las ediciones, dice
  «No changes since the last commit.»
- **Pie del compositor**: `Opus 5.5 (1M)`, `Ask`, anillo de contexto al 31 % y `Medium`.

El código se colorea al compilar con Shiki y las gramáticas y temas de VS Code que usa la app:
cada token lleva su color de Light+ y el de Dark+ como `light-dark()`, así que cambia con el tema
sin JavaScript en el navegador.

## Coreografía

### Idea

**La información dispersa se convierte en trabajo visible y comprensible.** Lo organizan dos hilos:
el objeto lima (campo, luego marca, luego campo) y la ventana, que hace el viaje de la portada al
escenario y se queda allí hasta el último capítulo.

Vocabulario, y nada más: revelado por máscara, cambio de encuadre, viaje entre elementos
relacionados, despliegue, escala contenida, continuidad espacial y pausas. Nada de rebotes,
parallax repartido por la página, fundido con desplazamiento en cada sección, letra a letra,
cursores propios, marquesinas ni precargadores.

### Fotogramas clave

| # | Estado | Lo mueve |
| --- | --- | --- |
| 1 | Entrada a 400 ms: la primera línea del titular ya está, la segunda sube; el campo lima se abre desde su borde; una línea de escaneo cruza la ventana mientras aparece | Tiempo (CSS) |
| 2 | Portada estable a ~1 s: todo en reposo para leer | Tiempo |
| 3 | Encuadre: la ventana viaja al escenario y crece; el campo se estrecha hasta la marca; el `sens` gigante se funde | Scroll ligado |
| 4 | Capítulo 01: la marca junto a la petición; lo demás al 25 %; Changes vacío | Scroll elige, tiempo resuelve |
| 5 | Capítulo 02: el pliegue se abre y los pasos llegan en orden; el foco cae en el primer Run | Tiempo |
| 6 | Capítulo 03: ese Run se abre desde su fila con su salida | Tiempo |
| 7 | Capítulo 04: la ruta `src/header.tsx` del Edit viaja a la cabecera del fichero en Changes y el diff se despliega | Tiempo |
| 8 | Capítulo 05: el trabajo se pliega, la respuesta pasa a primer plano y la marca brilla una vez | Tiempo |

El fotograma 8 es el estado de la portada: la demo cierra el círculo. El cierre de la página es la
coda: el lima vuelve como campo.

### Entrada (tiempo, CSS)

Todo el contenido está en el HTML y es visible sin animación. La entrada son `@keyframes`
aplicados solo con `prefers-reduced-motion: no-preference`: si la animación no corre, el contenido
ya está.

| Pieza | Inicio | Duración | Movimiento | Curva |
| --- | --- | --- | --- | --- |
| Estructura y navegación | 0 | — | presentes desde el primer pintado | — |
| Campo lima | 0 ms | 720 ms | una línea de 4 px baja por su borde izquierdo en el primer 22 % y el campo se abre hacia la derecha (`clip-path`) | `enter` |
| Líneas del titular | 60 ms | 520 ms | suben `0.3em` dentro de su máscara, 70 ms entre líneas (entre palabras en móvil) | `enter` |
| Ventana | 300 ms | 600 ms | se revela de izquierda a derecha (`clip-path`) con escala de 1,02 a 1, mientras una línea lima de 2 px la cruza | `enter` |
| Entradilla, etiqueta y botones | 620 ms | 380 ms | aparecen por opacidad, sin desplazamiento | `enter` |
| Recorte de la app (móvil) | 480 ms | 480 ms | aparece por opacidad | `enter` |

A los 1000 ms todo está quieto. Los solapes son de ~40 %: nada espera a que acabe lo anterior.

### Encuadre (scroll ligado)

Solo en el recorrido de escritorio. `motion/geometry.ts` calcula las posiciones para cada tamaño:

- **Portada**: la ventana a partir de la sexta columna más un medianil, a `max(nav + 32 px, 12 %
  de la altura)`, con la escala que cabe hasta medio margen del borde y nunca más de 0,85 de la del
  escenario.
- **Escenario**: la ventana desde la cuarta columna, centrada en vertical bajo la navegación, a la
  escala que cabe (como mucho 1). Nunca es menor que en la portada.

La ventana de la portada se desplaza con la página. Cuando el bloque de texto de la portada llega a
8 px bajo la navegación, la ventana fija del escenario ocupa su sitio exacto y viaja al escenario a
lo largo del 45 % de la altura de pantalla, con progreso lineal, un scrub de 0,4 s y vuelta atrás
al subir. A la vez:

- el campo lima se estrecha en línea recta hasta la marca de 4 px, a 14 px del borde izquierdo de
  la ventana y tan alta como ella;
- el `sens` gigante se funde en el primer 30 % del recorrido.

El texto de la ventana se compone a escala 1 y se reduce en la portada, nunca al revés, para que no
quede borroso al crecer. `will-change` solo dura lo que dura el scrub.

### Capítulos (el scroll elige, el tiempo resuelve)

La columna izquierda desplaza los textos de los capítulos; la ventana queda fija a la derecha, con
la marca pegada a su borde. Cada capítulo ocupa el 88 % de la altura de pantalla más un 18 % de
respiro, y se activa cuando su texto cruza el 45 % de la pantalla.

Cada capítulo es un **estado** de una única línea de tiempo: pliegue abierto o cerrado, paso en
foco, paso desplegado, relación visible, qué se atenúa y dónde está la marca. Hacia delante se
reproduce la coreografía y hacia atrás la inversa. Saltar más de un capítulo con el índice
reproduce los intermedios comprimidos en 1,4 s como máximo.

| Transición | Qué ocurre | Duración | Curva |
| --- | --- | --- | --- |
| → 01 | La marca va a la petición; los pasos y la respuesta bajan al 25 %; Changes queda vacío | 500 ms | `move`, `control` |
| 01 → 02 | La línea «Worked» se abre; los pasos llegan en orden, 80 ms entre ellos, con los estados vivos de la app; pausa de 400 ms; el resto baja al 35 % y la marca viaja al primer Run | ~2,1 s | `enter`, `move` |
| 02 → 03 | El Run se abre desde su propia fila (altura, no aparición); su salida aparece línea a línea, 40 ms entre líneas; el hilo se desplaza si hace falta y la marca crece con él | 360 ms; la salida acaba hacia los 500 ms | `move`, `enter` |
| 03 → 04 | El Run se cierra (300 ms) y la parada de Sens se abre desde su fila con la regla, el sitio y el código que ya existía, igual que se abrió el Run; la marca la sigue | ~0,7 s | `move`, `enter` |
| 04 → 05 | La parada se cierra; el hilo baja al primer Edit y la marca lo sigue; una copia de la ruta `src/header.tsx` vuela a la cabecera del fichero en Changes en 700 ms y las dos se iluminan al llegar; el diff se despliega línea a línea, 30 ms entre líneas | ~1,8 s | `move`, `enter` |
| 05 → 06 | Los pasos se pliegan en una línea; la respuesta vuelve; la marca va al resultado y brilla una vez: 1,5 veces más ancha y `signal-100` en 200 ms, y de vuelta en 360 ms | ~1,1 s | `move`, `enter`, `control` |

Mientras un capítulo está activo no se mueve nada: es la pausa para leer. Nada se repite en bucle.

### Cierre

El cierre no tiene revelado propio: el campo ya está al llegar, con el `sens` gigante entero
encima del pie. El revelado desde una línea, el mismo gesto de la entrada, queda para la etapa 2 si
al verlo entero hace falta.

### Elementos compartidos

- **El lima**: campo → marca → campo.
- **La ventana**: la de la portada le pasa el testigo, en el mismo sitio, a la del escenario.
- **Un paso y su detalle**: el detalle se abre desde la fila del paso, sin aparecer de la nada.
- **La ruta de un fichero y su cabecera en Changes**: una copia de la ruta vuela de una a otra.

### Microinteracciones

Todas usan `fast` (140 ms) o `medium` (220 ms) con la curva `control`, y comparten sistema: fondo,
color, línea e icono. Nunca mueven el área donde se pulsa.

| Control | Reposo → hover | Foco | Pulsado · seleccionado |
| --- | --- | --- | --- |
| Botón principal | `--button` → `--button-hover`; la flecha de descarga baja 2 px dentro del botón | anillo de 2 px `--ring`, separado 3 px | `--button-press` |
| Enlace | subrayado de 1 px → 2 px; la flecha avanza 2 px | anillo | — |
| Índice de capítulos | número `--ink-soft` → `--ink` y fondo `--bone` | anillo | actual: `--ink`, raya de 2 px y `aria-current` |
| Paso de la demo | fondo del paso, como en la app | anillo | se abre desde su fila |
| Menú móvil | fondo `--bone`; icono `menu` → `x` por opacidad | anillo | la hoja se abre por máscara desde arriba; enlaces con 30 ms de escalonado |
| Tema | fondo `--bone` | anillo | el icono pasa de sol a luna, o al revés (ver *Claro y oscuro*) |
| Repetir (móvil) | fondo `--bone` | anillo | reproduce la tarjeta desde su inicio |

### Por qué scroll o tiempo

El scroll decide dónde estás; el tiempo decide cómo llegas. Va ligado al scroll lo espacial, el
encuadre: es continuo, reversible y respeta el desplazamiento nativo sin fijar la página durante
pantallas vacías. Va en tiempo lo que en la app ocurre en el tiempo, como los pasos que llegan, un
paso que se abre o un cambio que se relaciona con su paso: arrastrar texto con el scroll lo vuelve
mecánico. El lector marca el ritmo, porque cada capítulo espera en su estado hasta que llega el
siguiente.

## Claro y oscuro

- **Botón**: 44 × 44, arriba a la derecha; en escritorio tras Download y en móvil entre Download y
  el menú. Iconos Lucide Sun y Moon a 20 px con trazo 1,5, que muestran el tema actual. Nombre
  accesible «Dark mode», con `aria-pressed`. Sin JavaScript no aparece.
- **Elección**: la página sigue al sistema, y a sus cambios, hasta que se pulsa el botón; desde
  entonces guarda la elección en `localStorage` (`sens-theme`). Un script en `<head>` pone
  `data-theme` y el `theme-color` antes del primer pintado, así que nunca asoma el otro tema.
- **Transición**: con la API de View Transitions, un anillo lima de 2 px sale del centro del botón
  y cubre la página hacia oscuro, con el tema nuevo dentro y el anterior fuera. Hacia claro, el
  anillo entra desde los bordes y se cierra sobre el botón. Detrás del frente queda un rastro de
  hasta 18 px que crece con el radio. Dura 1100 ms (`scan`) con la curva `move`, y el radio llega a
  la esquina más lejana.
- **Icono**: hacia oscuro, los rayos del sol se recogen en el sentido de las agujas del reloj,
  30 ms entre rayos; el círculo se borra desde 140 ms y la luna se dibuja desde 220 ms, en 560 ms.
  Hacia claro, la luna se borra cuando llega el anillo, al 90 % del escaneo, y el sol sale: primero
  el círculo, luego los rayos en el mismo orden.
- **Nitidez**: mientras dura el cambio se suspenden las demás transiciones de color, para que lo
  que cruza el anillo ya esté en su color final.
- **La ventana** cambia con la página y el anillo la cruza también.
- Con movimiento reducido, sin soporte de View Transitions o cuando cambia el sistema, el cambio es
  inmediato.

## Móvil y tamaños intermedios

El recorrido de escritorio exige al menos 1024 px de ancho y 640 px de alto. Por debajo, o en móvil
en horizontal, se usa este:

- **Portada**: titular en cuatro líneas; CTA a todo el ancho y 56 px de alto; debajo, un bloque
  lima con la app recortada a la columna del hilo a tamaño real, no la ventana entera encogida;
  `sens` gigante a sangre.
- **Demo**: sin escena fija. Cada capítulo es un texto seguido de una tarjeta con el recorte que le
  toca, en el tema de la página: petición y compositor; pasos; Run abierto; Edit y, debajo, Changes;
  y resultado. Las tarjetas de 02, 03 y 04 reproducen su transición una vez, cuando se ve una
  cuarta parte de ellas (1,7 s como mucho), y tienen un botón de repetir de 44 px de alto. La marca
  lima va en el borde izquierdo del recorte. En el 04 la ruta viaja en vertical hasta la cabecera
  del diff.
- **Menú**: un `<details>` que funciona sin JS; con JS se cierra con `Esc`, al tocar fuera y al
  elegir un enlace.
- Se prueba en 375 × 812, 390 × 844, 768 × 1024, 1024 × 768, 844 × 390, 1280 × 720 y 1440 × 900,
  y al redimensionar en caliente.

## Movimiento reducido

Con `prefers-reduced-motion: reduce`, o con `?motion=reduce` para pruebas, se conservan el
contenido, el orden y las relaciones, y se quitan viajes, escalas y máscaras:

- Sin entrada: todo presente desde el primer pintado.
- La ventana de la portada se queda en la portada hasta que el texto llega a la navegación; entonces
  la del escenario aparece en su sitio, con la marca al lado, sin viaje.
- El scroll sigue eligiendo el capítulo, pero el cambio es inmediato y la marca salta. En el 04, la
  ruta del paso y la cabecera del fichero se iluminan a la vez, sin viajar.
- Las tarjetas de móvil se muestran en su estado final, sin botón de repetir.
- El `sens` gigante no lleva ondas y el tema cambia al instante.
- Las microinteracciones conservan color y fondo.

La entrada dura 1 s y cada capítulo o tarjeta menos de 2,5 s. Las ondas del `sens` gigante sí se
mueven solas más de 5 s, así que, según WCAG 2.2.2, necesitan un control de pausa. Está pendiente
(ver *Etapas*).

## Sin JavaScript

- Todo el texto está en el HTML y la entrada es CSS que no oculta nada de antemano.
- La demo muestra su estado final, el de la portada, y los capítulos como lista ordenada; el
  índice son anclas.
- La descarga y los enlaces son enlaces normales.
- El tema sigue al sistema y el botón de tema no aparece.

## Sistema de movimiento

En `src/motion/tokens.ts`. El build los escribe también como variables CSS, de modo que CSS y GSAP
leen los mismos valores.

| Token | Valor | Uso |
| --- | --- | --- |
| `fast` | 140 ms | hover, foco, pulsación (el `--fast` de la app) |
| `medium` | 220 ms | menú, índice, partes del icono de tema |
| `scene` | 600 ms | duración por defecto dentro de la escena |
| `entrance` | 1000 ms en total | la entrada |
| `confirm` | 560 ms | el brillo `done` y el dibujo de la luna (el `motion.confirm` de la marca) |
| `scan` | 1100 ms | el cambio de tema (el `motion.scan` de la marca) |
| `control` | `cubic-bezier(.2,.6,.3,1)` | controles (la curva de la app) |
| `enter` | `cubic-bezier(.16,1,.3,1)` | lo que entra |
| `exit` | `cubic-bezier(.7,0,.84,0)` | lo que sale, siempre breve |
| `move` | `cubic-bezier(.65,0,.35,1)` | viajes entre elementos, la marca y el anillo del tema |
| `linked` | lineal | todo lo ligado al progreso del scroll |
| `shift` | `0.3em` | desplazamiento corto del texto |
| `scale` | 1,02 | escala contenida de entrada |
| `stagger` | 70 ms en líneas, 80 ms en pasos, 30 ms en enlaces y en los rayos del icono | escalonado |
| `overlap` | 40 % | solape entre piezas de una secuencia |

No hay resortes: `identity.md` §8 prohíbe las curvas elásticas y ningún movimiento de esta web
necesita una sensación física.

## Arquitectura

```
web/
  CLAUDE.md               local, fuera de git
  README.md
  astro.config.ts
  package.json
  docs/
    specs/                este documento
    screenshots/          las imágenes del README
  scripts/
    brand.ts              tokens.css, brand.json, marca, favicon, `sens` gigante y fuentes desde el repositorio
    display.ts            el trazado de `sens` en Space Grotesk 700
    release.ts            versión, fecha, tamaño y enlace del instalador desde GitHub
    frames.ts             capturas por posición, por capítulo, de la entrada y del cambio de tema
    perf.ts               tiempos de fotograma con la CPU ralentizada, al desplazar o al cambiar de tema
    a11y.ts               axe sobre la página
  src/
    brand/                (generado) brand.json, wordmark.svg
    data/release.json     (generado, con el último dato bueno)
    content/              session.ts (la sesión, fuente única), chapters.ts, site.ts, icons.ts, release.ts
    layouts/Base.astro    <head>: fuentes y el tema antes del primer pintado
    pages/index.astro
    sections/             Nav, Hero, Demo, Closing, Footer
    product/              Window, Topbar, Thread, Request, Step, Run, Answer, Composer, Changes, Code, Crop
    lib/                  code.ts (Shiki con Light+ y Dark+), root.ts (fuentes y movimiento en CSS)
    motion/
      tokens.ts           tiempos y curvas
      boot.ts             preferencias, puntos de corte (gsap.matchMedia), menú y tema
      geometry.ts         dónde van la ventana, el campo y la marca en cada tamaño
      scene.ts            portada → escenario, la marca y el índice fijo
      stage.ts            estados de la escena y transiciones entre ellos
      chapters.ts         capítulo activo según el scroll
      ghost.ts            la ruta que vuela a Changes
      cards.ts            tarjetas de móvil
      waves.ts            las ondas del `sens` gigante
      theme.ts            claro y oscuro, y el escaneo
      parts.ts            búsquedas y medidas en el DOM
    styles/
      tokens.css          (generado)
      base.css            roles de color, tipografía base, foco
      page.css            secciones y escena
      product.css         la app reconstruida, con sus roles en los dos temas
      motion.css          entrada, anillo del tema e icono
  public/                 fuentes con su OFL, marca, favicon e iconos de tipo de fichero con su licencia
  test/                   contraste, geometría, tokens, release, icono de tema y colores de código
```

- **`npm run brand`** importa `tokens.ts` y `mark.ts` de `../src/brand`, o de la ruta en
  `SENS_REPO`, ensancha ×1,3 el trazado de `display.ts` y escribe `tokens.css` (con la proporción
  del `sens` gigante), `brand.json`, `wordmark.svg`, `mark.svg` y `favicon.svg`. Copia
  `space-grotesk.woff2` con su `OFL.txt` y Geist Mono desde `@fontsource-variable/geist-mono`. Lo
  generado se guarda en el repo, así que la web se construye sola, y no se edita a mano.
- **`npm run release`** lee la última release de `beyondhumane/sens-ai` y guarda versión, fecha,
  tamaño y enlace directo del `.exe` en `release.json`. Si la API falla conserva el último dato
  bueno, y en último caso los botones van a `releases/latest`.
- **Responsabilidades**: CSS para la entrada, las microinteracciones, el modo reducido y el anillo
  del tema; GSAP con ScrollTrigger y CustomEase para el scroll, la escena y los viajes; View
  Transitions para el cambio de tema; IntersectionObserver para pausar las ondas fuera de pantalla
  y saber cuándo el índice está fijo. Ninguna otra librería de animación. GSAP es gratuito, también
  para uso comercial, pero con licencia propia, no OSI; el resto de dependencias son de código
  abierto.
- Los componentes marcan con atributos `data-*` lo que la capa de movimiento necesita: pasos,
  rutas, cabeceras y destinos de la marca. El JavaScript no duplica textos ni estructura.
- La capa de movimiento es un único módulo diferido. Si falla, la página es la versión sin JS.

## Accesibilidad

Objetivo WCAG 2.2 AA, comprobado, no solo declarado.

- `header`, `nav`, `main`, `section` y `footer`; un solo `h1`; un `h2` por sección; enlace para
  saltar al contenido; `lang="en"`.
- Teclado en todo: el índice son enlaces con `aria-current="step"`; el menú es un `<details>`, que
  expone si está abierto. Los capítulos tienen `scroll-margin-top` para que la navegación fija no
  tape el foco (2.4.11). El patrón de flechas, `Home` y `End` para el índice queda para la etapa 2,
  con las pestañas de ventajas.
- Foco visible: 2 px `signal-800` en claro y `signal-500` en oscuro.
- Objetivos de 44 × 44 en los controles principales; ninguno baja de 24 × 24 (2.5.8).
- Los contrastes de cada pareja de colores usada se calculan en un test, en los dos temas.
- La escena de la demo es `inert` y `aria-hidden`: es una repetición, no una sesión. Cada capítulo
  lleva una frase para lectores de pantalla con lo que muestra la escena.
- El enlace de descarga dice qué baja y cuánto pesa. Los enlaces externos lo indican.
- El botón de tema dice «Dark mode» y su estado con `aria-pressed`.
- El color nunca es la única señal de un estado.

## Rendimiento

Presupuestos que se miden, no que se prometen:

- JavaScript ≤ 70 KB con gzip; mide ~55 KB en un único módulo con GSAP, ScrollTrigger y
  CustomEase. CSS ≤ 30 KB; mide ~8 KB.
- Dos fuentes recortadas a latín; la principal con `preload`.
- Se animan `transform` y `opacity`. `clip-path` queda para la entrada, los revelados y la apertura
  de pasos; `mask-image`, para el cambio de tema; `background-color`, para el brillo de la marca.
  Sin filtros ni blur.
- Tamaños reservados en todo recurso, para que el CLS sea ~0. El LCP es el titular, que es texto.
- Imágenes, si las hay, en AVIF o WebP, con dimensiones y carga diferida bajo el primer pantallazo.
- Las líneas de tiempo fuera de pantalla se pausan. Hay un solo bucle de `requestAnimationFrame`,
  el de las ondas: en escritorio, con movimiento completo y mientras el `sens` está a la vista.
- Fluidez comprobada con la CPU ralentizada ×4, midiendo los fotogramas largos durante el scroll y
  durante el cambio de tema.

## Verificación

- `astro check` y TypeScript estricto.
- Vitest para la lógica pura: geometría de la escena en ocho tamaños, tokens a CSS, contrastes en
  los dos temas, lectura de la release con su caída, orden de los rayos del icono de tema y colores
  de código con Light+ y Dark+.
- En el navegador, con los scripts del repositorio contra el servidor de desarrollo:
  - `npm run frames`: fotogramas clave capturados deteniendo las líneas de tiempo, por posición,
    por capítulo, de la entrada y del cambio de tema, en claro, en oscuro, con movimiento reducido
    y sin JS;
  - `npm run a11y`: axe sobre la página en los dos temas;
  - `npm run perf`: fluidez con la CPU ralentizada.
- Medido el 2026-09-30:
  - axe no encuentra nada que corregir en ningún tema, a 1888 × 930 ni a 390 × 844;
  - al desplazar la página entera a 1888 × 930 con la CPU ×4, el fotograma mediano dura 16,7 ms,
    menos del 3 % pasa de 33 ms y el CLS es 0;
  - al cambiar de tema a velocidad normal, ningún fotograma pasa de 33 ms.
- Lighthouse con Edge si está disponible. Solo se citan cifras medidas.
- Todos los enlaces y descargas apuntan a destinos reales.

## Etapas

1. **Prototipo de motion**, hecho y revisado en movimiento:
   - andamiaje, `brand` y `release`;
   - navegación con el botón de descarga y el de tema;
   - portada en escritorio y móvil, con su entrada y el `sens` gigante con ondas;
   - la ventana reconstruida, en los dos temas de la app;
   - el encuadre hacia la demo;
   - los cinco capítulos como estados, con la coreografía completa y el índice;
   - las tarjetas de móvil de 02, 03 y 04;
   - claro y oscuro con la transición de escaneo;
   - cierre y pie;
   - movimiento reducido y versión sin JS.
2. **Web completa**, pendiente:
   - ventajas y bajo la ventana;
   - un control de pausa para las ondas (WCAG 2.2.2);
   - el patrón de teclado del índice y de las pestañas;
   - imagen para compartir;
   - el revelado del cierre, si hace falta al verlo entero;
   - la revisión completa: claridad del mensaje, fidelidad al producto, tipografía, coherencia del
     movimiento, lectura entre transiciones, saltos de layout, teclado y táctil, redimensionado,
     las siete medidas, carga lenta, movimiento reducido, enlaces reales, Lighthouse y rendimiento
     observado.

## Publicación

- **Dónde**: la carpeta `web/` de `github.com/beyondhumane/sens-ai`, con su propio
  `package-lock.json` y fuera de los workspaces de npm, de modo que el `tsc` y el `vitest` de la
  raíz no la alcanzan.
- **Cómo**: `.github/workflows/pages.yml` instala, prueba, lee las releases, construye y despliega
  en GitHub Pages en cada push a `main` que toque `web/`, y también a mano. El job `web` de
  `ci.yml` comprueba tipos, prueba y construye en todas las ramas.
- **Releases**: publicar, editar o borrar una release relanza el workflow sobre `main` (el entorno
  `github-pages` solo despliega desde `main`), y el workflow de release lo relanza también cuando
  ya ha adjuntado el instalador y escrito su SHA-256. `npm run release` corre en cada build, así
  que la versión de la navegación, la descarga, `/releases` con sus notas y créditos, y el pie
  siguen a la última release; si GitHub no responde, se usan los datos del repositorio.
- **Dirección**: `https://sens.beyondhumane.com`. `actions/configure-pages` entrega el origen y la
  ruta base como `SITE_URL` y `BASE_PATH`, y `withBase()` antepone esa ruta a cada enlace; sin
  dominio propio, la misma build sirve en `beyondhumane.github.io/sens-ai/`.
- **Dominio**: CNAME a `beyondhumane.github.io` en Cloudflare, en *DNS only*, porque con el proxy
  GitHub no puede renovar el certificado. HTTPS obligatorio; `http://` y la dirección de
  `github.io` redirigen con 301.

## Fuera de alcance

Analítica, y un blog o documentación más allá de `/research`.
