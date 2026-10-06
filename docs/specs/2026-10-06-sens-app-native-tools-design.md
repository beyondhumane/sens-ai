# Herramientas nativas de Sens

Fecha: 2026-10-06 · Ámbito: `rust/sens-app` (`mcp.rs`, `tools/`, `terminal.rs`,
`main.rs`), `rust/sens-agent/src/canon/circuit.rs`, `ui/src/app/acts.ts`,
`ui/src/features/terminal/`, `ui/src/features/chat/looks.ts`.
Fase 1 implementada; fases 2 a 5 por hacer.

## Decisiones

- Claude trabaja con las herramientas de Sens, no con las del sistema, igual que en
  Claude Code Desktop. Todo cuelga del puente MCP que ya existía (`sens`, HTTP en
  `127.0.0.1`, un token por ámbito): ningún proceso, puerto ni dependencia nueva
  salvo `vt100`.
- Sens se queda con la consola y el segundo plano: cada sesión arranca con
  `--disallowedTools Bash,PowerShell,Monitor`. En Claude Code 2.1.287 ya no existen
  `BashOutput` ni `KillShell`; `TaskStop` se queda porque también para subagentes.
  `Read`, `Edit`, `Write`, `Glob` y `Grep` siguen siendo los nativos: Sens ya los
  pinta como diffs y el Canon ya los juzga.
- Si el puente no arranca no se desactiva nada, para que Claude nunca se quede sin
  shell.
- Fuera, a propósito: credenciales y cuentas (claves, iniciar o cerrar sesión con un
  proveedor), escalar sus propios permisos (confiar en un proyecto, pasar su sesión
  a *No checks*), modificar el Canon (aceptar lo retenido, reglas, excepciones) y
  borrar su propia sesión. Instalar una actualización solo con permiso, porque
  reinicia la app en mitad del turno.

## Arquitectura

- `mcp.rs` es transporte: HTTP, token, JSON-RPC. Despacha `tools/list` y
  `tools/call` al registro y no sabe qué herramientas hay.
- `tools/mod.rs` es el registro. Cada dominio es un módulo con una tabla
  `TOOLS: &[Tool]`; una `Tool` lleva nombre, título, nivel, descripción, esquema y
  manejador `fn(&Desk, &Scope, &Value) -> Result<String, String>`. Las seis
  herramientas del índice (`sens-agent/canon/tools.rs`) se sirven desde el mismo
  registro.
- `Scope` es `{ session, within }`: la sesión y sus carpetas (el proyecto y, si la
  sesión está aislada, su worktree, que es la última y donde se trabaja). Cada ámbito
  tiene su token; una herramienta nunca toca nada fuera de `within`.
- `Desk` es lo que un manejador puede usar: `Consoles`, el `Keeper` del índice, la
  ventana (`Ui`) y `tell` para emitir eventos.
- La ventana responde por un único canal de ida y vuelta: el backend emite
  `sens-act { ask, act, input }` y la interfaz contesta con
  `act_answer(ask, ok, text)`. `app/acts.ts` reparte cada `act` a quien lo registró
  con `answers(act, run)`. Si nadie contesta en 5 s, la herramienta falla con «Sens
  did not answer in time».

## Niveles

| Nivel | Qué hace | Permiso |
| --- | --- | --- |
| Leer | Solo lee | En `--allowedTools`, `readOnlyHint` |
| Mostrar | Cambia lo que se ve, no los datos | En `--allowedTools` |
| Cambiar | Ejecuta, escribe, instala, borra | Aviso de permiso del hilo según el modo; `destructiveHint` |

Con *Plan* las de cambiar quedan bloqueadas y con *No checks* pasan, igual que
`Bash` hasta ahora.

## Fase 1 · Consola y segundo plano

