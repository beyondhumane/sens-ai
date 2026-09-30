# La web de Sens

Fecha: 2026-09-30 · Estado: diseño aprobado; prototipo pendiente.
Ámbito: `P:\sens-web`, repositorio nuevo. Lee de `P:\Sens`: `src/brand/{tokens,mark,wordmark}.ts`,
`rust/sens-app/ui/public/fonts/` y `rust/sens-app/ui/src` como referencia de la interfaz.

## Decisiones

Tomadas el 2026-09-30 en la sesión de diseño.

- **Referencia**: la propuesta 03, «Manifiesto». Fondo blanco cálido, tipografía negra enorme,
  un gesto lima, composición asimétrica, la app oscura como protagonista y `sens` a escala
  extraordinaria.
- **El lima es una excepción con significado** (ver *Excepción a la identidad*): grande solo en
  la portada y en el cierre, y siempre significa foco.
- **Idioma**: inglés, el de origen de Sens (`docs/i18n.md`), el del README y el de GitHub.
- **Base**: Astro con salida estática, GSAP (ScrollTrigger y Flip) y CSS.
- **Titular a peso 650**, el máximo de `identity.md` §5.2. La contundencia sale del tamaño y del
  espaciado, no de un peso ultranegro.
- **Tipografías**: Space Grotesk, la de la marca en `tokens.ts` y en la app (`identity.md` aún dice
  Geist), y Geist Mono para datos y código.
- **Repositorio que se enlaza**: `github.com/beyondhumane/sens-ai`. `iiTzSenn/Sens` redirige ahí.
- **Fuera de la web**: el Canon (rama `feat/canon`, sin publicar) y el logo de Windows del botón de
  la referencia, porque la identidad solo admite iconos Lucide. «for Windows» lo dice en texto.
- **Reglas del repositorio**: las de Sens. Ningún comentario en ningún fichero, e `identity.md`
  como fuente de verdad salvo la excepción escrita aquí.
- **Recorrido**: escritorio con escena fija a partir de 1024 × 640 px; por debajo, tarjetas.

## Lo que la web puede afirmar

Solo hechos comprobados el 2026-09-30 en el README, en el código o en la API de GitHub. Ninguna
cifra de uso, testimonio, premio ni integración que no esté en esta tabla.

| Hecho | Fuente |
| --- | --- |
| App de escritorio para Claude Code sobre la CLI oficial sin modificar, un proceso por sesión | README, «What it is» |
| Versión 0.29.0, publicada el 2026-09-29; instalador `Sens_0.29.0_x64-setup.exe` de 7,8 MB | API de releases |
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

## Excepción a la identidad: el lima como foco

`identity.md` §3.2 prohíbe Signal como gran fondo y lo limita a un 5 % de la composición. La web
se salta esa regla a propósito, con estas condiciones:

- El lima grande es **un único objeto con tres estados**: *campo* en la portada, *marca* de 4 px
  junto a la ventana durante la demo y *campo* otra vez en el cierre. Son los estados `focus` y
  `done` de §8 llevados a la escala de la página.
- Solo es grande **en la portada y en el cierre**. En el resto de la página cumple la guía: foco,
  estado, resultado relevante o finalización.
- Nunca va detrás de texto largo. Sobre lima solo hay carbón: `carbon-950` sobre `signal-500`
  da 16,5:1. En paper el lima no es texto ni línea informativa, porque solo alcanza 1,1:1.
- Dentro de la app reconstruida, el lima es exactamente el de la app.

## Dirección de arte

### Color

| Rol | Token | Uso |
| --- | --- | --- |
| Fondo | `paper` | la página |
| Fondo secundario | `bone-100` | la banda de las pestañas de ventajas |
| Tinta | `carbon-950` | titulares, texto y botón principal (18,6:1 sobre paper) |
| Texto secundario | `alloy-600` | entradillas largas, metadatos (6,3:1 sobre paper, 5,4:1 sobre bone-100) |
| Líneas | `bone-300` y `bone-200` | separadores y bordes finos |
| Foco en paper | `signal-800` | anillo de foco de 2 px (5,5:1 sobre paper) |
| Foco en oscuro | `signal-500` | anillo de foco dentro de la app |
| Lima | `signal-500` | el objeto de foco de la excepción |
| App | tokens del tema oscuro de la app | la ventana reconstruida |

