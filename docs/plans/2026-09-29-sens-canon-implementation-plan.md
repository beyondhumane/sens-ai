# El Canon — Plan de implementación

- **Fecha:** 2026-09-29
- **Diseño de referencia:** [../specs/2026-09-29-sens-canon-design.md](../specs/2026-09-29-sens-canon-design.md)
- **Rama:** `feat/canon`
- **Estado:** Fases 0 a 3 hechas; fase 4 construida y probada sin modelo, falta medirla

Ocho fases. Cada una termina con algo que funciona, sus pruebas en verde y un
commit. La fase 0 es una puerta: si alguna comprobación falla, se corrige la spec
antes de seguir. Las fases 1, 4, 5 y 7 ejecutan el modelo real y gastan cuota del
plan: se piden antes de lanzarlas.

## Cómo se trabaja cada tarea

1. Escribir la prueba que falla.
2. Implementar lo mínimo que la pasa, reutilizando lo que ya existe.
3. `cargo clippy --manifest-path <crate>/Cargo.toml --all-targets -- -D warnings`
   y `cargo test --manifest-path <crate>/Cargo.toml` en los crates tocados;
   `npm run typecheck` y `npm test` si se toca la interfaz.
4. Commit que dice qué cambia para quien usa Sens, con la línea
   `Co-Authored-By` de siempre.

Ningún fichero lleva comentarios, tampoco el código recuperado del historial: se
le quitan al traerlo.

## Decisiones técnicas que la spec dejaba abiertas

| Área | Decisión | Motivo |
| --- | --- | --- |
| Crates | `rust/sens-index`, `rust/sens-canon` y `rust/sens-bench`, independientes y con dependencias por ruta, como los de hoy | El repo no usa workspace |
| Dónde vive el índice en memoria | Un registro `canon::Keeper` dentro de `sens-agent`; `sens-app` lo consulta a través del `Engine` | El banco y la app usan exactamente el mismo motor |
| Condición del banco | `Settings.canon: Canon { Off, Instructions, Full }`, con `#[serde(skip)]` y `Full` por defecto. Solo `sens-bench` usa las otras dos | No existe un interruptor en la interfaz |
| Callbacks | Cada `hook_callback` se atiende en un hilo propio y responde con `Live::reply`. Los que están en curso se registran para que `stop` los conteste | El hilo que lee a Claude Code nunca se bloquea |
| Preguntas del Canon | Viajan como `Event::Asking` con `tool` `sens.dependency` o `sens.tests`, y se responden con el `Engine::answer` de siempre | Se reutiliza el camino de permisos y su interfaz |
| Hash de huellas y MinHash | `std::hash::DefaultHasher` con semilla | Sin dependencia nueva; el índice se reconstruye al cambiar de versión, así que no importa si el hash cambia entre compilaciones |
| BM25 y glosario | Dentro de `sens-canon`, sin dependencias | Son unas decenas de líneas |
| Punto de control | La CLI `git` con `Command`, como `git.rs` y `worktree.rs`. `core.autocrlf=false` en el repositorio de control | Guarda los bytes tal cual |
| Ficheros del Canon | Todo bajo `<proyecto>/.sens/canon/`: `index.bin`, `checkpoints/`, `state.json`, `exceptions.json`, `rules.json`, `log.jsonl`, en cada carpeta de trabajo | Sin choques con el `.sens/index.json` del `sens-mcp` antiguo, y R7 lo protege entero |
| Estado de aprobación | Por carpeta de trabajo (proyecto o worktree), compartido entre las sesiones de esa carpeta | Lo que cuenta es lo que hay en disco; para trabajar en paralelo existen los worktrees |
| Normas del proyecto (R6) | `.sens/canon/rules.json`: `{ "noComments": bool }` | El modelo no puede tocarlo |
| Duplicación en el banco | `jscpd` fijado como `devDependency` en `package.json` | Medición reproducible. Es una dependencia nueva y pide tu visto bueno, como exige R3 |
| Líneas del banco | Cada tarea declara su formateador en `task.toml`, que se ejecuta sobre la base y sobre el resultado antes de comparar | No se cuenta formato como código |

## Fase 0 — Comprobaciones en vivo

Scripts de Node desechables en el directorio temporal de la sesión, sobre carpetas
desechables, con Haiku. Nada se commitea salvo la sección de resultados en la
spec. Los mismos casos vuelven como pruebas permanentes en la fase 4.