| Herramienta | Nivel | Qué hace |
| --- | --- | --- |
| `run_in_terminal` | Cambiar | `{ command, description, background?, wait? }`. Un PTY nuevo en la carpeta de trabajo con la shell de Sens. En primer plano espera y devuelve `Exit code N` y la salida (los últimos 30 000 bytes). Con `background` vuelve al momento con el número de la terminal. Si pasan `wait` segundos (120 por defecto, 600 como máximo) no lo mata: pasa a segundo plano |
| `read_terminal` | Leer | `{ terminal?, lines? }`. Una terminal de Claude se lee del backend; la de la persona, de su pantalla, como antes |
| `write_terminal` | Cambiar | `{ terminal, text, enter? }`. Teclea en una terminal: responder un prompt o `\u0003` para Ctrl+C |
| `stop_terminal` | Cambiar | `{ terminal }`. Termina la terminal y todo lo que lanzó; la pestaña queda para leerla |
| `list_terminals` | Leer | Número, shell, carpeta, qué corre y si sigue vivo |

- El comando viaja como `-EncodedCommand` (UTF-16LE en base64), así que comillas y
  saltos de línea llegan intactos. Antes va `[Console]::OutputEncoding = UTF8`;
  después, la salida se decide por `$?` y `$LASTEXITCODE`. Fuera de Windows,
  `$SHELL -l -c`.
- ConPTY pregunta la posición del cursor (`ESC[6n`) al arrancar y no escribe nada
  hasta que se le contesta. El backend contesta con la posición real, en una sola
  escritura (troceada no la reconoce).
- La pantalla se reconstruye con `vt100` (120×30, 10 000 líneas de historia): `\r`,
  barras de progreso y líneas partidas salen como en la pantalla.
- Un comando en primer plano no abre pestaña: se ve en el paso del chat como
  cualquier comando. Al pasar a segundo plano, el backend emite
  `terminal-adopted { id, root, shell, title, backlog }` con todo lo impreso hasta
  entonces, y desde ahí la salida fluye por el evento `terminal` de siempre.
- La pestaña de Claude lleva el icono `brain`, su `description` como nombre y
  «La lanzó Claude · …» como título. La interfaz no repite la respuesta del cursor
  que ya dio el backend.

## Canon

- El hook `PreToolUse` de comandos vigila
  `Bash|PowerShell|mcp__sens__run_in_terminal|mcp__sens__write_terminal` y lee
  `command` o `text`. Como el modelo ya no tiene otra shell, el Canon ve todo
  comando, también lo que teclea en una terminal.

## Fases siguientes

2. Superficie de la app: `open_file` (archivo y línea en el visor),
   `reveal_in_tree`, `show_changes`, `show_pane`, `close_pane`, `get_layout`,
   `split_pane`, `notify`, `screenshot_app`, `open_terminal_tab`.
3. Navegador y preview: `preview_start`, `navigate`, `page_text`, `read_page`,
   `find`, `click`, `type`, `scroll`, `screenshot`, `eval_js`, `console_logs`,
   `network_requests`, `resize`.
4. Sesiones, git y proyectos: `list_sessions`, `read_session`, `new_session`,
   `send_to_session`, `rename_session`, `archive_session`, `stop_session`,
   `open_session_in_pane`, modelo, esfuerzo y pensamiento; `repo_status`,
   `changes`, `checkout`, `isolate_session`; `list_projects`, `open_project`.
5. Capacidades, mercado, artefactos y ajustes: skills, servidores y plugins,
   `create_skill`, `market_search`, `market_detail`, `market_install`,
   `market_update`; artefactos; tema, idioma, avisos, bandeja, inicio con Windows,
   novedades y actualizaciones. Canon solo en lectura.

## Pruebas

- `tools/mod.rs`: cada herramienta se anuncia una vez; solo las de cambiar esperan
  permiso; el ámbito acepta sus carpetas escritas de cualquier forma.
- `tools/console.rs`: salida recortada por el final; nada fuera del proyecto; leer la
  terminal de la persona pregunta a la ventana; con shell real, un comando devuelve
  salida y código, y uno lento pasa a segundo plano, se lee y se para.
- `terminal.rs`: la pantalla devuelve líneas enteras más allá de su altura; el
  comando viaja entero.
- `mcp.rs`: por HTTP actúa dentro de las carpetas de la sesión y rechaza a extraños.
- `circuit.rs`: teclear `rm -rf .sens` en una terminal se deniega como un comando.
- `terminal.test.tsx`: la pestaña de Claude llega con lo impreso antes y después, y
  no contesta dos veces al cursor; el puente recibe lo leído o el fallo.
