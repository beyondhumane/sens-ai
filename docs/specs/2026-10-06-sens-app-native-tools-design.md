# Herramientas nativas de Sens

Fecha: 2026-10-06 · Ámbito: `rust/sens-app` (`mcp.rs`, `tools/`, `terminal.rs`,
`main.rs`), `rust/sens-agent/src/canon/circuit.rs`, `ui/src/app/acts.ts`,
`ui/src/features/terminal/`, `ui/src/features/chat/looks.ts`.
Implementado: las cinco fases, 58 herramientas propias más las 6 del índice.

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
  ventana (`Ui`), la app (`Sens`: captura de la ventana y avisos de Windows) y
  `tell` para emitir eventos.
- Un manejador devuelve `Said`: texto, o una imagen con su pie, que viaja como
  contenido `image` de MCP.
- La ventana responde por un único canal de ida y vuelta: el backend emite
  `sens-act { ask, act, input }` y la interfaz contesta con
  `act_answer(ask, ok, text)`. `app/acts.ts` reparte cada `act` a quien lo registró
  con `answers(act, run)`. Si nadie contesta en 5 s (20 s para `navigate`, que espera
  a que cargue la página), la herramienta falla con «Sens did not answer in time».

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

## Fase 2 · Superficie de la app

| Herramienta | Nivel | Qué hace |
| --- | --- | --- |
| `open_file` | Mostrar | `{ path, line? }`. El archivo en el visor, con el árbol desplegado hasta él; la línea queda en el centro, marcada con `--tint` y su número con `--focus`. Un Markdown pedido en una línea se abre como código |
| `show_pane` | Mostrar | `{ pane }`: `files`, `changes`, `web`, `terminal` o `tasks` |
| `close_pane` | Mostrar | Cierra el panel lateral |
| `open_terminal_tab` | Mostrar | Una terminal para la persona en la carpeta de la sesión |
| `get_layout` | Leer | Si la persona mira esta sesión, qué panel y qué archivo hay, cuántas terminales (y cuántas de Claude) y cuántos chats lado a lado |
| `screenshot_app` | Leer | La ventana de Sens como imagen (`PrintWindow`, la misma captura que la barra flotante) |
| `notify` | Mostrar | `{ body, title? }`. Un aviso de Windows, titulado «Claude» si no se dice otra cosa |

- Lo que cambia la pantalla solo ocurre si la persona está mirando esa sesión; si
  no, la herramienta falla diciéndolo y la pantalla queda como estaba. Una sesión
  en segundo plano nunca le mueve la vista a quien trabaja en otra.
- La ruta de `open_file` se resuelve canónica dentro de la carpeta de trabajo:
  `../` o una ruta absoluta de fuera se rechazan.
- Partir el chat en dos y abrir una sesión en un panel pasan a la fase 4, con las
  sesiones.

## Fase 3 · Navegador

El navegador de Sens es una WebView2 hija de la ventana. Claude lo maneja por el
protocolo de DevTools de esa misma vista (`browser::devtools`, con
`CallDevToolsProtocolMethod`), sin otro navegador ni extensión.

| Herramienta | Nivel | Qué hace |
| --- | --- | --- |
| `navigate` | Mostrar | `{ url }`: una dirección, un servidor local, una página del proyecto (por el servidor de preview), palabras que buscar, o `back`, `forward` y `reload`. Abre el panel web y espera a que cargue (15 s) |
| `read_page` | Leer | `{ all? }`. Lo que se puede usar en la página, con su referencia `ref_N`; con `all`, también encabezados, regiones y listas. Nunca el valor de una contraseña |
| `find` | Leer | `{ query }`. Hasta 20 elementos por nombre, rol o placeholder |
| `page_text` | Leer | El texto de la página, el contenido principal primero |
| `click` | Cambiar | `{ ref }` o `{ x, y }`, y `double`. Lleva el elemento a la vista y envía el ratón por `Input.dispatchMouseEvent` |
| `type_text` | Cambiar | `{ text, ref?, clear? }`. Enfoca el campo, lo vacía si se pide, y escribe con `Input.insertText` |
| `press_key` | Cambiar | `{ key }`: Enter, Tab, Escape, Backspace, Delete, Space, flechas, Home, End, PageUp, PageDown, F5. Enter lleva su texto, así que envía un formulario |
| `scroll` | Mostrar | `{ ref?, screens? }` |
| `screenshot_page` | Leer | La página como imagen (`Page.captureScreenshot`) |
| `eval_js` | Cambiar | `{ expression }`. Su valor en JSON; una promesa se espera |
| `console_logs` | Leer | `{ errors_only?, pattern? }`. Lo que la página escribió en su consola desde que cargó |
| `network_requests` | Leer | `{ failed_only?, pattern? }`. Método, dirección, estado y tipo, o por qué falló |
| `resize_browser` | Mostrar | `{ width }`: 375 un móvil, 768 una tableta, 0 el ancho del panel |

