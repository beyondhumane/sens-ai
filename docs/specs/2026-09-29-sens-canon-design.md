# El Canon: el motor que decide qué código entra

Fecha: 2026-09-29 · Estado: diseño aprobado; fases 0 a 4 construidas, C2 del piloto medido, faltan las tareas difíciles.
Ámbito: `rust/sens-index` (nuevo), `rust/sens-canon` (nuevo), `rust/sens-bench`
(nuevo), `rust/sens-agent`, `rust/sens-app`, `rust/sens-app/ui`.

## Objetivo

Que cualquier IA conectada a Sens escriba el mínimo código correcto, reutilice lo
que ya existe, no añada lo que nadie pidió y no recorte lo que protege, **sin que
el modelo tenga forma de saltárselo**. No es una skill ni un texto que el modelo
pueda ignorar: es un circuito cerrado en el que Sens juzga todo lo que llega al
disco y decide si un turno queda aprobado.

Esta es la razón de ser de Sens. El resto del producto sirve a esto.

## Punto de partida

### Lo que ya se intentó

| Intento | Qué era | Por qué no basta |
| --- | --- | --- |
| `sens-mcp` (julio–septiembre) | Índice tree-sitter de 11 lenguajes, consultas por MCP y un hook de lectura | El modelo consulta si quiere; nada le impide duplicar |
| Motor propio (septiembre) | Sens generaba por API y pasaba un parche por G1–G5 antes de escribir | Perdía el agente de Claude Code y la suscripción; `sens-agent` lo sustituyó por el chat sobre Claude Code |
| Ponytail | Reglas en una skill | Instrucciones: el modelo puede no cargarlas o no seguirlas |

El código del índice, las consultas y las comprobaciones sigue en el historial,
en `acd058a^:rust/sens-hook/src/`. La [comparación con Ponytail](2026-09-22-sens-ponytail-analysis.md)
ya fijó el criterio que se mantiene: primero se cumple la tarea, y solo entre
soluciones correctas se compara el tamaño.

### Lo que no se puede tocar

No podemos cambiar los pesos de Claude: con la suscripción no hay ajuste fino de
Opus ni de Sonnet. Lo que Sens controla es lo que el modelo ve, lo que se le deja
hacer y cuándo se le deja terminar. El diseño usa esas tres cosas.

### El mecanismo, comprobado

Sens ya habla con Claude Code por stream-json y ya envía un `initialize` con
`"hooks": null`. Ese `initialize` acepta hooks como *callbacks*: Claude Code manda
un `control_request` con `subtype: "hook_callback"` por la misma tubería y espera
la respuesta del host. Prueba hecha el 2026-09-29 con Claude Code 2.1.283 y Haiku,
en una carpeta desechable con `lib/greet.js`:

| Hook | Respuesta de Sens | Lo que hizo el modelo |
| --- | --- | --- |
| `UserPromptSubmit` | `additionalContext`: ya existe `greet(name)` en `lib/greet.js` | Anunció que lo reutilizaría |
| `PreToolUse` sobre `Write` de `lib/hello.js` | `permissionDecision: "deny"` con motivo | No lo creó; importó `lib/greet.js` desde `main.js` |
| `appendSystemPrompt` en el `initialize` | Pedía terminar con una palabra | La respetó |

Una ejecución: demuestra el mecanismo, no la calidad. La documentación de Claude
Code dice además que un `deny` en `PreToolUse` bloquea en todos los modos de
permisos, `bypassPermissions` incluido, y que `additionalContext` llega al modelo
como recordatorio del sistema.

### Comprobaciones de la fase 0

Hechas el 2026-09-29 con Claude Code 2.1.283 y Haiku, en carpetas desechables,
con callbacks registrados en el `initialize`:

| # | Pregunta | Resultado | Consecuencia |
| --- | --- | --- | --- |
| 0.1 | ¿Un `disableAllHooks: true` del proyecto apaga los callbacks? | No. Apaga los hooks de ficheros (el hook de comando del proyecto no se ejecutó) pero los callbacks siguieron llegando y el `deny` funcionó. Tampoco los apaga un `.claude/settings.local.json` con `disableAllHooks` escrito por el modelo a mitad de sesión | No hace falta forzar nada con `--settings`. R7 sigue protegiendo esos ficheros |
| 0.2 | ¿Pasan los subagentes por los hooks? | Sí. `PreToolUse` y `PostToolUse` llegan con `agent_id` y `agent_type`, el `deny` funciona dentro del subagente y `SubagentStop` salta. Un subagente en segundo plano puede seguir escribiendo después del `Stop` principal; al terminar, Claude Code abre otro turno del modelo y llega otro `Stop` | El cierre no aprueba mientras `background_tasks` no esté vacío |
| 0.3 | ¿Llega `appendSystemPrompt` al retomar con `--resume`? | Se graba en la primera petición y se conserva al retomar; un texto nuevo al retomar se ignora. Con `--system-prompt-snapshot off` se usa el nuevo | Se mantiene la grabación; una sesión sin el Canon, o con una versión anterior, lo recibe una vez en `additionalContext` |
| 0.4 | ¿Qué herramientas hay? | `Write`, `Edit`, `NotebookEdit`, `Bash` y `PowerShell`; no hay `MultiEdit`. También hay `EnterWorktree`, `ExitWorktree` y `Workflow` | *Matcher* `Write\|Edit\|NotebookEdit`; `EnterWorktree` y `ExitWorktree` prohibidas |
| 0.5 | ¿Puede un callback esperar a la persona? | Sí: uno que tardó 180 s en contestar fue respetado, con y sin `timeout`, y el campo `timeout` se acepta | Las preguntas de R3 y R8 esperan dentro del callback; `timeout: 3600` |
| 0.6 | ¿Funciona `decision: "block"`? | En `Stop`, cuatro bloqueos seguidos respetados; `stop_hook_active` es `false` en el primero y `true` después. En `PostToolUse`, el motivo llega al modelo, pero Haiku lo trató como información y no hizo lo que pedía | Las rondas las cuenta Sens. `PostToolUse` es feedback; la garantía sigue en `Stop` |
| 0.7 | Dos MCP llamados `sens`: ¿cuál gana? | El de `--mcp-config` (`source: "dynamic"`) sustituye al global; solo se ven sus herramientas | El puente conserva el nombre `sens` |
| 0.8 | ¿Funcionan los callbacks en modo plan y en `bypassPermissions`? | Sí en los dos. En `bypassPermissions`, el `deny` de `Write` y el de un `git commit` por `Bash` funcionaron y el repositorio siguió con un solo commit. En modo plan, Claude Code escribe el plan en `~/.claude/plans` | Las rutas fuera de la carpeta de trabajo no se juzgan |
| 0.9 | ¿Qué pasa con un evento de hook que no existe? | Se ignora en silencio: el `initialize` responde `success` | Si el primer mensaje del modelo llega sin `sens-prompt` previo, el turno queda sin aprobar: «Sens no pudo conectarse» |