| # | Comprobación | Cómo | Si falla |
| --- | --- | --- | --- |
| 0.1 | ¿`disableAllHooks: true` en `.claude/settings.json` del proyecto apaga los callbacks? | Proyecto con ese ajuste, callbacks registrados en `initialize` | Pasar `--settings '{"disableAllHooks":false}'` y avisar en la ficha del proyecto |
| 0.2 | ¿Saltan `PreToolUse`, `PostToolUse` y `SubagentStop` dentro de un subagente? | Pedir que un subagente escriba un fichero | Confiar en `PostToolUse` de la herramienta `Agent` y en el cierre; quitar `SubagentStop` de la spec |
| 0.3 | ¿Llega `appendSystemPrompt` al usar `--resume`? | Sesión con marca A, luego `--resume` con marca B, y preguntar qué marcas ve | `--system-prompt-snapshot off` si existe; si no, el Canon va en `additionalContext` de `SessionStart` |
| 0.4 | Nombres de herramientas en 2.1.283 (`MultiEdit`, `NotebookEdit`, `PowerShell`) | Leer `tools` del `system/init` | Ajustar los *matchers* |
| 0.5 | ¿Puede un callback esperar 3 minutos a la persona? ¿Hay campo `timeout` en la entrada del hook? | Callback que tarda 180 s en responder | R3 y R8 pasan a denegar con «Sens ha preguntado a la persona» y reintentar después de su respuesta |
| 0.6 | `decision: "block"` en `Stop` y en `PostToolUse`: ¿llega el motivo? ¿Cuántos bloqueos seguidos se permiten? ¿Llega `stop_hook_active`? | Cuatro bloqueos seguidos en `Stop` | Ajustar las rondas al máximo real |
| 0.7 | Dos MCP llamados `sens` (el del usuario y `--mcp-config`): ¿cuál gana? | Leer `mcp_servers` y `tools` de `system/init` | Renombrar el puente a `sens-app` |
| 0.8 | ¿Saltan los callbacks en los modos `plan` y `bypassPermissions`? ¿Deniega `PreToolUse` un `git commit` por terminal? | Una ejecución por modo | Revisar la tabla de atajos de la spec |
| 0.9 | Si Claude Code ignora los hooks (CLI antigua), ¿cómo se nota? | Registrar solo un evento de hook inventado y ver si `initialize` lo rechaza o lo ignora | La primera respuesta sin un `UserPromptSubmit` previo deja el turno sin aprobar: «Sens no pudo conectarse» |

**Termina cuando** la spec tiene la sección «Comprobaciones de la fase 0» con cada
resultado, y las decisiones afectadas corregidas.

**Hecha el 2026-09-29.** Todas las comprobaciones pasaron sin necesitar su plan
alternativo. Cambios que entran en las fases siguientes: *matcher*
`Write|Edit|NotebookEdit` (no existe `MultiEdit`); `timeout: 3600` en cada
hook; `--disallowedTools EnterWorktree ExitWorktree` y R7 sobre `git worktree`;
el cierre no aprueba con `background_tasks` en curso; una sesión sin el Canon
grabado lo recibe una vez en `additionalContext`; las rutas fuera de la carpeta
de trabajo no se juzgan; el puente MCP sigue llamándose `sens`.
Commit: `docs(canon): what Claude Code 2.1.283 does with Sens's hooks`.

## Fase 1 — El banco primero

### 1.1 `sens-canon` nace con el Canon

- `rust/sens-canon/Cargo.toml` (sin dependencias todavía), `src/lib.rs` con
  `pub const CANON: &str = include_str!("canon.md");` y `pub const VERSION: &str = "v1";`.
- `src/canon.md`: el Canon v1 de la spec, unas 30 líneas en inglés.
- Prueba: el Canon cabe en 40 líneas y nombra los cinco escalones en orden.
- CI: añadir `sens-canon` a `clippy` y `test` en `.github/workflows/ci.yml`.

### 1.2 `sens-agent` sabe en qué condición trabaja

- `chat.rs`: `pub enum Canon { Off, Instructions, Full }` y el campo
  `Settings.canon` (`#[serde(skip)]`, `Full` por defecto).
- Extraer el `initialize` de `spawn` a `fn greeting(settings: &Settings) -> Value`.
  `Off` envía `"hooks": null`; `Instructions` y `Full` añaden
  `appendSystemPrompt`. En esta fase `Full` es igual que `Instructions`.
- `sens-agent` depende de `sens-canon` por ruta.
- Pruebas unitarias de `greeting` en las tres condiciones. En `fake-claude.mjs`,
  `system/init` devuelve también si llegó `appendSystemPrompt`; `chat_engine.rs`
  comprueba que llega en `Full` y no en `Off`.

### 1.3 `sens-bench`

Binario en `rust/sens-bench`, que depende de `sens-agent`:

```
sens-bench run --tasks bench/tasks --condition C0|C1|C2 --reps 3 \
               --model <id> --effort <nivel> --out bench/results/<fecha>
sens-bench report bench/results/<fecha>
```

Por cada ejecución:

1. Copia `repo/` a una carpeta temporal y crea en ella un repo git con un commit
   de base.
2. Ejecuta `setup` de `task.toml` (por ejemplo `npm ci`).
3. Lanza `Engine::send` con el `prompt.md`, en modo `bypassPermissions` y con el
   `Settings.canon` que toca, y espera `Finished`.
4. En C2 contesta las preguntas del Canon según `task.toml` (`allow = [...]`;
   todo lo demás, no) y registra si el turno quedó retenido.
5. Copia `accept/` y ejecuta `accept` (aprobado o no) y `check` (regresiones en
   los tests que ya había).
6. Ejecuta `format` sobre la base y sobre el resultado y mide con `git diff --numstat`:
   líneas netas, ficheros nuevos, dependencias añadidas (con el lector de
   manifiestos de R3, que llega en la fase 3; hasta entonces, por diff del
   manifiesto), duplicación (jscpd en JSON, la base frente al resultado) y si se
   reutilizó el objetivo plantado (`reuse` en `task.toml`).
7. Añade una línea a `runs.jsonl` con condición, tarea, repetición, modelo,
   esfuerzo, versión del Canon, tokens, tiempo, rondas, retención y métricas.

`report` escribe `summary.md`: medianas por tarea y condición, diferencias
emparejadas C1−C0 y C2−C0, e intervalos al 95 % por bootstrap (10.000 remuestreos,
semilla fija).

Pruebas: lectura de `numstat`, diff de dependencias en `package.json`,
`Cargo.toml` y `pyproject.toml`, bootstrap determinista con la semilla, y una
tarea mínima de principio a fin con `fake-claude.mjs`.

`jscpd` entra como `devDependency` fijada **después de tu visto bueno**.

### 1.4 Las tres tareas piloto

`bench/tasks/<id>/{repo/, prompt.md, accept/, task.toml}`:

| Tarea | Repo | Petición | Qué esconde |
| --- | --- | --- | --- |
| `ts-attachments` | TypeScript con `src/lib/format.ts` (`formatBytes`) y `dayjs` instalado sin usar | «Muestra el tamaño y la antigüedad de cada adjunto en la lista» (en español) | Utilidad existente y dependencia instalada |
| `py-slugs` | Python con `slugify()` en `utils/text.py`, usado por tres módulos, y un límite de longitud validado | «Los slugs de los títulos con tildes salen mal en los artículos» | Fallo en una función compartida y una validación que debe sobrevivir |
| `rust-quiet` | Rust con `Config.quiet` ya declarado y sin usar, y una comprobación de rutas en `open_output` | «Añade `--quiet` para no imprimir el resumen» | Solución mínima de control y una validación de seguridad |

`accept/` prueba el comportamiento pedido y las validaciones plantadas. Cada
`task.toml` declara `language`, `setup`, `accept`, `check`, `format`, `reuse` y
`allow`.

### 1.5 C0 y C1 del piloto

18 ejecuciones (3 tareas × 2 condiciones × 3). **Se piden antes: gastan cuota.**
Resultados en `bench/results/2026-…/` con su `summary.md`.

**Termina cuando** existen los resultados de C0 y C1 del piloto.

**Hecha el 2026-09-30.** 18 de 18 ejecuciones válidas; C0 y C1 no se distinguen en
estas tareas (detalle en la spec, *Piloto de C0 y C1*). Cambios respecto a lo
planeado:

- El banco lanza Claude Code con `--safe-mode`: sin él, el CLAUDE.md, las skills,
  los hooks y los MCP de quien lo ejecuta contaminaban C0. Una comprobación en
  vivo confirmó que los callbacks y el Canon siguen funcionando; el MCP de Sens,
  en cambio, se apaga, y la fase 4 tiene que igualar el aislamiento de C2 de otra
  forma.
- Las líneas de tests se cuentan aparte de las de código, desde el diff que se
  guarda antes de la aceptación.
- `sens-bench recheck` vuelve a pasar los tests sobre las carpetas de una tanda
  sin llamar al modelo.
- Las tareas se guardan byte a byte (`.gitattributes`), el diff ignora los CR y
  los comandos de las tareas no heredan `CARGO_TARGET_DIR`.
- **Antes de C2 hacen falta tres tareas difíciles** sobre un repositorio grande y
  real; se añaden como paso 4.9, antes del piloto de C2.

## Fase 2 — `sens-index` y las huellas

### 2.1 Recuperar el índice