Todo sale de `tokens.css`, generado desde `tokens.ts`. Las sombras y transparencias se escriben
con `color-mix()` sobre un token, nunca con un hex nuevo.

### Tipografía

Space Grotesk variable (300–700) y Geist Mono, ambas con licencia OFL y recortadas a latín.

| Rol | Tamaño | Peso · espaciado · interlineado |
| --- | --- | --- |
| Display (`h1`) | `clamp(4.5rem, 11.2vw, 11rem)`; en móvil `clamp(3.5rem, 23vw, 6rem)` | 650 · −0,05em · 0,88 |
| `h2` | `clamp(2.5rem, 5.6vw, 5rem)` | 650 · −0,04em · 0,95 |
| `h3` | 1,5rem | 600 · −0,02em · 1,2 |
| Entradilla | `clamp(1.375rem, 2vw, 1.75rem)` | 400 · −0,01em · 1,25 |
| Cuerpo | 1,125rem | 400 · 0 · 1,55 |
| Pequeño | 0,9375rem | 400 · 0 · 1,5 |
| Microetiqueta | 0,75rem, mayúsculas | 600 · 0,08em · 1,33 |
| Mono | 0,8125rem, cifras tabulares | 450 · 0 · 1,5 |

Los saltos de línea del titular son parte del diseño: *Less noise. / More sense.* en escritorio y
*Less / noise. / More / sense.* en móvil. Están en el HTML, con clases por punto de corte, no
dejados al azar del ancho.

El nombre `sens` a escala extraordinaria es el trazado de `wordmark.ts` en SVG, no texto: no
depende de la fuente y se corta con precisión.

### Forma

- Retícula de 12 columnas, medianil de 24 px (16 en móvil), márgenes `clamp(16px, 5.5vw, 96px)`
  y contenido de 1440 px como máximo. El campo lima y el wordmark sangran a sangre.
- Radios: 4 px en teclas y chips de código, 8 px en controles y 12 px en la ventana y las
  tarjetas de móvil.
- Bordes de 1 px. Una sola sombra, larga y suave, la de la ventana.
- Iconos Lucide con trazo de 1,5 por debajo de 24 px. Los iconos de tipo de fichero dentro de la
  app son los de Material Icon Theme, como en la app (§6.5).

## Estructura y texto

Texto en inglés, con la voz de Sens: frases cortas y exactas, sin exclamaciones ni promesas
(`identity.md` §9). Ortografía británica, como el README («colours»).

### 0 · Navegación

Fija arriba. Marca y `sens` (enlace al inicio, «Sens, home»), **Product** (a la demo), **GitHub**
(externo) y el botón **Download**. Al bajar gana fondo paper y una línea fina. En móvil: marca,
Download compacto y un botón de menú de 44 × 44.

### 1 · Portada

- `h1`: **Less noise. More sense.**
- Entradilla: *Claude Code, with everything in view.*
- Cuerpo: «A desktop app for Claude Code. Every step the agent takes on screen, with the
  project's files, changes, terminal and browser beside it.»
- Microetiqueta: `OPEN SOURCE · MIT`
- Botón: **Download for Windows**. Nombre accesible: «Download Sens 0.29.0 for Windows, 7.8 MB
  installer».
- Enlace: *View on GitHub*
- Mono: `v0.29.0 · 7.8 MB · Windows 10 & 11, x64`
- En móvil el botón lleva a la release en GitHub, y una línea dice «Sens runs on Windows 10 and
  11. Open this page on your PC to install it.»
- Composición: titular a la izquierda; campo lima arriba a la derecha, sangrando por arriba y por
  la derecha; la ventana encima, montada sobre su esquina inferior izquierda; `sens` gigante abajo,
  cortado por el borde de la sección.
- La ventana muestra la app entera, con la barra lateral, y la sesión ya terminada con el trabajo
  plegado en una línea: el mismo estado con el que acaba la demo.

### 2 · Demo