La entrada de cada callback trae `session_id`, `transcript_path`, `cwd`,
`prompt_id`, `permission_mode` y `hook_event_name`, y según el evento
`tool_name`, `tool_input`, `tool_response`, `tool_use_id`, `agent_id`,
`agent_type`, `stop_hook_active`, `last_assistant_message` y `background_tasks`.
Las rutas llegan absolutas y con `\` en Windows.

Una observación que cambia cómo se escriben los motivos: cuando un `deny` no
dice qué hacer en su lugar, Haiku pregunta a la persona si tiene permiso en vez
de buscar otra solución.

## Decisiones

| Decisión | Elegida | Descartadas |
| --- | --- | --- |
| Dónde vive el circuito | Callbacks del `initialize`, atendidos en proceso por `sens-agent` | Hooks en `--settings` (sustituyen los del usuario, un proceso por llamada, config editable por el modelo); motor propio por API |
| Última palabra | Si el modelo no pasa tras sus intentos, el turno queda **retenido** y solo una persona lo acepta (queda registrado) o lo deshace | Deshacer siempre; deshacer solo lo que falla |
| Reglas de criterio | Un revisor (Haiku) una vez al final de cada turno que cambió código | Solo reglas comprobables; revisor en cada escritura |
| Interruptor | No hay forma de apagar el Canon desde la interfaz | — |

## Arquitectura

| Pieza | Responsabilidad | Depende de |
| --- | --- | --- |
| `sens-index` | Índice del proyecto: símbolos, referencias, imports, exports, tests, entradas. Incremental por mtime. Huellas de cada unidad de código para detectar copias | — |
| `sens-canon` | Las reglas. Recibe un cambio o un diff y devuelve un veredicto. Contiene también el texto del Canon y el contrato del revisor. No lanza procesos ni escribe | `sens-index` |
| `sens-agent` | El circuito: registra los hooks, atiende cada callback con `sens-canon`, lleva el punto de control, lanza el revisor, cuenta rondas y retiene turnos | `sens-canon` |
| `sens-app` | Mantiene vivo el índice de cada proyecto, sirve las consultas por el puente MCP, guarda registro y excepciones, y expone Aceptar / Deshacer / Pedir arreglo | todo lo anterior |
| `sens-bench` | El banco de pruebas: conduce `sens-agent` sin interfaz en tres condiciones | `sens-agent` |

`sens-canon` no conoce a Claude. Recibe tipos propios (`Change`, `TurnDiff`) y
devuelve `Verdict`. El adaptador de Claude Code traduce `hook_callback` a esos
tipos y el veredicto a la respuesta del hook. Otra IA conectada en el futuro
necesita otro adaptador, no otro Canon; los pasos que miran el disco funcionan
igual para cualquiera.

### Tipos

```rust
struct Verdict { findings: Vec<Finding> }
enum Outcome { Pass, Ask, Deny }
enum Severity { Block, Ask, Note }

struct Finding {
    rule: Rule,
    severity: Severity,
    file: String,
    line: u32,
    message: String,
    target: Option<Target>,
    key: String,
}

