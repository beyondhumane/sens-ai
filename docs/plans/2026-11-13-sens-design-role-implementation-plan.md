# El rol de diseño — Plan de implementación

- **Fecha:** 2026-11-13
- **Diseño de referencia:** [../specs/2026-11-13-sens-design-role-design.md](../specs/2026-11-13-sens-design-role-design.md)
- **Estado:** Hitos 1 a 10 hechos. Las revisiones de los manuales las hizo el agente a petición de la persona, no alguien que haya publicado un juego del género. Del 7 falta la prueba a mano: una entrevista de punta a punta con Claude Code, en un proyecto vacío y en uno con código. Del 10 falta pasar una vez la prueba en vivo de S8 contra el revisor real (`live_review`, ignorada por defecto): en esta máquina la sesión del CLI de `claude` había caducado.

Cada hito se prueba por separado y deja el proyecto en un estado defendible.
El orden sigue las dependencias: primero lo que el índice tiene que saber,
después la maquinaria que lee el diseño y se lo pone delante al agente, luego
la parada, la entrevista y, al final, el contenido de los manuales. Los
manuales son lo que más trabajo da y lo que más importa, pero sin la
maquinaria no llegan a ningún sitio.

El hito 0 del plan de Unity, que hace que el índice ignore lo que genera el
editor y detecte un proyecto de Unity con `is_unity`, ya está hecho. La
entrevista lo usa.

---

## Decisiones que el diseño dejaba abiertas

| Área | Decisión | Motivo |
|---|---|---|
| Qué pasa cuando se contesta "no" a D1 | La pregunta se formula de forma que "sí" significa "puede quedarse sin describir" y guarda la excepción, y "no" significa "descríbela": el turno se retiene con esa instrucción **y no se deshace nada** | Hoy `Ask` deshace el cambio cuando la persona dice que no (`circuit.rs`, `weigh`), porque se pensó para dependencias. Con D1, deshacer la clase sería lo contrario de lo que se busca |
| Frontmatter de `Design/` | Se lee a mano: `clave: valor` y `clave: [a, b]`, y nada más. El lector avisa de cualquier otra forma | No hay ningún crate de YAML en el proyecto, y tres claves no justifican una dependencia |
| Lo anticipado en el chat | `Event::Canon` gana un campo `design` con título y fichero de cada sección. `suggestions` sigue siendo solo de símbolos | `Suggested` tiene forma de símbolo (firma, usos); meter ahí una sección de texto sería forzarlo |
| Dónde va el texto de diseño del Canon | En el hook de cada mensaje, junto a los pilares, y no en el saludo de `chat.rs` | Los dos dependen de que exista `Design/`, y el hook ya conoce el proyecto; el saludo no |
| Pilares repetidos | El circuito recuerda en memoria el último texto de diseño que mandó, igual que hace con el mapa, y lo olvida al compactar (`forget`) | Si `pillars.md` cambia a mitad de sesión el agente tiene que verlo, y después de compactar ya no lo tiene delante |

---

## Hito 1 — La herencia de Unity en el índice

**Objetivo:** que el índice sepa qué clases derivan de `MonoBehaviour` o
`ScriptableObject`, aunque la base esté en otro fichero.

Es el hito 1 del plan de Unity
([2026-11-13-sens-unity-canon-implementation-plan.md](2026-11-13-sens-unity-canon-implementation-plan.md)),
tal como está escrito allí. D1 necesita además poder preguntar por esa
herencia, así que el pase posterior en `build::build` deja la respuesta en el
índice y no solo la marca `entry` en los mensajes de Unity.

**Se prueba:** lo que dice el plan de Unity, y además que
`BossController : PlayerController : MonoBehaviour`, repartidos en dos
ficheros, responden los dos como derivados de `MonoBehaviour`.

---

## Hito 2 — Un solo BM25 para símbolos y secciones

**Objetivo:** que la relevancia que hoy ordena código pueda ordenar texto sin
que exista un segundo BM25.

- Extraer de `rust/sens-canon/src/relevant.rs` el núcleo a
  `rust/sens-canon/src/rank.rs`: documentos como mapas de términos con peso,
  la expansión de conceptos con el glosario y la puntuación con `K1` y `B`.
  `Catalog` pasa a usarlo y su comportamiento no cambia.
- Ampliar `glossary.toml`, con el mismo formato `english = [...]`, con
  vocabulario de videojuegos en los seis idiomas de Sens: jump, dash, wave,
  spawn, enemy, player, level, camera, health, damage, inventory, deck, card,
  run, upgrade, loot, quest, ability, cooldown, combo, boss, checkpoint,
  respawn y los que salgan al escribir el primer manual.

**Se prueba:** los tests de `relevant.rs` pasan sin tocarlos, que es la prueba
de que es un refactor puro. Los ejemplos `recall` y `relevant` de
`sens-canon` dan la misma salida antes y después. Y "el salto" encuentra un
documento titulado `Jump` y "las oleadas" uno titulado `Wave spawning`.