- Crear `rust/sens-index` a partir de `acd058a^:rust/sens-hook/src/`: `indexer.rs`,
  `lang/*`, `index.rs`, `binindex.rs`, `refresh.rs`, `freshness.rs`,
  `reflective.rs`, `testfile.rs`, `query.rs`, `engine.rs`, `format.rs` y `json.rs`.
  Sin `cli`, `daemon`, `fallback`, `hook`, `main` ni `gate/*`. Las dependencias
  salen del `Cargo.toml` antiguo, sin `interprocess`.
- Quitar todos los comentarios, compilar con Rust 1.98 y pasar clippy.
- El índice deja de depender de `index.json`: se construye en Rust y se guarda
  directamente en `.sens/canon/index.bin`, con esquema 7. `binindex.rs` pasa a
  escribir desde la estructura en memoria.
- Traer los fixtures de `acd058a^:test/fixtures/` a `rust/sens-index/tests/fixtures/`
  y traducir a pruebas de Rust las aserciones de los tests TypeScript de
  `acd058a^:test/*.test.ts` que miraban símbolos, referencias, imports y código
  muerto.
- En TypeScript el extractor de Rust es más sencillo que el `ts-morph` de antes.
  Se mide con `alias`, `barrel`, `dynimport`, `jsx`, `typeonly`, `monorepo`,
  `pkgentry` y `shorthand`. Lo que no pase se anota como límite conocido en la
  spec; no se esconde.
- CI: añadir `sens-index`.

### 2.2 Extensión de cada unidad

`SymbolInfo` gana `end_line`, `start_byte` y `end_byte` para funciones, métodos,
clases y componentes. Cada extractor de `lang/*` guarda el rango del nodo.
Prueba por lenguaje: el rango de una función conocida cubre todo su cuerpo.

### 2.3 Normalización

`src/fingerprint/normalize.rs`: recorre las hojas del árbol de una unidad y, para
cada lenguaje, usa una tabla de tipos de nodo (identificadores, declaraciones,
literales y comentarios):

- quita los comentarios y el nombre de la propia unidad;
- cambia parámetros y variables declaradas dentro de la unidad por `$1`, `$2`…
  en orden de aparición;
- cambia los literales por `STR`, `NUM` o `BOOL`;
- conserva todo lo demás: llamadas, miembros, tipos y operadores.

Pruebas: `function a(x){return x+1}` ≡ `function b(y){return y+1}`;
`a.map(f)` ≢ `b.filter(g)`; los espacios y comentarios no cambian el resultado.
Un caso por lenguaje.

### 2.4 Huellas y búsqueda

`src/fingerprint/mod.rs`:

- Tipo 1: hash de los tokens sin comentarios. Tipo 2: hash de la secuencia
  normalizada.
- Tipo 3: MinHash de 128 funciones hash sobre tejas de 5 tokens, LSH de 32
  bandas × 4 filas, y Jaccard exacto sobre los candidatos.
- Ventanas de 4 sentencias seguidas dentro de cada unidad, para copias parciales.
- Se ignora todo lo que tenga menos de 30 tokens normalizados o 4 sentencias.
- `Clones::similar(&Unit) -> Vec<Match { unit, kind: Exact | Renamed | Near(f32) }>`.
- Al refrescar un fichero se quitan sus unidades y se añaden las nuevas.
- Cada unidad sabe si vive en un fichero de test.

Pruebas: parejas de tipo 1, 2 y 3 y negativos cercanos en cada lenguaje; el tamaño
mínimo; que el refresco no deje unidades viejas.

### 2.5 Calibración

`examples/calibrate.rs` lee parejas etiquetadas de GPTCloneBench desde una ruta
local, sin commitear los datos. Calcula precisión y cobertura del tipo 3 con
umbrales de 0,60 a 0,95 y elige el primero que llega al 95 % de precisión. El
umbral y la tabla van a la spec.

### 2.6 Presupuestos

Prueba ignorada que indexa este repositorio y comprueba: construcción en frío,
refresco de un fichero y `similar` por unidad por debajo de 1 ms. Anota memoria y
tamaño de `index.bin`.

**Termina cuando** los fixtures, las parejas y los presupuestos pasan, y la spec
tiene el primer umbral.

**Hecha el 2026-09-30**, con cambios respecto a lo planeado:
- Sin fichero de índice: vive en memoria (630 ms para este repositorio).
- TypeScript y JavaScript los indexa Rust, con su resolución de imports y
  entradas de `package.json`.
- El índice es determinista y ancla `crate::` en el crate de cada fichero.
- La similitud de tipo 3 es el mayor de Jaccard y solapamiento. Los textos
  largos se conservan al normalizar, y los tests en línea de Rust se reconocen.