- El guion que lee la página vive en `ui/src/features/web/page.js`, se prueba con
  jsdom y Rust lo incluye con `include_str!`. Se instala una vez por página como
  `window.__sens` y marca cada elemento con `data-sens-ref`, así que las referencias
  duran mientras la página no cambie; una que ya no está lo dice.
- La consola y la red se guardan también en el backend (las últimas 500 entradas).
  La consola se vacía al empezar a cargar una página, como la del panel; la red se
  sigue con `Network.enable`.
- Leer la página se puede desde cualquier sesión; lo que la cambia o la mueve
  (`navigate`, `click`, `type_text`, `press_key`, `scroll`, `eval_js`,
  `resize_browser`) solo si la persona mira esa sesión, porque el navegador es uno
  para toda la ventana.
- Para la preview de un servidor de desarrollo no hay herramienta propia: Claude lo
  arranca con `run_in_terminal` en segundo plano y lo abre con `navigate`.

## Fase 4 · Sesiones, proyectos y git

| Herramienta | Nivel | Qué hace |
| --- | --- | --- |
| `list_projects` | Leer | Los proyectos abiertos en Sens, con cuántas sesiones tienen y cuál es el de esta |
| `list_sessions` | Leer | `{ project?, archived? }`. Id, título, mensajes de la persona, y si es esta o está trabajando |
| `read_session` | Leer | `{ session, project?, turns? }`. Los últimos turnos (10): lo que pidió la persona, lo que respondió Claude y las herramientas que usó |
| `open_session` | Mostrar | `{ session, project? }`. La abre al lado: la ventana se parte en dos chats |
| `new_session` | Cambiar | `{ prompt?, project? }`. Una sesión nueva al lado; con `prompt`, se lo envía y otro Claude trabaja en paralelo |
| `send_to_session` | Cambiar | `{ session, text, project? }`. Le escribe como lo haría la persona, con el modelo y el modo de ese panel; la abre al lado si no está en pantalla |
| `stop_session` | Cambiar | `{ session }`. Como el botón de parar |
| `rename_session` | Cambiar | `{ title, session?, project? }`. Esta si no se dice otra |
| `archive_session` | Cambiar | `{ session, archived?, project? }` |
| `close_session_pane` | Mostrar | `{ session }`. Cierra la mitad de la ventana que la muestra |
| `set_session_model` | Cambiar | `{ session?, model?, effort?, thinking? }`. Para los mensajes siguientes de una sesión en pantalla; el modelo por su nombre o id en el selector, y un esfuerzo que ese modelo tenga |
| `repo_status` | Leer | Rama, las demás ramas, y los archivos cambiados o nuevos desde el último commit |
| `switch_branch` | Cambiar | `{ branch }`. Cambia de rama en la carpeta de trabajo y refresca el árbol, el visor y la rama del panel |

- Una sesión nunca se para, se archiva ni se escribe a sí misma, y no se escribe a una
  que sigue trabajando.
- Otro proyecto vale solo si Sens lo tiene registrado; la ruta se compara sin
  distinguir barras ni mayúsculas.
- Leer, renombrar, archivar y la rama van directos al backend (`session`, `git`);
  abrir, escribir, empezar y elegir modelo pasan por la ventana, que conoce los
  paneles y el modelo de cada uno (`currentSettings`). Abrir una sesión que no está en
  pantalla solo ocurre si la persona mira la que lo pide.
- No hay herramienta para aislar una sesión en un worktree: Sens lo decide antes del
  primer mensaje y mover una sesión viva de carpeta rompería su proceso.

## Fase 5 · Capacidades, mercado, ajustes, artefactos y Canon