- Microetiqueta `HOW IT WORKS` y `h2` **A chat that shows its work.**
- Cinco capítulos, cada uno con `h3`, una frase y una descripción para lectores de pantalla de lo
  que muestra la escena:

| | `h3` | Texto |
| --- | --- | --- |
| 01 | You ask. | Type it, or say it. The request stays at the top of the thread, with the project and branch it works in. |
| 02 | Sens shows the work. | Every tool call is a step, in order: what Claude thought, read, searched, ran and edited, with how long each took. |
| 03 | Open any step. | A command opens to its output, in its own program's colours. A file and line open the file. |
| 04 | See what changed. | Each edit lands in Changes as a diff, file by file, against the last commit. |
| 05 | Read the result. | When the reply is done, the work folds into one line that opens to all of it. A failure is never folded away. |

- Etiqueta mono junto a la escena: `Replay · recreated from Sens 0.29`. Distingue la demo de una
  sesión real.
- Índice `01`–`05` para saltar entre capítulos con ratón o teclado.

### 3 · Ventajas

Microetiqueta `BUILT FOR REAL WORK`. Tres bloques editoriales, no tarjetas, cada uno con su prueba
en la interfaz:

1. **Know what the agent is doing.** «Thoughts are titled by what they are about, with how long
   they took. Permission requests, questions and plans wait in the thread, where you are reading.»
   Prueba: un detalle ampliado de un razonamiento con su título y su tiempo, una petición de
   permiso y la línea «Worked · …» como botón real que se abre y se cierra.
2. **Keep the project in view.** «Files, changes, the project's pages, a terminal and background
   tasks sit beside the chat, so you check the work where it happens.» Prueba: pestañas **Files ·
   Changes · Web · Terminal · Background**, cada una con su panel reconstruido, sobre la banda
   `bone-100`.
3. **Choose what each project loads.** «Install a plugin, skill or MCP server once, pinned to a
   commit. Enable it per project. Each session gets only what its project enables, and your
   `~/.claude` settings stay as they are.» Prueba: la tira real *Install once · Enable per project
   · Use it in sessions* de Capabilities y una lista de activación por proyecto.

Cada panel se reconstruye mirando la interfaz real (`npm run dev -w sens-app-ui` sirve la app con
datos simulados en el navegador).

### 4 · Bajo la ventana

- `h2` **The same Claude Code, in a window.**
- «Under the window is the unmodified `claude` CLI, one process per session, resumed when you come
  back to it. Nothing Claude Code does is reimplemented, and nothing it does is hidden.»
- Lista de definiciones con los datos del README: Engine, Account, Footprint, Platform, Languages,
  Looks, Updates.

### 5 · Cierre

- Vuelve el campo lima. `h2` **Give Claude Code a window.**
- **Download for Windows** y **Explore on GitHub**.
- Requisitos en una línea: «Windows 10 or 11 · x64 · WebView2 · A Claude account».
- Aviso: «The installer isn't code-signed yet, so Windows SmartScreen will warn you. Compare it
  with the SHA-256 in the release notes.»
- `sens` gigante, cortado por el pie.

### 6 · Pie

MIT · Releases · Issues · README, y la frase del README: «Sens is an independent project, not
affiliated with or endorsed by Anthropic. Claude and Claude Code are trademarks of Anthropic.»

No hay analítica ni cookies, así que no hay banner de consentimiento.

## La sesión de la demo

Es la sesión `orbit` de la captura del README, con la misma interfaz y los mismos textos. Los datos
se ajustan para que cuadren entre sí, porque la captura mezcla «1 edit» con tres ficheros cambiados
y un test «nuevo» que ya pasaba antes de crearse. Vive en `src/content/session.ts` y es la única
fuente: la pintan los componentes y la lee la capa de movimiento.

- **Petición**: «The header should greet people by name, with a greeting that defaults to Welcome.
  Add a test for it.» Chips: carpeta `orbit` y rama `main`.