---

## Hito 3 — El lector de `Design/`

**Objetivo:** convertir la carpeta en datos con los que se pueda trabajar.

- Un módulo `design` en `sens-canon`, con `Design::read(root)`, que devuelve:
  subgéneros, pilares (identificador, frase y si tiene `Descarta`), bucle,
  mecánicas (fichero, pilares y estado), las secciones de cada `.md`
  divididas por encabezados, y los avisos.
- El frontmatter, leído a mano como se decidió arriba.
- Un subgénero desconocido o un pilar sin `Descarta` dan un aviso, no un
  error. Si `Design/` no existe, el resultado es `None` y el rol queda
  apagado.

**Se prueba:** los casos del lector que lista el spec, y que una carpeta
`Design/` con un fichero roto sigue leyéndose entera excepto ese fichero, que
queda avisado.

---

## Hito 4 — Lo que ve el agente en cada mensaje

**Objetivo:** que el agente empiece cada petición con el diseño relevante
delante.

- El texto de diseño del Canon: `rust/sens-canon/src/design/canon.md`,
  incluido con `include_str!`, con presupuesto de líneas y un test que
  comprueba sus tres afirmaciones.
- En `prompt` de `rust/sens-agent/src/canon/circuit.rs`, si existe
  `Design/`: el texto de diseño y los pilares una vez por sesión, y otra vez si
  cambia el hash de `pillars.md`. Después, hasta tres secciones de `Design/` y
  de los manuales activos, con un tope de 80 líneas entre todas, ordenadas con
  `rank.rs`.
- El catálogo de secciones se construye junto al índice, en el mismo sitio
  que hoy guarda el proyecto en memoria, y se refresca cuando cambia algo en
  `Design/`.
- `Event::Canon` gana el campo `design`. En la app, `SensStep.tsx` lo pinta
  junto al código anticipado, con texto en los seis idiomas, y
  `mock-tauri.ts` lo devuelve.

En este hito todavía no hay manuales, así que solo se anticipa `Design/`. Los
manuales entran solos en el hito 8, porque el catálogo ya los contempla.

**Se prueba:** el presupuesto (tres secciones, 80 líneas); los pilares una
sola vez por sesión y otra vez si `pillars.md` cambia; un proyecto sin
`Design/` recibe exactamente el contexto que recibía antes; y el test de la
app en jsdom pinta la línea de diseño.

---

## Hito 5 — Las herramientas `design` y `design_search`

**Objetivo:** que el agente pueda consultar el diseño cuando quiera, además de
lo que Sens le pone delante.

- `rust/sens-agent/src/canon/tools.rs`: `NAMES` pasa de 9 a 11, `listed`
  declara las dos con `described`, y el despacho las atiende.
- `design` devuelve los pilares, el bucle y las mecánicas con su estado, en
  una línea cada una. Sin `Design/`, dice que no hay diseño y cómo empezarlo.
- `design_search` devuelve las secciones que mejor responden a una consulta,
  con el mismo presupuesto que el contexto de cada mensaje.

**Se prueba:** la salida de las dos sobre un `Design/` de prueba, y la
respuesta de las dos sin `Design/`.

---

## Hito 6 — D1

**Objetivo:** que el juego no se separe del diseño sin que nadie lo decida.

- `Rule::D1` en `verdict.rs`, evaluada en `judge_turn`. Necesita ver todos los
  cambios del turno a la vez para saber si alguno tocó `Design/`.
- Usa la herencia del hito 1. Ignora cualquier carpeta llamada `Editor` o
  `Tests`. La clave de la excepción es `D1:<Clase>`.
- `weigh`, en `circuit.rs`: con un "no" a D1 no se llama a `undo`. El turno se
  retiene con la instrucción de describir la clase en `Design/mechanics/` y
  volver a terminar.
- La app pregunta con una herramienta nueva, `sens.design`, junto a
  `sens.dependency`, con su texto en los seis idiomas en `Ask.tsx`. La
  pregunta deja claro qué significa cada respuesta.

**Se prueba:** los cinco casos de D1 del spec, y que un "no" deja la clase en
el disco y retiene el turno.

---

## Hito 7 — La entrevista de diseño

**Objetivo:** que nadie tenga que escribir `Design/` desde cero.

- El texto en `rust/sens-canon/src/design/interview.md`, con presupuesto de
  líneas. Cubre los dos modos (proyecto nuevo y proyecto con código), las
  preguntas de una en una, la estructura exacta de `Design/` y los
  identificadores de subgénero válidos.