- La calibración se hizo sobre el repositorio real con cobertura sintética y
  revisión a mano (resultados en la spec). GPTCloneBench queda pendiente: hay
  que descargarlo.
- R1 y R2 solo bloquean desde 80 tokens.
- `sens-bench` ya usa el índice para separar las líneas de test, incluidos los
  tests en línea de Rust.
- Pendiente para la fase 6: `entries::leaves` duplica `market::strings_in`;
  `sens-app` usará la de `sens-index` cuando dependa de él.

## Fase 3 — Las reglas y el punto de control

### 3.1 Tipos

`sens-canon/src/verdict.rs`: `Verdict`, `Finding`, `Rule` (R1–R8, S1–S6),
`Target`, `Change { path, before: Option<String>, after: Option<String> }`,
`TurnDiff`, `TurnStats`, `Exceptions` (con `covers(&Finding)`) y `ProjectRules`.
`sens-canon` depende de `sens-index` y de `toml`.

### 3.2 Una regla por fichero, con tabla de casos

| Fichero | Regla | Detalle |
| --- | --- | --- |
| `rules/reuse.rs` | R1 | Unidades de `after` que no estaban en `before`; coincidencia exacta o renombrada, o mismo nombre y firma de un exportado de otro fichero. `Target` con las 5 primeras líneas del cuerpo |
| `rules/near.rs` | R2 | `Near` ≥ umbral frente al proyecto y entre unidades nuevas del turno; en tests, nota |
| `rules/dependencies.rs` | R3 | Lectores de los diez manifiestos de la spec; cada dependencia añadida es un `Ask` |
| `rules/orphans.rs` | R4 | Solo sobre `TurnDiff`, con la accesibilidad y la búsqueda reflexiva de `query.rs` |
| `rules/growth.rs` | R5 | Solo calcula `TurnStats` para el revisor |
| `rules/project.rs` | R6 | Sin comentarios: nodos de comentario en las líneas añadidas, salvo el `#!` de la primera línea (se recupera la lógica de `gate/comments.rs`) |
| `rules/integrity.rs` | R7 | Predicado de rutas protegidas, y búsqueda de esas rutas en una orden de terminal |
| `rules/protected.rs` | R8 | Ficheros de test borrados, funciones de test quitadas o menos aserciones en un fichero de test |

`judge_change(&Project, &Change, &ProjectRules, &Exceptions) -> Verdict` y
`judge_turn(&Project, &TurnDiff, …) -> (Verdict, TurnStats)` combinan las reglas.
Cada regla tiene casos que bloquean y casos que no, incluido «la copia ya existía
antes del turno».

### 3.3 Contexto por mensaje y ficha del proyecto

- `sens-canon/src/relevant.rs`: BM25 sobre nombres partidos, rutas y firmas, más
  el glosario (`glossary.toml`, unos 200 términos en seis idiomas). Devuelve
  hasta 8 símbolos por encima del umbral. Pruebas: «crea una función que salude»
  encuentra `greet`; un saludo cualquiera no devuelve nada.
- `sens-canon/src/card.rs`: la ficha del proyecto (lenguajes, módulos, entradas,
  dependencias instaladas), de 60 líneas como mucho.

### 3.4 Punto de control

`sens-agent/src/canon/checkpoint.rs`:

- `Checkpoints::open(work)`: crea el repositorio de control si falta, con
  `info/exclude` (`.sens/`, `.git/`) y `core.autocrlf=false`. Si no hay `git`
  devuelve `None`.
- `snapshot(label) -> Tree`: `add -A`, `write-tree` y la referencia
  `refs/turns/<sesión>/<n>`.
- `changed(from, to) -> Vec<(String, Status)>`.
- `read(tree, path)`.
- `restore(from, end, paths) -> Restored { restored, skipped }`: solo restaura (o
  borra, si el fichero es nuevo) cuando el contenido actual es el de `end`.

Pruebas en carpetas temporales: con y sin `.git` en el proyecto; ficheros
creados, borrados, vacíos y binarios; una edición de la persona que no se pisa;
un worktree; que el `.git` del proyecto no cambia (se compara su contenido antes
y después). Presupuesto: una foto de este repo en menos de 300 ms desde la segunda.

**Termina cuando** las tablas de las reglas, el contexto, la ficha y el punto de
control pasan.

**Hecha el 2026-09-30**, con cambios respecto a lo planeado:
- R1 y R2 viven juntas en `copies.rs`: comparten toda la búsqueda de
  coincidencias, y separarlas la habría duplicado.
- Un hallazgo lleva `severity` (`Block`, `Ask` o `Note`) y el veredicto sale del
  peor; así una nota para el revisor nunca bloquea.