- **Pasos**:
  1. Thought · *Reading how the header is built* · 0,6 s
  2. Read · `src/app.tsx` · 42 lines
  3. Search · `<h1>` · 2 results
  4. Thought · *Checking the tests before changing anything* · 0,8 s
  5. Run · `npm test -- --reporter=dot`, con salida `✓ src/app.test.tsx (10 tests)`,
     `Test Files 1 passed`, `Tests 10 passed`
  6. Thought · *Adding the prop with its default* · 0,7 s
  7. Edit · `src/app.tsx` · +5 −2
  8. Edit · `src/app.test.tsx` · +9
  9. Run · `npm test -- --reporter=dot`, con `Tests 12 passed`
- **Pliegue**: «Worked · 2 commands · 1 file read · 2 edits · 1 search», con el recuento que hace
  la app (`work.ts`: Tasks no cuenta, Edit y Write son ediciones).
- **Respuesta**: «Done. The header greets people by name now, and falls back to **Welcome** when no
  greeting is given:», un bloque TSX con `<App title="Orbit" />` y `<App title="Orbit"
  greeting="Good morning" />`, y dos viñetas: «`src/app.tsx` takes an optional `greeting` prop» y
  «`src/app.test.tsx` covers both cases, and the suite passes: **12 tests**». Debajo, `7.6 s ·
  2.1k tokens`.
- **Changes**: `2 files +14 −2`. `app.tsx` modificado (+5 −2) con el diff de la captura, sin su
  línea de comentario; `app.test.tsx` modificado (+9) con los dos tests nuevos.
- **Barra lateral**: proyectos `orbit` y `landing` con sesiones cuyos títulos se cortan como en la
  captura; la sesión abierta se llama *Greet people by name*. Usuario *Alex*.
- **Pie del compositor**: `Opus 5.5 (1M)`, `Ask`, anillo de contexto al 31 % y `Medium`.

El código se colorea al compilar con Shiki y el tema Dark+, los mismos que usa la app, así que los
colores coinciden sin JavaScript en el navegador.

## Coreografía

### Idea

**La información dispersa se convierte en trabajo visible y comprensible.** Lo organizan dos hilos:
el objeto lima (campo, luego marca, luego campo) y la ventana, que es el mismo objeto desde la
portada hasta el último capítulo.

Vocabulario, y nada más: revelado por máscara, cambio de encuadre, transformación entre elementos
relacionados (Flip), despliegue, escala contenida, continuidad espacial y pausas. Nada de rebotes,
parallax repartido por la página, fundido con desplazamiento en cada sección, letra a letra,
cursores propios, marquesinas ni precargadores.

### Fotogramas clave

| # | Estado | Lo mueve |
| --- | --- | --- |
| 1 | Entrada a 400 ms: la primera línea del titular ya está, la segunda sube; el campo lima se abre desde su borde; una línea de escaneo revela la ventana | Tiempo (CSS) |
| 2 | Portada estable a ~1 s: todo en reposo para leer | Tiempo |
| 3 | Encuadre: la ventana viaja al escenario y crece; la barra lateral sale del encuadre; el campo se estrecha hacia el borde de la ventana | Scroll ligado |
| 4 | Capítulo 01: la marca junto a la petición y sus chips; lo demás al 25 % | Scroll elige, tiempo resuelve |
| 5 | Capítulo 02: el pliegue se abre y los pasos llegan en orden; el foco cae en el primer Run | Tiempo |
| 6 | Capítulo 03: ese Run se abre desde su fila con su salida | Tiempo, o clic |
| 7 | Capítulo 04: la ruta `src/app.tsx` del Edit viaja a la cabecera del fichero en Changes y el diff se despliega | Tiempo (Flip) |
| 8 | Capítulo 05: el trabajo se pliega, la respuesta pasa a primer plano y la marca brilla una vez | Tiempo |

El fotograma 8 es el estado de la portada: la demo cierra el círculo. El cierre de la página es la
coda: el lima vuelve como campo.

### Entrada (tiempo, CSS)

Todo el contenido está en el HTML y es visible sin animación. La entrada son `@keyframes` con
`animation-fill-mode: both`, aplicados solo con `prefers-reduced-motion: no-preference`: si la
animación no corre, el contenido ya está.