struct Target { symbol: String, file: String, line: u32, signature: String, excerpt: String }
```

`Rule` enumera R1–R8 y S1–S6. El resultado del veredicto es el del hallazgo más grave: un `Block` deniega, un `Ask` pregunta a la persona y las notas nunca bloquean. `key` identifica el hallazgo para las excepciones. `message` y `excerpt` son para el modelo, en inglés;
la interfaz traduce la regla y el hallazgo a los seis idiomas.

## Un turno

1. **Llega el mensaje** (`UserPromptSubmit`). Sens toma una foto del proyecto
   (ver *Punto de control*) y devuelve en `additionalContext` lo que ya existe
   relacionado con la petición.
2. **Antes de escribir** (`PreToolUse` con `Write|Edit|NotebookEdit`; Claude Code
   2.1.283 no tiene `MultiEdit`). Sens reconstruye el fichero resultante en
   memoria (`Write`: el contenido; `Edit`: el reemplazo sobre el fichero actual;
   `NotebookEdit`: la celda sobre el JSON), lo analiza y aplica las reglas de
   cambio. `Deny` responde `permissionDecision: "deny"` con el motivo y el
   objetivo; `Ask` pregunta a la persona (ver *Preguntas*). Las rutas fuera de la
   carpeta de trabajo no se juzgan (en modo plan, Claude Code escribe su plan en
   `~/.claude/plans`).
3. **Antes de la terminal** (`PreToolUse` con `Bash|PowerShell`). Solo dos
   comprobaciones: R7 sobre rutas protegidas nombradas en la orden, y *commit
   como cierre* (ver más abajo).
4. **Después de cualquier herramienta** (`PostToolUse`, todas). Sens descarta al
   instante las de una lista cerrada de solo lectura en `sens-canon` (`Read`,
   `Grep`, `Glob`, `WebFetch`, `WebSearch`, `TodoWrite`); cualquier otra, las de
   MCP incluidas, se mira. Sens busca ficheros cambiados desde la última
   mirada, reindexa esos y aplica las mismas reglas a lo que de verdad hay en
   disco. Si hay hallazgos bloqueantes, responde `decision: "block"` con el
   motivo, que llega al modelo como corrección.
5. **Al querer terminar** (`Stop` y `SubagentStop`). Auditoría del diff completo
   desde el último punto aprobado: reglas de cambio, R4 y, si todo pasa y el
   turno tocó código, el revisor. Con hallazgos bloqueantes responde
   `decision: "block"` y el modelo sigue trabajando. Si la entrada de `Stop` trae
   `background_tasks` sin terminar, Sens audita pero **no aprueba**: el turno queda
   pendiente de esas tareas. Cuando terminan, Claude Code abre otro turno del
   modelo que acaba en otro `Stop`, y ese sí puede aprobar.
6. **Tercera ronda sin pasar**: Sens deja terminar al modelo y el turno queda
   retenido.

Los pasos 2–4 dan feedback temprano. **La garantía está en el paso 5**: si el
índice no está listo en los pasos 2–4, esos pasos dejan pasar; el paso 5 espera
al índice y lo juzga todo.

Si el primer mensaje del modelo en un turno llega sin que antes haya llegado
`sens-prompt`, el circuito no está enganchado (una CLI que ignora los hooks lo
hace en silencio): el turno queda sin aprobar con «Sens no pudo conectarse».

Todo motivo que recibe el modelo dice qué hacer en su lugar. Sin alternativa, el
modelo tiende a preguntar a la persona si tiene permiso en vez de corregir.

### Registro de hooks

```json
{
  "subtype": "initialize",
  "appendSystemPrompt": "<Canon + ficha del proyecto>",
  "hooks": {
    "UserPromptSubmit": [{ "hookCallbackIds": ["sens-prompt"] }],
    "PreToolUse": [
      { "matcher": "Write|Edit|NotebookEdit", "hookCallbackIds": ["sens-write"], "timeout": 3600 },
      { "matcher": "Bash|PowerShell", "hookCallbackIds": ["sens-shell"], "timeout": 3600 }
    ],
    "PostToolUse": [{ "hookCallbackIds": ["sens-landed"], "timeout": 3600 }],
    "Stop": [{ "hookCallbackIds": ["sens-close"], "timeout": 3600 }],
    "SubagentStop": [{ "hookCallbackIds": ["sens-close"], "timeout": 3600 }]
  }
}
```

`timeout` da margen a una pregunta a la persona y al revisor. La sesión se lanza
además con `--disallowedTools EnterWorktree ExitWorktree`: Sens gestiona sus
propios worktrees, y uno creado por el modelo dentro de `.claude/worktrees` (que
suele estar en `.gitignore`) quedaría fuera del punto de control.

Las instrucciones base quedan grabadas en la primera petición de cada sesión y se
conservan al retomarla (`--system-prompt-snapshot` activado, como viene por
defecto: desactivarlo haría que el prompt de sistema se volviera a generar en
cada petición, con el coste de caché que eso supone). Una sesión empezada sin el
Canon, o con una versión anterior, recibe el Canon actual una vez como
`additionalContext` en su siguiente `sens-prompt`; Sens guarda en el estado de la
sesión qué versión tiene grabada.

## Las reglas

Se juzga solo lo que añade el turno, nunca la deuda previa: una copia que ya
existía antes del turno no es un hallazgo.

### Reglas de cambio (`sens-canon`, deterministas)

| Regla | Detecta | Veredicto |
| --- | --- | --- |
| R1 Reutilizar | Unidad nueva (función, método, clase, componente) con la misma huella de tipo 1 o 2 que una existente, o con el mismo nombre y firma que un símbolo exportado | `Deny` con el objetivo si la unidad más pequeña tiene 80 tokens o más; por debajo, prueba para el revisor |
| R2 Casi-copia | Unidad o bloque nuevo con similitud de tipo 3 por encima del umbral respecto a código existente, o entre dos bloques nuevos del turno | `Deny` (extraer y compartir) con la misma regla de 80 tokens; por debajo, prueba para el revisor. En tests, nota no bloqueante |
| R3 Dependencia nueva | Un manifiesto gana una dependencia: `package.json`, `Cargo.toml`, `pyproject.toml`, `requirements*.txt`, `go.mod`, `*.csproj`, `composer.json`, `Gemfile`, `build.gradle(.kts)`, `pom.xml` | `Ask` |
| R4 Huérfanos | Solo al cierre: símbolo nuevo inalcanzable desde las entradas, o símbolo existente que el turno dejó sin usos | `Deny` si es interno y sin usos reflexivos; nota si es exportado o reflexivo |
| R5 Crecimiento | Líneas netas y ficheros nuevos | Nunca bloquea; va al revisor |
| R6 Normas del proyecto | Normas declaradas en los ajustes del proyecto. La primera: sin comentarios | `Deny` |
| R7 Integridad | Escritura en `.claude/settings*.json`, `.mcp.json`, `.sens/` o `.git/`, y órdenes de terminal con `git worktree` | `Deny` siempre; si llegó por la terminal, Sens restaura el fichero desde el punto de control y avisa |
| R8 Tests protegidos | El turno borra ficheros de test, funciones de test o aserciones existentes | `Ask` |

### Reglas de criterio (revisor)

| Regla | Detecta |
| --- | --- |
| S1 | Abstracción sin segundo uso: interfaz, factoría, capa, envoltorio o configuración con un único consumidor |
| S2 | Síntoma en lugar de causa: el arreglo se repite en los llamadores en vez de en la función compartida |
| S3 | Reinventar lo que dan la biblioteca estándar, la plataforma o una dependencia instalada |
| S4 | Especulación: opciones, parámetros o ramas que la petición no pide |
| S5 | Ingenio donde bastaba lo evidente |
| S6 | **Recorte peligroso**: el cambio quitó validación en un límite de confianza, manejo de errores que evita pérdida de datos, seguridad, accesibilidad o algo pedido explícitamente |

S6 es el contrapeso: impide que "menos código" se cumpla quitando lo que protege.
Nada pedido explícitamente en el mensaje de la persona cuenta como sobrante.

### El revisor

- Se lanza como el título de sesión (`title.rs`): `claude -p --model haiku
  --output-format json --json-schema <esquema> --tools "" --strict-mcp-config
  --no-session-persistence --settings {"disableAllHooks":true}` con un
  `--system-prompt` propio.
- Recibe la petición de la persona, el diff del turno, para cada símbolo tocado lo
  que dice el índice (quién lo usa, qué se le parece) y las dependencias
  instaladas.
- Devuelve `{ findings: [{ rule, file, quote, why, fix, confidence }] }`, con
  `rule` en S1–S6 y `confidence` en `high` o `medium`.
- Sens descarta todo hallazgo cuyo `quote` no aparezca literal en las líneas
  añadidas de ese fichero (en S6, en las eliminadas). Solo `high` bloquea;
  `medium` es una nota para la persona.
- Se lanza una vez por ronda de cierre, solo si las reglas de cambio pasan y el
  turno tocó código.

### El Canon

Texto en inglés, unas 30 líneas, versionado (`canon v1`), en
`rust/sens-canon/src/canon.md` e incluido con `include_str!`. Viaja como
`appendSystemPrompt` junto a la ficha del proyecto:

1. Every change is judged by Sens against the project index; what Sens says about
   this project is a fact.
2. Before writing, in order: is it needed → does it exist in the project → does
   the standard library or platform give it → does an installed dependency give
   it → only then, the smallest correct code.
3. Never cut: validation at trust boundaries, error handling that prevents data
   loss, security, accessibility, anything the person asked for.
4. Fix causes in shared code, no abstraction without a second use, boring over
   clever, delete what your change leaves unused.

No se adoptan los comentarios `ponytail:` de Ponytail: chocan con las normas de
proyectos como este.

## Punto de control, retención y blindaje

### El invariante

**Un cambio nace sin aprobar.** La aprobación es un hecho positivo que solo
producen una auditoría de cierre que pasa o una aceptación de la persona. Una
caída de Claude Code, de Sens o del revisor deja lo cambiado sin aprobar.

### Punto de control

- Un repositorio git propio en `<proyecto>/.sens/canon/checkpoints`, con el proyecto
  como árbol de trabajo: `git --git-dir=.sens/canon/checkpoints --work-tree=<proyecto>`.
  Excluye `.sens/` y `.git/` en su `info/exclude` y respeta los `.gitignore` del
  proyecto. **Nunca toca el `.git` del proyecto**: ni stage, ni stash, ni ramas,
  ni objetos. Funciona igual en proyectos sin git.
- En cada mensaje: `add -A` y `write-tree` con el índice propio. Tras la primera
  foto, git solo vuelve a leer lo que cambió. Cada foto queda bajo
  `refs/turns/<sesión>/<turno>` en ese repositorio.
- Las sesiones en worktree tienen su propio repositorio de control en su carpeta.
- El diff de cierre se toma **desde el último punto aprobado**. Un turno retenido
  seguido de otro mensaje hereda sus cambios pendientes: empezar un turno nuevo
  no sirve para escapar.
- Al cerrar cada turno Sens toma también una foto final. **Deshacer** restaura
  cada fichero cambiado desde el último punto aprobado (y borra los creados) solo
  si su contenido sigue siendo el de esa foto final; si la persona lo editó
  después, no lo pisa y lo dice.
- Sin git instalado no hay punto de control: el Canon juzga y retiene igual, y la
  interfaz dice que esos cambios no se pueden deshacer desde Sens.

### Commit como cierre

Un `PreToolUse` de terminal cuya orden contiene `git commit` o `git push` con
cambios sin aprobar ejecuta antes la auditoría de cierre completa. Si no pasa, se
deniega con los hallazgos. Si pasa, el punto actual queda aprobado.

### Preguntas a la persona

`Ask` no delega en el modo de permisos. Dentro del callback, Sens emite al hilo un
evento de pregunta con el mismo aspecto que las de permisos y **no responde al
callback hasta que la persona decide**. Sí se traduce en `allow`; no, en `deny`
con el motivo. Si R3 o R8 se detectan después (por ejemplo, `npm install` en la
terminal), Sens pregunta en ese momento; un no restaura desde el punto de control
el manifiesto y su lockfile, o los tests, y se lo explica al modelo.

### Turno retenido

Al final del turno, una barra: *Sens no aprueba este turno*, los hallazgos y tres
acciones:

| Acción | Efecto |
| --- | --- |
| Pedir que lo arregle | Envía un turno con los hallazgos; vuelve a haber tres rondas |
| Deshacer | Restaura el último punto aprobado, con la regla de no pisar ediciones de la persona |
| Aceptar | Aprueba el punto actual y guarda cada hallazgo como excepción del proyecto, con fecha |

Las excepciones viven en `.sens/canon/exceptions.json` y dejan de bloquear ese
mismo hallazgo (misma regla, mismo par de unidades o misma dependencia). El
modelo no puede escribirlas: R7.

### Cerrar los atajos

| Atajo del modelo | Qué lo cierra |
| --- | --- |
| Escribir por terminal, Python u otro MCP | Paso 4 tras cada herramienta y auditoría de cierre sobre el disco |
| Subagentes | Sus herramientas pasan por los mismos hooks, con `agent_id`, y `SubagentStop` se audita |
| Tareas en segundo plano que escriben tarde | Mientras `background_tasks` no está vacío, el cierre no aprueba; al terminar, el `Stop` siguiente audita todo desde el último punto aprobado |
| Apagar los hooks o editar la configuración | Los hooks viven en el proceso: un `disableAllHooks` del proyecto, aunque lo escriba el modelo a mitad de sesión, no los apaga. R7 bloquea y restaura igualmente esos ficheros, que también guardan los hooks de la persona |
| Un worktree propio del modelo | `--disallowedTools EnterWorktree ExitWorktree` y R7 sobre `git worktree` |
| Modo sin comprobaciones o modo plan | Los callbacks funcionan en todos los modos, y `deny` en `PreToolUse` bloquea también en `bypassPermissions` |
| Declarar que ha terminado | El cierre lo decide la auditoría de `Stop` |
| Bucle sin fin | Sens cuenta tres rondas y retiene |
| Commit o push de lo no aprobado | Commit como cierre |

### Cuando falla Sens

| Fallo | Comportamiento |
| --- | --- |
| Índice no listo en los pasos 2–4 | Deja pasar; el cierre espera al índice |
| Índice no listo al cierre tras 30 s | Retiene: "Sens no pudo juzgar este turno" |
| Revisor sin respuesta (red, cuota) | Retiene con la acción extra *Revisar otra vez* |
| Error interno de una regla | Retiene con el error; nunca aprueba |

### Límites

Esto cierra los atajos y los errores del modelo, no un programa hostil en la
máquina. Los detectores pueden equivocarse: para eso están la aceptación de la
persona, las excepciones y el banco de pruebas.

## El índice y las huellas

### `sens-index`

- Recupera de `acd058a^:rust/sens-hook/src/` los extractores de `lang/*`,
  `reflective.rs`, `testfile.rs`, `query.rs` y `format.rs`, sin hook, CLI, daemon,
  formato binario ni delegación en Node. Lenguajes: TypeScript y JavaScript,
  Python, Rust, Go, Java, C#, C, C++, PHP, Ruby y Kotlin.
- **TypeScript y JavaScript los indexa ahora Rust.** Antes los resolvía `ts-morph`
  en Node. Resuelve imports relativos, `.js` escrito para un `.ts`, `index.*` de
  carpeta, re-exportaciones con nombre y `export *`; los alias de `tsconfig` no se
  resuelven, y en ese caso el índice vuelve a emparejar por nombre. Las entradas
  salen de los `index.*` y de `main`, `module`, `types`, `bin` y `exports` de cada
  `package.json`, traducidos a su fichero fuente.
- **Vive solo en memoria**, sin fichero de índice: este repositorio, 455 ficheros
  y 8.600 unidades, se indexa en unos 630 ms en release, y buscar las copias de
  una unidad cuesta un microsegundo. Guardarlo en disco solo se añadirá si un
  proyecto grande lo pide.
- **Es determinista**: dos rutas que terminan igual se resuelven siempre a la
  más corta, y `crate::` se ancla en el crate del propio fichero. Antes el
  resultado cambiaba entre ejecuciones y, en un repositorio con varios crates,
  era incorrecto.
- Sus pruebas fijan el comportamiento de código muerto de los 11 lenguajes con
  los 29 proyectos de ejemplo recuperados.

### Huellas

La unidad es cada función, método o clase, y además cada ventana de 4
sentencias seguidas dentro de una función, para las copias parciales.

| Tipo | Qué es | Técnica |
| --- | --- | --- |
| 1 | Copia exacta salvo espacios y comentarios | Hash de los tokens sin comentarios |
| 2 | Copia con otros nombres locales o literales cortos | Hash de los tokens normalizados |
| 3 | Copia con sentencias añadidas o quitadas | MinHash de 128 funciones sobre tejas de 5 tokens normalizados, LSH de 32 bandas × 4 filas, y sobre los candidatos el mayor de Jaccard y solapamiento |
| 4 | Mismo comportamiento, código distinto | Embeddings de código con un modelo local; spec siguiente |

La normalización:
- Se conservan los nombres a los que se llama, los miembros a los que se accede y
  los tipos: `a.map(f)` y `b.filter(g)` no coinciden.
- Los demás identificadores pasan a `$1`, `$2`…, y el nombre de la propia unidad
  a `$self`.
- Los textos de menos de 12 caracteres pasan a `STR`, y los números y booleanos
  a su tipo. Los textos más largos se conservan: en una función que es sobre
  todo mensajes, el contenido es el texto.
- Una unidad es de test si está en un fichero de test o si ella, o algo que la
  contiene, lleva un atributo `test` (`#[test]`, `#[cfg(test)]`).
- Por debajo de 30 tokens normalizados no se compara.

**Por qué el mayor de Jaccard y solapamiento.** Una línea añadida a una función
de 64 tokens deja el Jaccard en 0,79, pero el 93 % del original sigue dentro de la
copia. «Copiar y retocar» se mide mejor por cuánto del original contiene lo
nuevo.

### Calibración (2026-09-30)

`cargo run --release --example calibrate -- <proyecto>` sobre este repositorio
(2.171 funciones comparables, 400 editadas con semilla fija):

| Umbral | Parejas de funciones distintas | +1 trozo | +2 | +3 | −1 | −1 +1 |
| --- | --- | --- | --- | --- | --- | --- |
| 0,70 | 155 | 100 % | 96 % | 84 % | 90 % | 71 % |
| 0,80 | 104 | 100 % | 89 % | 73 % | 90 % | 54 % |
| 0,90 | 52 | 82 % | 60 % | 47 % | 76 % | 28 % |

Las columnas de la derecha son la cobertura sintética: a funciones reales se les
insertan o quitan trozos de 5 a 12 tokens de otras funciones, y se mide qué
parte se sigue encontrando. No imita del todo una edición real.

Revisión a mano de las parejas por encima de 0,80, por tamaño de la función
más pequeña:

| Tokens | Parejas | Duplicación real | Resto |
| --- | --- | --- | --- |
| 30–49 | 85 | ~31 | Envoltorios que son así por diseño (comandos de Tauri, setters) y gemelos que solo cambian una llamada |
| 50–79 | 14 | ~8 | Gemelos |
| 80 o más | 5 | 5 | — |

Entre la duplicación real hay tres copias que se escribieron en esta misma obra
(`text`, un generador aleatorio y `entries::leaves` frente a
`market::strings_in`); las dos primeras ya se corrigieron.

**Consecuencia para R1 y R2**: solo bloquean cuando la unidad más pequeña de la
pareja tiene al menos 80 tokens. Por debajo, la coincidencia va como prueba al
revisor, que decide con criterio si es una oportunidad de reutilizar. El umbral de
parecido queda en 0,80. Cinco parejas no bastan para asegurar el 95 %: la muestra
se amplía con el repositorio grande de la fase 4.

## Lo que recibe el modelo

| Momento | Contenido | Límite |
| --- | --- | --- |
| Inicio de sesión, en `appendSystemPrompt` | Canon y ficha del proyecto: lenguajes, módulos principales con una línea cada uno, entradas y dependencias instaladas | ~60 líneas de ficha |
| Cada mensaje, en `additionalContext` | Hasta 8 símbolos relacionados: firma, `fichero:línea`, número de usos. Nada si la relevancia no supera el umbral | ~400 tokens |
| Cada bloqueo, en el motivo | Regla, qué hacer y el objetivo exacto: firma, `fichero:línea` y 3–5 líneas del código existente | lo justo |
| Cuando lo pide | `project_map`, `file_outline`, `find_symbol`, `who_uses`, `already_exists`, `dead_code` por el puente MCP de `sens-app` | respuestas compactas |

La búsqueda por mensaje es léxica: nombres partidos por mayúsculas y guiones,
rutas y firmas, puntuadas con BM25. Como las peticiones pueden venir en otro
idioma que el código, lleva un glosario de unos 200 términos de programación en
los seis idiomas de Sens. Los embeddings lo sustituyen en la spec siguiente. La
garantía no depende de esta búsqueda: R1 y R2 comparan código con código.

El puente MCP de la app se llama `sens`. Si la configuración del usuario trae otro
servidor con ese nombre, como el `sens-mcp` antiguo, el de `--mcp-config` lo
sustituye (fase 0): el modelo solo ve las herramientas de la app.

### Presupuestos (p95, proyecto de ~2.000 ficheros)

| Paso | Presupuesto |
| --- | --- |
| Veredicto de una escritura | 50 ms |
| Buscar cambios en disco tras una herramienta | 150 ms |
| Contexto para un mensaje | 30 ms |
| Auditoría de cierre sin revisor | 1 s |
| Revisor | ~5–15 s, una vez por ronda de cierre con código |

## El banco de pruebas

### Pregunta

¿Claude Code con Sens entrega código igual de correcto, más pequeño y que
reutiliza más, con un coste aceptable? ¿Y cuánto de eso consiguen solas las
instrucciones?

### Condiciones

| Condición | Configuración |
| --- | --- |
| C0 | Claude Code sin Canon ni hooks |
| C1 | Solo el Canon en `appendSystemPrompt` |
| C2 | Sens completo |

Mismo modelo, esfuerzo, tareas y revisión de los repos; el modelo usado queda en
los resultados. C1 frente a C2 mide lo que añade el circuito sobre una skill.

### Tareas

`bench/tasks/<id>/` con `repo/`, `prompt.md`, `accept/` y `task.toml` (lenguaje,
orden de aceptación, objetivo de reutilización plantado). Repos en TypeScript,
Python y Rust; algunas peticiones en español. Cada tarea planta al menos una de:

- una utilidad existente que la solución buena reutiliza;
- una dependencia instalada que ya resuelve lo pedido;
- un fallo en una función compartida con varios llamadores;
- una validación de seguridad que debe sobrevivir;
- una tarea cuya solución ya es mínima, como control.

`accept/` se copia al repo solo después de cada ejecución: el modelo nunca ve esos
tests.

### Métricas, en orden

1. Pasa la aceptación y no rompe los tests existentes. Lo demás solo se compara
   entre soluciones que pasan.
2. Líneas netas tras el formateador del lenguaje, ficheros añadidos, dependencias
   añadidas, duplicación introducida, huérfanos, reutilización del objetivo
   plantado.
3. Tokens, tiempo, rondas de cierre y tasa de turnos retenidos.

La duplicación la mide **jscpd**, independiente de las huellas de Sens: no se
evalúa con la misma vara que bloquea.

### Tamaño y análisis

Piloto: 3 tareas × 3 condiciones × 3 repeticiones = 27 ejecuciones. Se publican
medianas, diferencias emparejadas por tarea e intervalos por bootstrap, y se
llama exploración. Luego 12 tareas. Resultados en `bench/results/<fecha>/` como
`runs.jsonl` y un resumen.

### Listón

C2 no pierde ninguna tarea frente a C0, mejora en duplicación y reutilización, y
menos del 5 % de sus bloqueos son injustos a juicio de una persona que revisa una
muestra.

### Piloto de C0 y C1 (2026-09-30)

Sonnet 5.5 con esfuerzo medio, Claude Code aislado con `--safe-mode`, tres
repeticiones por tarea y condición, C0 y C1 alternadas. Resultados en
`bench/results/2026-09-30-pilot/`.

| Tarea | C0 | C1 |
| --- | --- | --- |
| `py-slugs` | 3/3 válidas; arreglo en `slugify` con `unicodedata`, 2 líneas netas de código y 4 de test | Igual |
| `rust-quiet` | 3/3 válidas; usa `config.quiet`, 9 líneas netas (incluye un test dentro de `config.rs`) | Igual, una ejecución en 3 líneas |
| `ts-attachments` | 3/3 válidas; `formatBytes` 3/3, `dayjs` 2/3: una ejecución escribió a mano el «hace 3 horas» (30 líneas frente a 10) | `formatBytes` 3/3, `dayjs` 3/3 |

Ninguna condición añadió dependencias, ficheros ni duplicación. Las diferencias de
líneas, duplicación y tokens tienen intervalos que incluyen el cero.

Lo que dice el piloto:

- **Estas tareas no discriminan.** En repositorios de cinco ficheros lo reutilizable
  está a la vista, y Sonnet 5.5 ya lo usa sin ayuda. El único fallo de C0 (reinventar
  lo que da una dependencia instalada) es justo el tipo de falta que cubre el Canon,
  pero una ejecución no es evidencia.
- **El valor de Sens tiene que medirse donde el modelo no ve lo que existe:**
  repositorios de cientos de ficheros, la pieza reutilizable lejos del sitio que se
  edita, nombres parecidos pero no iguales y tareas que tocan varios ficheros.
- **Límites de la medición encontrados:** los tests en línea de Rust
  (`#[cfg(test)]`) cuentan como código porque la separación se hace por ruta, y
  se corregirá con las regiones de test del índice; y los tokens suman lecturas
  de caché, así que no son todavía una medida de coste.

Consecuencia para el plan: antes de medir C2 se añaden tres tareas difíciles sobre
un repositorio real y grande en un commit fijado, y el piloto de C2 corre sobre
las seis.

### Piloto de C2 (2026-09-30)

Las mismas tres tareas, el mismo modelo y el mismo aislamiento, con Sens completo.

| Tarea | C2 |
| --- | --- |
| `py-slugs` | 3/3 válidas; `unicodedata` 3/3, 2 líneas netas de código y ningún test |
| `rust-quiet` | 3/3 válidas; `config.quiet` 3/3, 5 líneas netas de código y 4 de test |
| `ts-attachments` | 3/3 válidas; `formatBytes` 3/3 y `dayjs` 3/3, 9 líneas netas |

Lo que hizo el circuito, leído en el estado de cada carpeta y en las
transcripciones: en las nueve ejecuciones el modelo recibió el Canon, la ficha del
proyecto y, con la primera palabra, lo que ya existe relacionado con la petición;
ninguna escritura se bloqueó, ningún cierre necesitó otra ronda, nada quedó
retenido y Sens no tuvo que preguntar nada. No había nada que corregir: en tareas
donde el modelo ya reutiliza sin ayuda, el circuito no estorba.

- Código, ficheros, dependencias y duplicación: idénticos a C0 y C1.
- Tokens: C2 − C0 = −43 787 [−54 393, −13 960] sumando las tres tareas, el único
  intervalo del piloto que no toca el cero. La explicación probable es que la
  lista de lo que ya existe ahorra búsquedas, pero los tokens incluyen lecturas de
  caché, así que no es todavía un ahorro de coste demostrado.
- En `py-slugs` C2 no añadió tests (C0 y C1 añadieron 4 líneas). Ni la tarea ni el
  Canon los piden; se vigila en las tareas difíciles.
- El banco no guardaba lo que hacía el circuito en cada ejecución; desde aquí cada
  ejecución lo anota (`circuit`) y el resumen lo cuenta.

### Tareas difíciles

Sobre el propio Sens en `7eb9269`, el commit anterior al trabajo del Canon: 412
ficheros, 752 tests. Cada tarea parte de ese commit, instala sus dependencias y
se juzga con tests ocultos que se colocan después del turno; las regresiones se
comprueban antes, con los tests y los tipos del proyecto.

| Tarea | Qué se pide | Lo que ya existe y dónde |
| --- | --- | --- |
| `sens-shelf-size` | El tamaño de cada artefacto en su tarjeta | `weigh` en `shared/format.js`, que escribe tamaños en los seis idiomas; la estantería no lo importa |
| `sens-file-language` | El lenguaje del fichero en el visor y en el panel de cambios | `languageOf` y `titleOf` en `shared/syntax/languages.ts`, sobre la tabla que el proyecto genera desde `linguist-languages`; hay otra `languageOf` en `shared/i18n.ts` y otra `titleOf` en `bar/choices.ts` |
| `sens-bar-accents` | Buscar en la barra sin distinguir acentos | `plain` en `features/market/search.js`, que la barra no importa |

Cada una se validó: en la base los tests del proyecto pasan y los ocultos fallan;
con una solución de referencia de 2 a 7 líneas pasan todos y los tipos compilan.

## Registro

`.sens/canon/log.jsonl`, local: por cada veredicto, turno, versión del Canon, fase,
regla, objetivo propuesto, ronda, decisión de la persona y coste del revisor.
Sirve para calibrar (las excepciones aceptadas por regla estiman sus falsos
positivos), para medir el contexto por mensaje (qué fracción de los símbolos
sugeridos acaba usada en el diff final) y para mostrar a la persona lo que Sens
evitó.

## Interfaz

- Eventos nuevos de `sens-agent`: `Canon { stage, verdict, findings }` para cada
  intervención y `Held { findings }` al retener. Se guardan en la transcripción
  como el resto.
- En el hilo, cada intervención es un paso: *Sens · ya existe `greet` en
  `lib/greet.js`*, *Sens bloqueó una escritura*, *Sens revisó el turno*.
- Signal marca solo *pasa* y lo reutilizado; los bloqueos y la retención usan el
  color funcional de aviso. Iconos de Lucide. Todo según
  [la identidad](../brand/identity.md).
- La línea plegada del turno suma lo de Sens: *Worked · 2 commands · 1 edit ·
  Sens: 1 reutilizado · pasa*.
- La barra de turno retenido; las preguntas de R3 y R8 con el componente de
  permisos; en los ajustes del proyecto, las normas del proyecto (R6) y la lista
  de excepciones, que la persona puede retirar.
- Comandos nuevos en `sens-app`: `canon_accept`, `canon_undo`, `canon_fix`,
  `canon_retry`, `canon_exceptions`, `canon_rules`.
- Todo texto en los seis idiomas (`copy()` en la interfaz, `said!` en Rust).

## Pruebas

| Pieza | Qué se prueba |
| --- | --- |
| `sens-index` | Los 15 fixtures recuperados; parejas de tipo 1, 2 y 3 por lenguaje; negativos cercanos (`map` frente a `filter`, getters, unidades bajo el mínimo) |
| `sens-canon` | Cada regla con tablas de casos que bloquean y que no; citas inventadas del revisor descartadas |
| Punto de control | Con y sin git; ficheros creados, borrados y vacíos; edición de la persona no pisada; worktrees |
| `sens-agent` | `fake-claude.mjs` emite `hook_callback`: escritura denegada, pregunta a la persona, cambio por terminal, tres rondas y retención, commit como cierre, R7 restaurado, caída que deja el turno sin aprobar |
| Interfaz | Pasos de Sens, barra de retención y sus acciones, preguntas, con los fixtures de `mock-tauri` |
| En vivo (ignoradas) | Las comprobaciones de la fase 0 con Haiku |

Ningún fichero lleva comentarios.

## Orden de construcción

| Fase | Contenido | Termina cuando |
| --- | --- | --- |
| 0 | Comprobaciones en vivo del comportamiento de Claude Code con los callbacks | Hecha: ver *Comprobaciones de la fase 0* |
| 1 | `sens-bench` con las 3 tareas piloto; C0 y C1 | Resultados de C0 y C1 del piloto |
| 2 | `sens-index` recuperado y huellas de tipo 1–3 con su calibración | Fixtures y parejas en verde; primer umbral |
| 3 | `sens-canon` (R1–R8, Canon v1) y punto de control | Tablas de reglas y pruebas de restauración en verde |
| 4 | El circuito en `sens-agent` y C2 en el piloto | Pruebas del circuito en verde y C2 del piloto |
| 5 | El revisor (S1–S6) | Contrato y validación de citas en verde; piloto repetido |
| 6 | Interfaz, registro y seis idiomas | Pruebas de interfaz y `typecheck` en verde |
| 7 | Calibración con datos, 12 tareas, Canon v2 | Listón cumplido o motivo documentado |

## Fuera de esta spec

Cada uno con su propia spec:

- Embeddings de código locales: copias de tipo 4 y contexto por mensaje entre
  idiomas.
- Genoma del proyecto: convenciones aprendidas del código (nombres, patrones,
  normas implícitas) añadidas al Canon y a R6.
- Otras IAs conectadas además de Claude Code.
