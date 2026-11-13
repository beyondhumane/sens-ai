# El rol de diseño: el juego tiene un diseño y el agente lo sigue

Primer rol del giro de Sens a desarrollo de videojuegos. El otro rol escrito
hasta hoy, la dirección técnica, está en
`2026-11-13-sens-unity-canon-design.md`.

## La visión

> **Sens es un estudio de videojuegos en un solo agente: diseña con criterio
> de género, programa con criterio de Unity y no se aparta del juego que
> decidiste hacer sin preguntártelo.**

La tesis de Sens no cambia. Las instrucciones escritas son consejos que el
modelo puede no cargar o no seguir; Sens las convierte en obligaciones
controlando lo que el modelo ve, lo que se le deja hacer y cuándo se le deja
terminar. Lo que cambia es la jurisdicción: ya no vigila solo el código, sino
el juego entero.

Un estudio tiene roles: diseño, dirección técnica, producción, contenido y
calidad. Este documento cubre el primero. Esta frase es la fuente de verdad
para el README, la web, la identidad y el paper cuando toque reescribirlos.

## Para qué

Claude sabe mucho de diseño de juegos. Si le preguntas qué hace bueno el bucle
de un roguelike, te lo explica bien. Lo que no hace es aplicarlo con
constancia:

- no guarda el diseño del juego como un compromiso entre una sesión y otra;
- no comprueba cada funcionalidad contra los pilares del juego;
- no trae las convenciones del género si no se las pides;
- no avisa cuando un cambio va contra lo que el juego quiere ser.

Este rol resuelve las cuatro cosas con la maquinaria que Sens ya tiene: el
contexto que el hook de cada mensaje pone delante del modelo, las
herramientas MCP que sirve el propio Sens y la parada al cerrar el turno.

## El documento de diseño

Vive en `Design/`, una carpeta en la raíz del proyecto, junto a `Assets/`. Es
Markdown: lo editan la persona y el agente, viaja con el juego y git lo
versiona. No va en `.sens/` porque `.sens/` se queda fuera de git, y el diseño
lo tiene que ver el equipo entero.

Es una carpeta y no un fichero único para que cada mecánica sea un fichero y
los diffs se lean bien.

| Fichero | Qué dice |
| --- | --- |
| `Design/README.md` | Pitch en una línea, fantasía del jugador, plataforma y público. El frontmatter declara los subgéneros |
| `Design/pillars.md` | Entre 3 y 5 pilares, cada uno como un encabezado `##` con una frase y una sección de lo que descarta |
| `Design/loop.md` | El bucle principal en tres escalas: segundos, minutos y horas |
| `Design/mechanics/*.md` | Un fichero por mecánica o sistema |

Solo esos cuatro tienen estructura fija. Cualquier otro `.md` dentro de
`Design/` (economía, progresión, narrativa) se divide por encabezados y entra
en la búsqueda igual que el resto.

### Subgéneros

El frontmatter de `Design/README.md` lleva `subgenres:` con uno o más
identificadores. Cada uno activa el manual que lo contiene:

| Manual | Subgéneros |
| --- | --- |
| Acción 2D y plataformas | `platformer`, `metroidvania`, `action-2d` |
| Roguelike y deckbuilders | `roguelike`, `roguelite`, `deckbuilder` |
| Acción 3D y shooters | `fps`, `tps`, `action-3d` |
| RPG y estrategia | `rpg`, `action-rpg`, `tactics`, `strategy` |

Un identificador que Sens no conoce se ignora y se dice una vez en el chat: no
rompe nada y tampoco pasa en silencio.

### Pilares

Cada pilar es un encabezado `##` de `pillars.md`, y su identificador es el
slug de ese encabezado. Bajo el encabezado va una frase y una sección
`### Descarta`. Un pilar que no descarta nada no sirve para decidir, y esa es
la diferencia entre un pilar de verdad y un eslogan. El lector de `Design/`
avisa de un pilar sin `Descarta`; no lo rechaza. El encabezado se reconoce en
los seis idiomas de Sens: `Descarta`, `Rules out`, `Exclut`, `Schließt aus`,
`除外` y `排除`.

### Mecánicas

Cada fichero de `Design/mechanics/` lleva en el frontmatter:

- `pillars:` los identificadores de los pilares a los que sirve;
- `status:` uno de `idea`, `prototype`, `in-game` o `cut`.

El cuerpo es libre. Lo habitual es que tenga sus reglas y sus números.

## La primera vez: la entrevista de diseño