| Pieza | Inicio | Duración | Movimiento | Curva |
| --- | --- | --- | --- | --- |
| Estructura y navegación | 0 | — | presentes desde el primer pintado | — |
| Línea del campo lima (`scan`) | 0 ms | 160 ms | una línea de 2 px se dibuja de arriba abajo en el borde izquierdo del campo | `enter` |
| Líneas del titular | 60 ms | 520 ms | suben `0.3em` dentro de su máscara, 70 ms entre líneas | `enter` |
| Campo lima | 160 ms | 560 ms | se abre hacia la derecha desde la línea (`clip-path`) | `enter` |
| Ventana | 300 ms | 600 ms | una línea de escaneo lima la cruza de izquierda a derecha y la revela (`clip-path`); escala de 1,02 a 1 | `enter` |
| Entradilla, cuerpo, CTA y metadatos | 620 ms | 380 ms | aparecen por opacidad, sin desplazamiento | `enter` |

A los 1000 ms todo está quieto. Los solapes son de ~40 %: nada espera a que acabe lo anterior. El
tiempo se ajusta viéndolo en movimiento.

### Encuadre (scroll ligado)

Solo en el recorrido de escritorio. Mientras la portada sale, a lo largo del ~70 % de la altura de
pantalla, con progreso lineal y reversible:

- La ventana pasa de su sitio en la portada al escenario de la demo, con escala de ~0,72 a 1. El
  movimiento se calcula con Flip entre los dos rectángulos.
- La barra lateral de la app sale del encuadre, como con `Ctrl+B`, y quedan el hilo y Changes a
  tamaño real.
- El campo lima se estrecha hacia el borde izquierdo de la ventana hasta ser la marca de 4 px.
- El titular y `sens` siguen el scroll nativo y pasan por detrás de la ventana.

El texto de la ventana se compone a escala 1 y se reduce en la portada, nunca al revés, para que no
quede borroso al crecer. `will-change` solo dura lo que dura el scrub.

### Capítulos (el scroll elige, el tiempo resuelve)

La columna izquierda desplaza los textos de los capítulos; la ventana queda fija a la derecha, con
la marca pegada a su borde. Cada capítulo ocupa ~65 % de la altura de pantalla de scroll. Un
capítulo se activa cuando su texto cruza el centro de la pantalla.

Cada capítulo es un **estado**: pliegue abierto o cerrado, paso en foco, paso desplegado,
relación visible, qué se atenúa y dónde está la marca. Una transición va del estado actual al de
destino: hacia delante con su coreografía y hacia atrás con la inversa. Saltar con el índice de 01
a 05 interpola directamente al estado final, sin reproducir los intermedios.

| Transición | Qué ocurre | Duración | Curva |
| --- | --- | --- | --- |
| → 01 | La marca se asienta junto a la petición; hilo y Changes bajan al 25 % | 500 ms | `move` |
| 01 → 02 | La línea «Worked» se abre; los pasos llegan en orden, 80 ms entre ellos, con el brillo de paso vivo de la app; pausa; la marca viaja al primer Run y el resto se atenúa | ~1,6 s + 400 ms de pausa | `enter`, `move` |
| 02 → 03 | El Run se abre desde su propia fila (altura, no aparición); la marca crece con él | 360 ms | `move` |
| 03 → 04 | El Run se cierra; la ruta `src/app.tsx` del primer Edit viaja a la cabecera del fichero en Changes; el diff se despliega línea a línea | 700 ms + 400 ms | `move`, `enter` |
| 04 → 05 | Los pasos se pliegan en una línea; la respuesta pasa a primer plano; la marca brilla una vez y se queda quieta | 900 ms + 560 ms | `move`, `confirm` |

Mientras un capítulo está activo no se mueve nada: es la pausa para leer. Nada se repite en bucle.

### Cierre (scroll ligado)

Al entrar el cierre, el campo lima se revela por máscara desde una línea, el mismo gesto de la
entrada, en ~40 % de la altura de pantalla. El `sens` gigante ya está, cortado por el pie.

### Elementos compartidos

- **El lima**: campo → marca → campo.
- **La ventana**: la misma de la portada a la demo.
- **Un paso y su detalle**: el detalle se abre desde la fila del paso, sin aparecer de la nada.
- **La ruta de un fichero y su cabecera en Changes**: el mismo elemento de texto viaja (Flip).