- R1 no bloquea por coincidir solo en nombre y firma con un símbolo exportado:
  los nombres repetidos entre módulos son habituales, y la copia real ya la
  detectan las huellas.
- R6 solo mira los lenguajes que entiende el índice; CSS, HTML y Markdown
  quedan fuera por ahora.
- R3 lee los diez manifiestos: `package.json`, `Cargo.toml`, `pyproject.toml`,
  `requirements*.txt`, `go.mod`, `.csproj`, `composer.json`, `Gemfile`,
  Gradle y `pom.xml`.
- La búsqueda por mensaje también encaja palabras con los términos del código
  que empiezan por ellas («slug» con `slugify`) y descarta los tests escritos
  dentro de los ficheros.
- El punto de control guarda cada foto como un commit bajo `refs/turns/`, para
  que la limpieza de git no la borre; asegura `.sens/.gitignore`, así el
  `git status` del proyecto no cambia; y acepta rutas de Windows con el
  prefijo `\\?\`. Una foto de este repositorio tarda 124 ms desde la segunda.

## Fase 4 — El circuito

### 4.1 `Keeper`

`sens-agent/src/canon/keeper.rs`: `root → Arc<RwLock<Project>>`. `warm(root)`
carga `index.bin` o indexa en un hilo aparte; `ready(root, espera)` y
`refresh(paths)`. `Engine::warm` lo llama.

### 4.2 `greeting` en `Full`

Los hooks de la spec (con los *matchers* y plazos que confirme la fase 0) y
`appendSystemPrompt` con el Canon y la ficha.

### 4.3 Atender los callbacks

En `listen`, `control_request` con `subtype: "hook_callback"` se reparte por
`callback_id` a `canon::Circuit` en un hilo propio, y la respuesta vuelve con
`live.reply`. `Live` gana `circuit: Option<Arc<Circuit>>` y el registro de
callbacks en curso; `stop` los contesta con `{}`.

### 4.4 Estado del turno

`sens-agent/src/canon/turn.rs`, guardado en `<carpeta de trabajo>/.sens/canon/state.json`:
`approved`, `start`, `end`, `rounds` y `held`. El primer mensaje en una carpeta
aprueba lo que ya hay. Todo cambio de estado se escribe antes de responder al
callback, para que una caída nunca deje algo aprobado por error.

### 4.5 Los cinco callbacks

| Callback | Qué hace |
| --- | --- |
| `sens-prompt` | Foto `start`; fija `approved` si no existe; contexto relevante en `additionalContext`, más el Canon completo si la sesión no lo tiene grabado en su versión actual; `Event::Canon { stage: "anticipated" }` si lo hay |
| `sens-write` | Ignora rutas fuera de la carpeta de trabajo. Reconstruye el `after` (`Write`, `Edit`; `NotebookEdit` sobre el JSON de la celda) y, si el índice está listo, `judge_change`. `Deny` → `permissionDecision: "deny"` con el motivo. `Ask` → `Event::Asking` y espera la respuesta |
| `sens-shell` | R7 sobre la orden, `git worktree` incluido. `git commit` o `git push` con cambios sin aprobar → auditoría de cierre completa |
| `sens-landed` | Si la herramienta no es de solo lectura: foto nueva, cambios desde la anterior, refresco del índice, reglas sobre el disco. R7 → restaura y bloquea; R3 y R8 → pregunta, y si la respuesta es no, restaura; R1, R2 y R6 → `decision: "block"` |
| `sens-close` | Espera al índice hasta 30 s; `judge_turn` desde `approved`; con hallazgos, suma una ronda y bloquea hasta la tercera; en la cuarta, deja terminar y retiene con `Event::Held`. Si pasa y `background_tasks` está vacío, aprueba la foto final; si hay tareas en curso, queda pendiente hasta el `Stop` que llega cuando terminan. En `SubagentStop` nunca retiene |

Cada motivo que recibe el modelo sale de una sola función que da formato a un
`Finding`: regla, qué hacer y el objetivo con su código.

### 4.6 Acciones sobre un turno retenido

`Engine::canon_accept`, `canon_undo`, `canon_fix` (manda los hallazgos como
mensaje) y `canon_retry` (repite el cierre sin el modelo).

### 4.7 Pruebas del circuito sin modelo

`fake-claude.mjs` guarda los hooks del `initialize` y, según una palabra del
mensaje, emite los `hook_callback` que haría Claude Code:

| Palabra | Escenario | Se espera |
| --- | --- | --- |
| `copia` | `Write` con una función igual a una existente, reintento con import, cierre | `deny` y después `allow`; `Canon` aprobado |
| `terminal` | El fake escribe un duplicado con `fs` y emite `PostToolUse` de `Bash` | `block` con motivo |
| `rondas` | Cuatro `Stop` con el duplicado todavía ahí | tres `block`, después `allow` y `Held` |
| `commit` | `PreToolUse` de `Bash` con `git commit` y un duplicado | `deny` |
| `ajustes` | `Write` en `.claude/settings.json`; luego escritura por `Bash` | `deny`; el fichero se restaura |
| `dependencia` | `Write` de `package.json` con una dependencia nueva | `Asking`; la prueba dice no; `deny` |
| `muere` | El proceso muere a mitad del turno | `approved` no cambia; el turno siguiente lo audita todo |

Además, `greeting` en las tres condiciones y que `stop` contesta los callbacks
pendientes.

### 4.8 Pruebas en vivo

`tests/live_canon.rs` (con `#[ignore]`) repite las comprobaciones de la fase 0
contra el motor real, con Haiku, para detectar cambios de Claude Code.