| Herramienta | Nivel | Qué hace |
| --- | --- | --- |
| `list_capabilities` | Leer | Skills, servidores MCP y plugins instalados, y si cada uno está activo en este proyecto |
| `read_skill` | Leer | `{ name }`. Su `SKILL.md` entero |
| `create_skill` | Cambiar | `{ name, description, body }`. Una skill nueva, activa en este proyecto |
| `set_capability` | Cambiar | `{ kind, name, enabled }`. Activa o desactiva una skill, un servidor o un plugin en este proyecto |
| `remove_capability` | Cambiar | `{ kind, name }`. La desinstala de Sens |
| `add_server` | Cambiar | `{ name, command, args? }`. Un servidor MCP local, sin variables de entorno |
| `market_search` | Leer | `{ query }`. Hasta 20 resultados del mercado |
| `market_detail` | Leer | `{ id }`. Todo lo de un elemento: partes, archivos, readme y valores que pide |
| `market_install` | Cambiar | `{ id, values? }`. Lo instala, activo en este proyecto |
| `market_update` | Cambiar | `{ id, name }`. Lo trae a su última versión |
| `get_settings` | Leer | Nombre, aspecto, idioma, avisos, bandeja, inicio con Windows y búsqueda de actualizaciones |
| `set_look` | Cambiar | `{ mode?, accent? }`. Oscuro, claro o como Windows, y el acento |
| `set_language` | Cambiar | `{ language }`: `en`, `es`, `fr`, `de`, `ja` o `zh` |
| `set_preference` | Cambiar | `{ name, on }`: `notify`, `keep_in_tray`, `start_with_windows` o `check_updates` |
| `show_view` | Mostrar | `{ view }`: `capabilities`, `artifacts`, `news`, `settings` o vuelta a `chat` |
| `check_updates` | Leer | Si hay una versión nueva de Sens |
| `read_news` | Leer | Lo nuevo de las últimas versiones |
| `list_artifacts` | Leer | Lo que hicieron o enlazaron las sesiones de todos los proyectos |
| `read_artifact` | Leer | `{ target }`. El texto de un artefacto |
| `canon_status` | Leer | Reglas, cambios retenidos, excepciones y lo que el Canon evitó en este proyecto |

- Ningún secreto pasa por Claude: `add_server` no acepta variables de entorno ni
  cabeceras, y `market_install` se niega con un elemento que pide un secreto
  obligatorio o con un valor secreto, y dice a la persona que lo instale en
  Capacidades.
- Instalar una actualización de Sens no tiene herramienta: reinicia la app, así que
  es el clic de la persona en la píldora de la barra de título.
- Del Canon solo se lee. Aceptar lo retenido, deshacer, cambiar reglas o conceder
  excepciones siguen siendo de la persona, porque son justo la manera de saltárselo.
- Los ajustes pasan por las mismas funciones de la interfaz que los ajustes
  (`chooseLook`, `chooseLanguage`, `setNotices`, `setResident`, `setAutomatic`), así
  que se aplican al momento y quedan guardados. Un cambio de capacidades vale desde
  el siguiente mensaje de cada sesión y refresca la vista de Capacidades si está
  abierta.

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
- `tools/surface.rs`: un archivo se abre por su ruta y nada de fuera; los paneles
  son los que hay; la ventana llega como imagen salvo oculta; el aviso lleva su
  título.
- `surface.test.ts`: abre en una línea, dice por qué no pudo leer, abre y cierra el
  panel, no toca la pantalla si la persona mira otra sesión, y describe lo que hay.
- `Viewer.test.tsx`: marca la línea pedida.
- `tools/web.rs`: la página se lee con el guion de Sens; el clic va donde está el
  elemento y solo en la sesión en pantalla; un fallo de la página vuelve como su
  mensaje; Enter lleva su texto; el JS devuelve su valor; consola y red se filtran;
  la captura es una imagen.
- `browser.rs`: una petición se sigue de enviada a respondida o fallida.
- `page.test.ts`: lista lo usable con referencias y nunca una contraseña; mantiene
  las referencias; encuentra; lee el texto; apunta, enfoca, vacía y avisa de una
  referencia caducada.
- `surface.test.ts`: navega y espera la carga; vuelve atrás; ancho de móvil.
- `tools/sessions.rs`: una sesión se lee por turnos; la lista marca esta y las que
  trabajan; ninguna se para, archiva ni escribe a sí misma; renombrar y archivar
  llegan al disco; otro proyecto ha de ser conocido; escribir pasa por la ventana.
- `tools/repo.rs`: rama, otras ramas, cambiados y nuevos en un repositorio real, y el
  cambio de rama.
- `sessions.test.ts`: escribe a una sesión en pantalla sin mover nada, abre al lado
  antes de escribir, empieza una nueva con su mensaje, elige modelo y esfuerzo, y
  cierra un panel solo si hay dos.
- `tools/kit.rs`: una skill se escribe, se lista, se lee, se desactiva y se
  desinstala; un servidor se añade sin secretos.
- `tools/app.rs`: los ajustes se leen de la carpeta de datos; una elección ha de ser
  de las que Sens ofrece; el Canon solo se lee.
- `preferences.test.ts`: cambia solo la parte del aspecto pedida, usa el mismo
  setter que los ajustes, muestra una vista solo para la sesión en pantalla, dice si
  hay versión nueva y refresca Capacidades solo si están a la vista.
- `terminal.test.tsx`: la pestaña de Claude llega con lo impreso antes y después, y
  no contesta dos veces al cursor; el puente recibe lo leído o el fallo.