### Microinteracciones

Todas usan `fast` (140 ms) o `medium` (220 ms) con la curva `control`, y comparten sistema: fondo,
color, línea e icono. Nunca mueven el área donde se pulsa.

| Control | Reposo → hover | Foco | Pulsado · seleccionado |
| --- | --- | --- | --- |
| Botón principal | `carbon-950` → `carbon-800`; la flecha de descarga baja 2 px dentro del botón | anillo de 2 px `signal-800`, separado 3 px | fondo `carbon-700` |
| Enlace | subrayado de 1 px → 2 px; la flecha avanza 2 px | anillo | — |
| Índice de capítulos | número `alloy-600` → `carbon-950` y fondo `bone-100` | anillo | actual: `carbon-950`, raya de 2 px y `aria-current` |
| Pestañas | como el índice | anillo | la raya de la pestaña activa se desplaza a la nueva (`transform`) |
| Paso de la demo | fondo del paso, como en la app | anillo | se abre desde su fila |
| Menú móvil | icono `menu` → `x` por opacidad | anillo | la hoja se abre por máscara desde arriba; enlaces con 30 ms de escalonado |
| Repetir (móvil) | fondo | anillo | reproduce la tarjeta desde su inicio |

### Por qué scroll o tiempo

El scroll decide dónde estás; el tiempo decide cómo llegas. Va ligado al scroll lo espacial, el
encuadre y el cierre: es continuo, reversible y respeta el desplazamiento nativo sin fijar la
página durante pantallas vacías. Va en tiempo lo que en la app ocurre en el tiempo, como los pasos
que llegan, un paso que se abre o un cambio que se relaciona con su paso: arrastrar texto con el
scroll lo vuelve mecánico. El lector marca el ritmo, porque cada capítulo espera en su estado hasta
que llega el siguiente.

## Móvil y tamaños intermedios

El recorrido de escritorio exige al menos 1024 px de ancho y 640 px de alto. Por debajo, o en móvil
en horizontal, se usa este:

- **Portada**: titular en cuatro líneas; CTA a todo el ancho y 56 px de alto; debajo, un bloque
  lima con la app recortada a la columna del hilo a tamaño real, no la ventana entera encogida;
  `sens` gigante a sangre.
- **Demo**: sin escena fija. Cada capítulo es un texto seguido de una tarjeta oscura con el recorte
  que le toca: petición y chips; pasos; Run abierto; Edit y, debajo, la tarjeta de Changes; y
  resultado. Cada tarjeta reproduce su transición una vez al quedar a la vista (1,2 s como
  máximo) y tiene un botón de repetir de 44 × 44. La marca lima va en el borde izquierdo de la
  tarjeta. En el 04 la ruta viaja en vertical hasta la cabecera del diff.
- **Ventajas**: apiladas; las pestañas pasan a un control segmentado con desplazamiento horizontal.
- **Menú**: un `<details>` que funciona sin JS; con JS se cierra con `Esc` y al tocar fuera.
- Se prueba en 375 × 812, 390 × 844, 768 × 1024, 1024 × 768, 844 × 390, 1280 × 720 y 1440 × 900,
  y al redimensionar en caliente.

## Movimiento reducido

Con `prefers-reduced-motion: reduce`, o con `?motion=reduce` para pruebas, se conservan el
contenido, el orden y las relaciones, y se quitan viajes, escalas y máscaras:

- Sin entrada: todo presente desde el primer pintado.
- La ventana de la portada se queda en la portada. El escenario de la demo está en su sitio con
  la marca al lado.
- El scroll sigue eligiendo el capítulo, pero el cambio es inmediato o con un fundido de opacidad
  de 120 ms como máximo. La marca salta. En el 04, la ruta del paso y la cabecera del fichero se
  resaltan a la vez con un contorno lima, sin viajar.
- Las tarjetas de móvil se muestran en su estado final, sin botón de repetir.
- Las microinteracciones conservan color y fondo, sin desplazamientos.