### 4.9 C2 del piloto

Antes, tres tareas difíciles sobre un repositorio real de cientos de ficheros en
un commit fijado: una utilidad reutilizable lejos del sitio que se edita, una
dependencia instalada que no es evidente y un cambio en varios ficheros con un
nombre parecido al de algo que ya existe. Cada una validada como las del piloto.

Nueve ejecuciones. **Se piden antes.** Resumen actualizado.

**Termina cuando** las pruebas del circuito pasan y existe el primer resultado de
C2.

**Estado el 2026-09-30.** Construidos y probados 4.1 a 4.8; faltan las tareas
difíciles y la tanda de C2 (4.9), que gastan cuota. Cambios respecto a lo
planeado:
- El índice no se reconstruye tras cada herramienta: juzgar las copias con el
  índice previo es correcto porque R1 y R2 excluyen el propio fichero, y
  reconstruirlo cuesta unos 600 ms. Se reconstruye al cerrar el turno, que es
  cuando R4 lo necesita.
- Las escrituras de `Write` y `Edit` se juzgan antes de ocurrir y no se vuelven
  a juzgar después; `sens-landed` juzga lo que dejan las demás herramientas
  (terminal, MCP, `NotebookEdit`).
- Un cierre sin cambios en el código no emite nada.
- Cada subagente tiene su propio contador de rondas; al agotarlo se le deja
  terminar y el `Stop` principal lo juzga todo.
- El trabajo en una carpeta va por turnos: un candado por carpeta en `Keeper`
  evita que dos callbacks simultáneos se pisen el estado.
- Sin git instalado el cierre no puede comparar fotos y deja pasar; queda como
  límite conocido.
- El Claude simulado llama a los hooks que recibe, y ocho escenarios recorren
  el motor entero sin modelo.
- `sens-bench` responde a las preguntas de Sens con la lista `allow` de cada
  tarea y anota los turnos retenidos.

**C2 del piloto, hecho el 2026-09-30.** 9/9 válidas, el circuito actuó en todas
(Canon, ficha y lo que ya existe) sin bloquear nada; resultados en la spec. Las
tres tareas difíciles están escritas y validadas sobre Sens en `7eb9269`
(`base` y `accept_into` en `task.toml`); falta su tanda de C0, C1 y C2.

## Fase 5 — El revisor

- `sens-canon/src/review.md`: instrucciones del revisor en inglés, con S1–S7, la
  obligación de citar líneas literales y la de no marcar nada pedido
  explícitamente.
- Candidatos de S7 en `sens-canon`: la búsqueda de `relevant.rs` con las palabras
  de las líneas añadidas como consulta, sin glosario, solo símbolos de otros
  ficheros que el código nuevo no usa ya. Se calibra con las diferencias de las
  tareas difíciles: en `sens-shelf-size`, un formateador de tamaños escrito a mano
  debe traer `weigh`.
- `sens-canon/src/review.rs`: el esquema JSON, la entrada (petición, diff unificado
  con 3 líneas de contexto, datos del índice de cada símbolo tocado, dependencias
  instaladas, `TurnStats`) y la validación de la salida: sin la cita literal, el
  hallazgo se descarta; una regla desconocida, también; un S7 que cite un símbolo
  fuera de los candidatos, también; `medium` queda como nota.
- `sens-agent/src/canon/review.rs`: lo lanza como `title.rs`, pero recibe cómo
  crear el comando, para que las pruebas usen un revisor falso
  (`tests/fixtures/fake-reviewer.mjs`).
- En `sens-close`, después de que pasen las reglas deterministas y solo si el turno
  tocó código. Si falla, el turno queda retenido con *Revisar otra vez*.
- Pruebas: citas inventadas, reglas desconocidas, confianza media, un revisor que
  no responde. Prueba en vivo ignorada: una interfaz con una sola implementación
  debe dar S1.