Cuando se abre un proyecto de Unity sin `Design/`, Sens ofrece una entrevista,
sin imponerla. El agente pregunta de una en una, carga el manual del
subgénero en cuanto lo sabe y escribe `Design/` con la estructura de arriba.

Si el proyecto ya tiene código, la entrevista empieza al revés: el agente lee
lo construido, propone un borrador del diseño que se deduce de ahí y pide a la
persona que confirme o corrija los pilares antes de escribir nada.

Es una sesión con un briefing propio. `settings.briefing` ya se añade al
prompt de sistema en `chat.rs`; la entrevista es otro texto en ese mismo
campo, con su propio presupuesto de líneas y su test.

## Los manuales de género

Cuatro ficheros Markdown dentro de Sens, en
`rust/sens-canon/src/design/genres/`, incluidos con `include_str!` como el
Canon. Están en inglés para que encajen con el vocabulario del código, y el
glosario hace de puente con los seis idiomas de las peticiones.

Cada manual se divide en **temas** con encabezados `##` (movimiento, cámara,
combate, economía…). Un tema que solo vale para algunos subgéneros lo dice en
una línea `Subgenres:` justo bajo el encabezado; sin esa línea, vale para
todos los subgéneros del manual.

Cada tema tiene exactamente estas cinco partes, como encabezados `###`:

| Parte | Qué lleva |
| --- | --- |
| `Expectations` | Las convenciones que el jugador espera. Romper una tiene que ser una decisión de diseño, no un descuido |
| `Systems` | Los sistemas que siempre están y su arquitectura habitual en Unity. Aquí el diseño enlaza con el código |
| `Numbers` | Rangos, nunca valores mágicos. Cada viñeta termina con su fuente como enlace: una charla, un postmortem o un valor publicado por quien hizo el juego |
| `Pitfalls` | Errores conocidos, cada uno con su porqué |
| `References` | Juegos publicados que lo resuelven bien y en qué hay que fijarse exactamente |

Un test vigila la forma de los cuatro manuales:

- cada tema tiene las cinco partes, en ese orden;
- cada viñeta de `Numbers` contiene un enlace;
- ningún tema pasa de 60 líneas;
- cada `Subgenres:` nombra identificadores del propio manual.

El test garantiza la estructura. La calidad del contenido la revisa el equipo
manual a manual, y eso no lo puede garantizar ningún test.

El primer manual que se escribe, el de acción 2D y plataformas, es la
plantilla de los otros tres.

## Lo que ve el agente en cada mensaje

En `prompt` de `rust/sens-agent/src/canon/circuit.rs`, después del Canon, el
mapa y el código relacionado:

- **Los pilares**, una vez por sesión, y otra vez solo si `pillars.md` ha
  cambiado. Es el mismo mecanismo con el que `state.canon` evita repetir el
  Canon, usando el hash del fichero en lugar de la versión.
- **Hasta tres secciones** de `Design/` y de los temas de los manuales
  activos, ordenadas por relevancia para la petición, con un tope de 80
  líneas entre todas.

El chat lo enseña como hoy enseña el código anticipado, con el mismo evento
`Canon` en la etapa `anticipated`: *Diseño: pilar Movimiento preciso ·
mecánica Dash · manual: Plataformas › Salto*.

Hay dos herramientas MCP nuevas en `rust/sens-agent/src/canon/tools.rs`:

- `design`: los pilares, el bucle y la lista de mecánicas con su estado;
- `design_search`: las secciones de `Design/` y de los manuales activos que
  mejor responden a una consulta.

### El texto del Canon

El Canon gana una sección de diseño, incluida solo cuando existe `Design/`,
con su propio presupuesto de líneas y su test. Dice tres cosas:

- `Design/` es la fuente de verdad sobre qué juego se está haciendo;
- cuando el trabajo añade o cambia una mecánica, se actualiza su fichero;
- cuando una petición choca con un pilar, se dice antes de escribir.

## La relevancia

`rust/sens-canon/src/relevant.rs` ya ordena símbolos del índice con BM25,
stemming y un glosario multilingüe que conecta una petición en español con
código en inglés. Su `Catalog` está atado a símbolos, pero el núcleo no.

Se extrae ese núcleo (documentos como términos con peso, puntuación BM25
contra los conceptos de la petición) para que lo usen dos catálogos: el de
símbolos, que sigue haciendo lo mismo, y uno nuevo de secciones de texto. No
se escribe un segundo BM25.