Nada se mueve solo durante más de 5 s: la entrada dura 1 s y cada capítulo o tarjeta menos de 2 s.
Por eso no hace falta un control de pausa (WCAG 2.2.2). Si en las pruebas algo supera el umbral,
llevará su pausa.

## Sin JavaScript

- Todo el texto está en el HTML y la entrada es CSS que no oculta nada de antemano.
- La demo muestra su estado final, el de la portada, y los capítulos como lista ordenada; el
  índice son anclas.
- Las pestañas de ventajas se ven como bloques apilados con su título.
- La descarga y los enlaces son enlaces normales.

## Sistema de movimiento

En `src/motion/tokens.ts`. El build los escribe también como variables CSS, de modo que CSS y GSAP
leen los mismos valores.

| Token | Valor | Uso |
| --- | --- | --- |
| `fast` | 140 ms | hover, foco, pulsación (el `--fast` de la app) |
| `medium` | 220 ms | pestañas, menú, índice |
| `scene` | 600 ms | cambio de capítulo |
| `entrance` | 1000 ms en total | la entrada |
| `confirm` | 560 ms | el brillo `done` (el `motion.confirm` de la marca) |
| `control` | `cubic-bezier(.2,.6,.3,1)` | controles (la curva de la app) |
| `enter` | `cubic-bezier(.16,1,.3,1)` | lo que entra |
| `exit` | `cubic-bezier(.7,0,.84,0)` | lo que sale, siempre breve |
| `move` | `cubic-bezier(.65,0,.35,1)` | viajes entre elementos (Flip, marca) |
| `linked` | lineal | todo lo ligado al progreso del scroll |
| `shift` | `0.3em` | desplazamiento corto del texto |
| `scale` | 1,02 | escala contenida de entrada |
| `stagger` | 70 ms en líneas, 80 ms en pasos, 30 ms en enlaces | escalonado |
| `overlap` | 40 % | solape entre piezas de una secuencia |

No hay resortes: `identity.md` §8 prohíbe las curvas elásticas y ningún movimiento de esta web
necesita una sensación física.

## Arquitectura

```
sens-web/
  CLAUDE.md
  astro.config.ts
  package.json
  scripts/
    brand.ts              tokens.css, marca, wordmark y fuente desde P:\Sens
    release.ts            versión, tamaño y enlace del instalador desde GitHub
  src/
    brand/                (generado) mark.svg, wordmark.svg
    data/release.json     (generado, con el último dato bueno)
    content/session.ts    la sesión de la demo, fuente única
    layouts/Base.astro
    pages/index.astro
    sections/             Nav, Hero, Demo, Benefits, Engine, Close, Footer
    product/              Window, Topbar, Rail, Thread, Step, Output, Answer, Composer, Changes, Diff
    motion/
      tokens.ts
      boot.ts             preferencias, puntos de corte (gsap.matchMedia) y arranque
      reframe.ts          portada → escenario
      stage.ts            estados de la escena y transiciones entre ellos
      marker.ts           la marca lima
      chapters.ts         capítulo activo según el scroll e índice
      cards.ts            tarjetas de móvil
      close.ts            revelado del cierre
    styles/
      tokens.css          (generado)
      base.css  type.css  layout.css  motion.css
  public/fonts/           Space Grotesk y Geist Mono recortadas, con OFL.txt
  test/
```

- **`npm run brand`** importa `brandTokensCss()`, `markSvg()` y `sensPath` de `P:\Sens\src\brand`
  y copia `space-grotesk.woff2` con su `OFL.txt`. Lo generado se guarda en el repo, así que la web
  se construye sola, y no se edita a mano.
- **`npm run release`** lee la última release de `beyondhumane/sens-ai` y guarda versión, fecha,
  tamaño y enlace directo del `.exe` en `release.json`. Si la API falla conserva el último dato
  bueno, y en último caso los botones van a `releases/latest`.
- **Responsabilidades**: CSS para la entrada, las microinteracciones y el modo reducido; GSAP con
  ScrollTrigger y Flip para el scroll, la escena y los viajes; IntersectionObserver para pausar lo
  que sale de pantalla. Ninguna otra librería de animación. GSAP es gratuito, también para uso
  comercial, pero con licencia propia, no OSI; el resto de dependencias son de código abierto.