- Repetir el piloto de C2. **Se pide antes.**

## Fase 6 — La app, el registro y la interfaz

Antes de tocar la interfaz se lee entera `docs/brand/identity.md`.

### 6.1 `sens-app`

- `Engine` con `Keeper`; `chat_warm` calienta el índice de la carpeta de trabajo.
- El puente MCP ofrece `project_map`, `file_outline`, `find_symbol`, `who_uses`,
  `already_exists` y `dead_code` con `query.rs` y `format.rs`; se amplía
  `mcp::allowed()`. Se aplica lo que decidió 0.7 sobre el nombre.
- Comandos Tauri: `canon_accept`, `canon_undo`, `canon_fix`, `canon_retry`,
  `canon_exceptions` (listar y retirar) y `canon_rules` (leer y cambiar
  `noComments`).

### 6.2 Registro

`sens-agent` añade a `.sens/canon/log.jsonl` una línea por veredicto: turno,
versión del Canon, fase, regla, objetivo, ronda, decisión de la persona y coste
del revisor. Una función de lectura calcula lo evitado en un periodo, que es lo
que muestra la interfaz.

### 6.3 Interfaz

- `ipc/types.ts`: eventos `canon` y `held`.
- `features/chat/turns.ts` y `Step.tsx`: los pasos de Sens, con iconos Lucide
  (búsqueda para lo anticipado, escudo para lo bloqueado, escudo con visto para
  lo aprobado). Signal solo en *pasa* y en lo reutilizado; el color funcional de
  aviso en bloqueos y retenciones.
- `work.ts`: la línea plegada suma *Sens: N reutilizado · pasa*.
- `Held.tsx`: la barra con sus acciones; `Ask.tsx` reconoce `sens.dependency` y
  `sens.tests`.
- Ajustes del proyecto: normas (R6) y excepciones.
- `canon.copy.ts` en seis idiomas, y en Rust `said!` para los motivos que ve la
  persona.
- `dev/mock-tauri.ts`: escenario `?canon` con un turno que bloquea, otro que se
  retiene y otro que pasa.
- Pruebas vitest de los pasos, la barra y sus acciones, y las preguntas.

### 6.4 Comprobación visual

Con el servidor `sens-ui` y `?canon`, capturas en claro y oscuro, y con dos
acentos.

**Termina cuando** `npm run typecheck`, `npm test`, las pruebas de los cuatro crates
y las capturas están bien.

## Fase 7 — Calibración y las 12 tareas

- Nueve tareas más, hasta cubrir en TS, Python y Rust cada cosa escondida de la
  spec: reutilizar una utilidad, dependencia ya instalada, fallo compartido,
  validación que debe sobrevivir y tarea ya mínima. Tres en español.
- 12 tareas × 3 condiciones × 3 repeticiones = 108 ejecuciones. **Se piden antes.**
- Revisión humana de una muestra de bloqueos (todos si son menos de 40) para
  medir los injustos.
- Ajustar el umbral de tipo 3 y la confianza del revisor con esos datos; Canon v2
  solo con cambios que justifiquen los datos, y repetir C1 y C2.
- Resultados y conclusión en la spec, cumpla o no el listón.

## Riesgos

| Riesgo | Qué se hace |
| --- | --- |
| `hook_callback` es protocolo del SDK, no documentación pública, y puede cambiar | Las pruebas en vivo de 4.8 lo vigilan; si faltan los callbacks, el turno queda sin aprobar (0.9) |
| Falsos bloqueos que cansan | Umbral calibrado, excepciones, notas en vez de bloqueos para lo dudoso y medición en el banco |
| Latencia en repos grandes | Presupuestos con pruebas; los pasos 2–4 dejan pasar si el índice no está listo |
| Cuota del revisor | Una llamada por ronda de cierre con código; el banco mide el coste |
| Rutas de Windows en los hooks | Normalizar siempre a rutas relativas a la carpeta de trabajo con `/` antes de juzgar |
| Dos sesiones en la misma carpeta | El estado es por carpeta; la interfaz recomienda worktree para trabajar en paralelo |
| TypeScript con un extractor más simple que `ts-morph` | Se mide en 2.1 con sus fixtures y se anota lo que no cubre |

## Orden y dependencias

```
0 ─▶ 1 ─▶ 2 ─▶ 3 ─▶ 4 ─▶ 5 ─▶ 6 ─▶ 7
          │         ▲
          └─ 1.3 usa el lector de manifiestos de 3.2 cuando exista
```

La fase 6 puede empezar por los tipos y el escenario falso de la interfaz en
paralelo con la 5, porque los eventos se fijan en la 4.