- En la UI de `sens-app`: la pantalla de nueva sesión ofrece la entrevista
  cuando el proyecto tiene `ProjectSettings/ProjectVersion.txt` y no tiene
  `Design/`, comprobado con el comando `folder` que ya existe, y con texto en
  los seis idiomas. Al aceptarla, el panel queda marcado como entrevista, sus
  `Settings` llevan `interview: true` en cada mensaje y `greeting` añade el
  texto al prompt de sistema. La marca se borra al cambiar de sesión. La Rust
  de `sens-app` no se toca.

**Se prueba:** el presupuesto del texto; que la app solo la ofrece en un
proyecto de Unity sin `Design/` (jsdom); que la sesión arranca con los dos
briefings; y, a mano, una entrevista de punta a punta en un proyecto vacío y
otra en uno con código. Lo que salga tiene que pasar el lector del hito 3 sin
avisos.

---

## Hito 8 — El manual de acción 2D y plataformas

**Objetivo:** el primer manual, que sirve de plantilla para los otros tres.

- Primero el test de forma, sobre todos los ficheros de
  `rust/sens-canon/src/design/genres/`: las cinco partes en orden, un enlace
  en cada viñeta de `Numbers`, ningún tema de más de 60 líneas, y cada
  `Subgenres:` con identificadores del propio manual.
- Después `action-2d.md`. Temas: carrera (aceleración y frenada), salto
  (altura variable, coyote time, buffer, gravedad en el ápice, velocidad
  máxima de caída), cámara (anticipación y zonas muertas), colisión y
  sensación (corrección de esquinas, plataformas de un sentido), combate 2D
  (hitstop, retroceso, invulnerabilidad), mapa y progresión de metroidvania
  (habilidades que abren camino, vuelta atrás, mapa) y diseño de niveles
  (enseñar, poner a prueba, dar la vuelta; legibilidad).
- Cada número se investiga y se enlaza con su fuente. Ninguno se escribe de
  memoria.

**Se prueba:** el test de forma, y que "añade doble salto" anticipa el tema
del salto. El hito no se cierra hasta que el equipo haya revisado el manual:
el test garantiza la forma, no que el contenido sea bueno.

---

## Hito 9 — Los otros tres manuales

**Objetivo:** `roguelike.md`, `action-3d.md` y `rpg-strategy.md`, con la
plantilla del hito 8.

Se escriben uno a uno, en ese orden, y cada uno pasa el test de forma y la
revisión del equipo antes de empezar el siguiente.

**Se prueba:** lo mismo que el hito 8, con una prueba de relevancia por
manual: "añade un mazo inicial" en un deckbuilder, "ajusta el retroceso" en un
shooter y "curva de experiencia" en un RPG.

---

## Hito 10 — S8 como `Note`

**Objetivo:** que el revisor señale cuándo un turno va contra un pilar, sin
parar nada hasta que haya demostrado que acierta.

- `rust/sens-canon/src/review.rs`: `RULES` pasa de 7 a 8 y `BRIEF` describe
  S8. Cuando existe `Design/`, el prompt del revisor incluye `pillars.md`.
- Cada hallazgo de S8 trae dos citas: una línea añadida del diff y una línea
  de `pillars.md`. La validación que hoy comprueba la cita del diff comprueba
  también la del pilar. Si cualquiera de las dos no aparece tal cual, el
  hallazgo se descarta.
- La severidad es `Note`.

**Se prueba:** un hallazgo con la cita del pilar inventada se descarta, y uno
con las dos citas literales se queda como `Note`.

---

## Hito 11 — Calibrar S8 y promoverla

**Objetivo:** convertir S8 en `Ask` con una medida detrás.

Igual que el hito 7 del plan de Unity: se trabaja con un proyecto real que
tenga `Design/`, se recogen todos los S8, se revisan a mano y S8 sube a `Ask`
solo si no da falsos positivos. Necesita un proyecto real; sin él, S8 se queda
como `Note`.

---

## Lo que no entra

Una vista de diseño en la app, una casilla para apagar el rol, el alcance y
los hitos del juego (que son el rol de producción) y comparar los números del
código con los del diseño. Las reglas de código de Unity, de los hitos 2 a 7
de su plan, vienen después de este plan.

## Los riesgos

**El ruido.** Si D1 pregunta por cada clase pequeña, la gente acabará
contestando "sí" sin leer, y la parada dejará de servir. Lo frenan tres
cosas: la excepción por clase, las carpetas `Editor` y `Tests` excluidas, y
que solo pregunta cuando el turno no ha tocado `Design/`. Si en la práctica
pregunta demasiado, se estrecha la condición; la excepción no se quita. Lo
mismo vale para el contexto: el presupuesto de líneas es fijo y tiene su test.

**Un manual que se equivoca con seguridad.** Es peor que no tener manual,
porque el agente se lo cree y lo repite con autoridad. Lo frenan tres cosas:
que cada número lleve su fuente, que el equipo revise cada manual antes de
cerrarlo y que se empiece por uno solo, que fija el listón de los demás.