- Los componentes marcan con atributos `data-*` lo que la capa de movimiento necesita: pasos,
  rutas, cabeceras y destinos de la marca. El JavaScript no duplica textos ni estructura.
- La capa de movimiento es un único módulo diferido. Si falla, la página es la versión sin JS.

## Accesibilidad

Objetivo WCAG 2.2 AA, comprobado, no solo declarado.

- `header`, `nav`, `main`, `section` y `footer`; un solo `h1`; un `h2` por sección; enlace para
  saltar al contenido; `lang="en"`.
- Teclado en todo: el índice y las pestañas siguen el patrón WAI-ARIA (flechas, `Home`, `End`); el
  menú móvil lleva `aria-expanded`. Los destinos tienen `scroll-margin-top` para que la
  navegación fija no tape el foco (2.4.11).
- Foco visible: 2 px `signal-800` sobre paper y `signal-500` sobre oscuro.
- Objetivos de 44 × 44 en los controles principales; ninguno baja de 24 × 24 (2.5.8).
- Los contrastes de cada pareja de colores usada se calculan en un test.
- La escena de la demo es `inert` y `aria-hidden`: es una repetición, no una sesión. Cada capítulo
  lleva una frase para lectores de pantalla con lo que muestra la escena.
- El enlace de descarga dice qué baja y cuánto pesa. Los enlaces externos lo indican.
- El color nunca es la única señal de un estado.

## Rendimiento

Presupuestos que se miden, no que se prometen:

- JavaScript ≤ 70 KB con gzip (GSAP, ScrollTrigger y Flip son unos 50 KB); CSS ≤ 30 KB.
- Dos fuentes recortadas a latín; la principal con `preload`.
- Solo se animan `transform` y `opacity`. `clip-path` se reserva a la entrada y los revelados. Sin
  filtros ni blur.
- Tamaños reservados en todo recurso, para que el CLS sea ~0. El LCP es el titular, que es texto.
- Imágenes, si las hay, en AVIF o WebP, con dimensiones y carga diferida bajo el primer pantallazo.
- Las líneas de tiempo fuera de pantalla se pausan; no hay bucles de `requestAnimationFrame`.
- Fluidez comprobada con la CPU ralentizada ×4, midiendo los fotogramas largos durante el scroll.

## Verificación

- `astro check` y TypeScript estricto.
- Vitest para la lógica pura: capítulo según el progreso de scroll, estados de la escena, tokens a
  CSS, contrastes y lectura de la release con su caída.
- En el navegador:
  - las siete medidas de *Móvil y tamaños intermedios*, con redimensionado en caliente;
  - recorrido con teclado de toda la página;
  - axe sobre la página;
  - `?motion=reduce` y una compilación sin JS;
  - red lenta, comprobando que nada queda oculto ni salta mientras carga;
  - fotogramas clave capturados deteniendo las líneas de tiempo, y la fluidez medida con la CPU
    ralentizada.
- Lighthouse con Edge si está disponible. Solo se citan cifras medidas.
- Todos los enlaces y descargas apuntan a destinos reales.

## Etapas

1. **Prototipo de motion**, con parada para revisarlo en movimiento:
   - andamiaje, `brand` y `release`;
   - navegación con el botón de descarga como microinteracción;
   - portada en escritorio y móvil, con su entrada;
   - la ventana reconstruida;
   - el encuadre hacia la demo;
   - los cinco capítulos como estados, con la coreografía completa de 02, 03 y 04 y el índice;
   - las tarjetas de móvil de esos tres capítulos;
   - movimiento reducido.
2. **Web completa**: la coreografía de 01 y 05, ventajas, bajo la ventana, cierre y pie, imagen
   para compartir y la revisión completa: claridad del mensaje, fidelidad al producto, tipografía,
   coherencia del movimiento, lectura entre transiciones, saltos de layout, teclado y táctil,
   redimensionado, carga lenta, movimiento reducido, enlaces reales y rendimiento observado.

## Fuera de alcance

Alojamiento y dominio (la salida es estática y sirve para cualquier hosting), analítica, otros
idiomas, el Canon, un blog o documentación.