`glossary.toml` se amplía con vocabulario de videojuegos en los seis idiomas
de Sens, para que "el salto" encuentre `Jump` y "las oleadas" encuentre
`Wave spawning`.

## La parada al cerrar el turno

Las dos reglas preguntan, nunca deniegan.

**D1, determinista (`Ask`).** El turno añade una clase que deriva de
`MonoBehaviour` o `ScriptableObject` dentro de `Assets/`, fuera de cualquier
carpeta llamada `Editor` o `Tests`, y ningún fichero de
`Design/` ha cambiado en el turno. El mensaje nombra las clases y pregunta si
son una mecánica que describir en `Design/mechanics/` o no lo son. Si la
persona dice que no, la respuesta se guarda como excepción con la clave
`D1:<Clase>`, con el mecanismo de `Exceptions` que ya existe, y no se vuelve a
preguntar por esa clase.

D1 necesita saber qué deriva de `MonoBehaviour`, y eso es el hito 1 del plan
de Unity: la herencia cerrada dentro del proyecto. Por eso ese hito va
primero.

**S8, del revisor.** El turno va contra un pilar. Es una cuestión de criterio,
así que la juzga el revisor de cierre de `review.rs`. Cada hallazgo de S8 tiene
que citar al pie de la letra una línea añadida del diff y una línea de
`pillars.md`; si cualquiera de las dos citas no aparece tal cual, el hallazgo
se descarta. El paper mide que el revisor automático acierta poco, así que S8
**empieza como `Note`** y sube a `Ask` cuando se haya calibrado sobre un
proyecto real.

## Dónde vive el código

| Pieza | Dónde |
| --- | --- |
| Lector de `Design/`, catálogo de secciones, manuales, D1 | Un módulo `design` en `sens-canon`. El diseño pasa a formar parte de lo que juzga el Canon |
| Núcleo BM25 | Extraído dentro de `sens-canon`, usado por `relevant.rs` y por `design` |
| Contexto en cada mensaje | `sens-agent`, `canon/circuit.rs` |
| `design` y `design_search` | `sens-agent`, `canon/tools.rs` |
| S8 | `sens-agent`, `canon/review.rs`, y la lista de reglas de `sens-canon` |
| Entrevista | El texto en `sens-canon`, junto al Canon; el arranque de la sesión en `sens-app` |

## Tests

- El lector de `Design/`: subgéneros del frontmatter, pilares con su
  identificador y su `Descarta`, mecánicas con su estado, un subgénero
  desconocido avisado una vez y un pilar sin `Descarta` avisado sin
  rechazarse.
- La relevancia: "añade un dash" encuentra `Design/mechanics/dash.md` y el
  tema de movimiento del manual de plataformas, y "el salto" encuentra un
  tema llamado `Jump`.
- El presupuesto: nunca más de tres secciones ni más de 80 líneas, y los
  pilares no se repiten si `pillars.md` no ha cambiado.
- D1: una clase nueva sin tocar `Design/` pregunta; con `Design/` tocado no
  pregunta; dentro de una carpeta `Editor` o `Tests` no pregunta; con la
  excepción guardada no pregunta; y una clase que deriva de un
  `MonoBehaviour` del proyecto en otro fichero también pregunta.
- S8: un hallazgo cuya cita del diff o del pilar no aparece tal cual se
  descarta.
- La forma de los manuales, como se describe arriba.
- Los presupuestos de líneas de la sección de diseño del Canon y del briefing
  de la entrevista.

## Orden

1. El hito 1 del plan de Unity: puntos de entrada y herencia.
2. Extraer el núcleo BM25, el catálogo de secciones y el glosario ampliado.
3. El lector de `Design/` y la sección de diseño del Canon.
4. El contexto en cada mensaje y las dos herramientas.
5. D1.
6. La entrevista, para proyectos nuevos y para los que ya existen.
7. El manual de acción 2D y plataformas, con su test de forma.
8. Los manuales de roguelike y deckbuilders, acción 3D y shooters, y RPG y
   estrategia.
9. S8 como `Note`, y calibrarla.

Los hitos 2 a 7 del plan de Unity, las reglas de código, quedan como el rol de
dirección técnica, para después de esta entrega.

## Fuera de esta entrega

- Una vista de diseño propia en la app: `Design/` ya se ve en el panel de
  ficheros.
- Una casilla para apagar el rol: si existe `Design/`, está activo.
- El alcance y los hitos del juego, que son el rol de producción.
- Comparar los números que cambia el código con los que declara el diseño.
