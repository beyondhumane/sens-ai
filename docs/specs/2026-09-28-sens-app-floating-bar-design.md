# Barra flotante

Fecha: 2026-09-28 · Ámbito: `rust/sens-app` (`bar.rs`, `front.rs`, `life.rs`,
`profile.rs`, `main.rs`, `capabilities/bar.json`), `rust/sens-setup` (cierre y
desinstalación), `ui/bar.html`, `ui/src/bar/`, y en la ventana principal
`ui/src/features/{handover,notify,settings,profile}` y `ui/src/main.ts`.

## Decisiones

Todas las tomó Sofía el 2026-09-28, salvo el atajo, que no eligió; se queda el que
propuso la sesión anterior.

- Una barra que se abre encima de cualquier app con **Ctrl+Alt+Espacio**. Escribes,
  la respuesta **se despliega dentro de la barra** y puedes seguir la conversación
  sin abrir Sens.
- **Cada apertura es una sesión nueva en el último proyecto** (`last_project`). El
  proyecto se ve como chip; **Tab** pasa al siguiente y **Mayús+Tab** al anterior.
  Con **↑** en la barra vacía se retoma la última conversación de la barra.
- **Esc** la cierra (la oculta). **Ctrl+P** la fija: sigue encima y ya no se oculta
  al perder el foco. **Ctrl+⏎** la pasa a la ventana principal.
- Contexto, **siempre como chips que se añaden o se quitan; nada se envía solo**:
  - **Captura de la ventana que tenías delante** (Ctrl+Mayús+S o clic en el chip).
  - **Portapapeles reciente**: aparece si copiaste texto hace menos de 30 s.
  - **Arrastrar y pegar** ficheros e imágenes, con el mismo código que el compositor.
  - **Dictado** con la voz local de Sens (botón de micrófono).
- **Sens vive en la bandeja**: cerrar la ventana principal la oculta, arranca con
  Windows sin ventana y solo hay un Sens a la vez. Las dos cosas se desactivan en
  Ajustes.
- Aspecto **D2, «el corte vuelve a casa»** (maqueta
  `.superpowers/brainstorm/1686-1790593076/content/costura-con-marca.html`).

## Aspecto (D2)

Una sola barra de 640 px de ancho, radio `--r-sheet`, fondo `--panel` y borde
`--hair-strong` (`--edge` en oscuro). Sin sombra: la profundidad la da el filete,
como en el resto de Sens. La ventana es transparente y 24 px más ancha por cada
lado.

- **Fila**: marca apagada · campo · chip del proyecto · micrófono · enviar/parar.
  - La **marca apagada** es `Mark` a 22 px con `--mark-body: var(--edge)` y el corte
    en `--panel` (negativo), trazo 5.2.
  - El campo usa `--ui` a 16 px. Placeholder: «Pide lo que necesites a Sens».
    Cuando ya hay conversación y el campo está vacío, el placeholder es la última
    pregunta, en `--dim`.
  - El chip del proyecto va en `--mono` 12 px: icono de carpeta, nombre y caret.
  - Enviar se pinta en `--primary` solo si hay algo que enviar. Mientras trabaja
    muestra parar.
- **Chips** debajo de la fila, sangrados a la altura del campo. Ofrecido = borde
  discontinuo `--edge` y texto `--ghost`. Añadido = sólido, fondo `--card` y texto
  `--dim`, con una × para quitarlo.
- **Costura**: una línea SVG al pie de la barra (entre la fila y la respuesta cuando
  esta existe), a 14 px de cada borde. Mismos trazados que la maqueta:
  - **reposo**: la S dormida `M0 10 C150 10 196 2 300 6 C404 10 450 2 600 2` en
    `--edge`, 1.6 px, respirando (opacidad 1 ↔ .6, 5 s).
  - **escribiendo**: recta, en `--ghost`.
  - **trabajando**: recta en `--hair-strong` con una luz que la recorre cada 1100 ms
    (`--focus`, 1.6 px).
  - **esperando** (hay un permiso o una pregunta pendiente): la luz se detiene donde
    esté y pasa a `--amber`.
  - **respuesta** (mientras llega el texto): la S `M0 9 C150 9 196 2.5 300 6 C404 9.5
    450 3 600 3` en `--focus`.
  - **hecho**: la costura vuelve a la S dormida en 240 ms y el **corte de la marca se
    enciende una vez** en `--focus`, 900 ms en total (22–45 % encendido). Es el
    único destello.
  - **error**: recta en `--red`, partida por el medio una sola vez. Sin bucle.
- **Signal está en un solo sitio en cada momento.** Los trazos encendidos usan
  `--focus`, nunca `--glow`, para mantener el contraste en tema claro. El resplandor
  (`drop-shadow`) solo va en `[data-mode="dark"]` y no pasa de 1.5 veces el grosor.
- **Respuesta**: la última respuesta, con los mismos componentes que el hilo
  (`Said`, `Run`, `Ask`). Mientras trabaja, la línea viva con el brillo de la
  maqueta. Al terminar, un pie con «Ctrl ⏎ abrir en Sens · Ctrl P fijar · Esc
  cerrar» y, a la derecha, «N ficheros leídos · 3,8 s» (de `tally` y `finished`).
- Con `prefers-reduced-motion` no hay luz que corra, ni respiración, ni destello:
  cada estado se pinta quieto.
- La altura crece con el contenido hasta 680 px lógicos. A partir de ahí la
  respuesta hace scroll y se queda pegada al final mientras llega texto.

## Comportamiento

- **Abrir**: el atajo alterna. Si la barra está visible y con foco, se oculta. Si no,
  se coloca (centrada en el monitor del cursor, al 22 % del área de trabajo),
  recibe `bar-open` y toma el foco.
  - Si la barra está fijada y la moviste, conserva su sitio mientras Sens siga
    abierto.
  - Al abrir, si la conversación de la barra no está trabajando ni tiene una
    pregunta pendiente, la barra empieza en limpio: sesión nueva, en el último
    proyecto, con los ajustes de modelo, esfuerzo, pensamiento y modo guardados
    (`sens.choice`, `sens.effort`, `sens.thinking`, `sens.mode`), leídos otra vez.
- **Ocultar**: con Esc, con el atajo, o al perder el foco si no está fijada. Lo que
  esté trabajando sigue trabajando.
- **Enviar**: ⏎ envía y Mayús+⏎ salta de línea. El primer mensaje crea la sesión
  (`new_session_id` → `open_session` → `chat_send`) y llama a `remember(root)`.
  Los siguientes siguen en la misma sesión. Tras el primer turno, la barra pide el
  título con `title_session`.
- **↑ con la barra vacía y sin conversación**: retoma `sens.bar.last` (`{root,
  session}`) con `load` y muestra su última respuesta.
- **Ctrl+⏎**: llama a `bar_hand_over({root, session, text})`, donde `text` es lo que
  hubiera escrito. La barra se queda en limpio. La ventana principal abre la sesión
  (`resume`) o, sin sesión, un borrador en `root`, pone `text` en el compositor y
  toma el foco.
- **Proyecto**: Tab / Mayús+Tab recorren `bar_projects()`. El clic abre una lista.
  Cambiar de proyecto empieza en limpio.
- **Chips**:
  - **Captura**: se ofrece si había una ventana ajena delante. Su etiqueta es el
    nombre de la app (el ejecutable sin `.exe`, o el título en las apps UWP). Al
    activarla, `bar_shot()` devuelve un PNG de ≤ 1600 px que entra como imagen
    pegada (`desk.pasted`). Si la captura sale vacía o negra, el chip desaparece.
  - **Portapapeles**: se ofrece si `bar_context()` trae `clip`. Al activarlo,
    `bar_clip()` devuelve el texto completo (≤ 20 000 caracteres) y entra como texto
    pegado (`pasteText`).
  - **Ficheros e imágenes** arrastrados o pegados, igual que en el compositor.
- **Permisos y preguntas**: se contestan dentro de la barra con `Ask`. Al contestar
  se aplica el cambio de modo en el pane de la barra, igual que en la principal.
- **Notificaciones**: mientras la barra está visible, la ventana principal no avisa
  de la sesión que la barra muestra (evento `bar-watching`). Si la ocultas mientras
  trabaja, los avisos vuelven.
- **Dictado**: el micrófono usa `useDictation` con la voz local. Si la voz no está
  descargada, el aviso es el mismo que en el compositor.
- **«Hey Sens»** (añadido el 2026-09-29; apagado por defecto, en Ajustes › Focus y
  voz): con él activo, Sens escucha frases cortas con Whisper en el equipo y, si
  una empieza por «Hey Sens» u «Oye Sens», abre la barra con `bar-open { listen:
  true }` y la barra empieza a dictar sola. Ese dictado para tras unos 4 s de
  silencio y deja el texto sin enviar. Mientras hay otro dictado o una prueba del
  micrófono, no escucha.

## Ciclo de vida

- **Una sola instancia**: un mutex con nombre (`Local\SensDesktop`, o
  `Local\SensDesktopDev` en debug). Si ya existe, la segunda instancia avisa a la
  primera y sale. El aviso es un mensaje registrado `SensShow`, enviado a su ventana
  de mensajes (clase `SensListener`). Si la segunda arrancó con `--hidden`, sale sin
  avisar.
- **Bandeja**: icono con el icono de la app y el tooltip «Sens».
  - Clic: muestra la ventana principal.
  - Menú: «Abrir Sens», «Barra rápida (Ctrl+Alt+Espacio)», separador, «Salir».
    Localizado a los seis idiomas.
- **Cerrar la ventana principal**: si `keepInTray` está activo (por defecto sí), se
  oculta. Si no, Sens sale.
- **Salir** es siempre `life::quit(app)`, que para voz, consolas y motor y termina
  el proceso. Lo usan el menú «Salir», el cierre sin bandeja y la actualización.
- **Inicio con Windows** (`startWithWindows`, por defecto sí): valor `Sens` en
  `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` = `"<exe>" --hidden`.
  - Se escribe al arrancar si falta o apunta a otro exe, y se borra al desactivarlo.
  - Nunca se escribe en builds de debug.
  - Con `--hidden` la ventana principal carga, pero no se muestra
    (`window.__SENS_HIDDEN__`) y el hilo de respaldo de 4 s no la enseña.
- **Mensaje `SensQuit`**: la ventana de mensajes lo atiende con `quit`.
  `sens-setup` lo envía antes de su `WM_CLOSE` (que ahora solo ocultaría) y el
  desinstalador borra el valor `Run`.
- Si el atajo no se puede registrar (otra app lo tiene), Ajustes lo dice y la barra
  sigue disponible desde el menú de la bandeja.

## Contrato

Comandos nuevos (Rust → `commands` en `ipc/commands.ts`):

| Comando | Firma JS | Devuelve / efecto |
| --- | --- | --- |
| `bar_hide` | `invoke("bar_hide")` | oculta la barra |
| `bar_fit` | `invoke("bar_fit", { height })` | ajusta la ventana a `height` lógicos (NaN se ignora; 90–680) |
| `bar_pin` | `invoke("bar_pin", { on })` | fija o suelta la barra (no se oculta al perder el foco) |
| `bar_drag` | `invoke("bar_drag")` | empieza a arrastrar la ventana (se usa con la barra fijada) |
| `bar_hand_over` | `invoke("bar_hand_over", { hand: { root, session, text } })` | oculta la barra, muestra la principal y le emite `bar-hand-over` |
| `bar_context` | `invoke("bar_context")` | `{ front: { app, title } \| null, clip: { preview, chars } \| null }`; `clip` solo si se copió hace < 30 s |
| `bar_clip` | `invoke("bar_clip")` | `string \| null`, el texto del portapapeles (≤ 20 000 caracteres) |
| `bar_shot` | `invoke("bar_shot")` | `{ mediaType: "image/png", data, width, height } \| null`, la ventana de delante |
| `bar_projects` | `invoke("bar_projects")` | `{ root, name }[]` del registro, sin sesiones, el último primero |
| `bar_open` | `invoke("bar_open")` | abre la barra (lo usa la bandeja; también sirve a la principal) |
| `set_keep_in_tray` | `invoke("set_keep_in_tray", { on })` | guarda `keepInTray` |
| `set_start_with_windows` | `invoke("set_start_with_windows", { on })` | guarda `startWithWindows` y escribe o borra el valor `Run` |
| `shortcut_state` | `invoke("shortcut_state")` | `{ keys: "Ctrl+Alt+Espacio", taken: boolean }` |

`Profile` gana `keepInTray: bool` y `startWithWindows: bool`, los dos `true` por
defecto y con `#[serde(default)]`.

Eventos:

| Evento | Destino | Carga |
| --- | --- | --- |
| `bar-open` | solo `bar` (`emit_to`) | `{ look, language, front: { app, title } \| null, pinned }` |
| `bar-hand-over` | solo `main` | `{ root, session, text }` |
| `bar-watching` | la barra lo emite a `main` | `{ session: string \| null }` |

El portapapeles nunca viaja en `bar-open`; solo se lee cuando la barra lo pide.

## Piezas

- `bar.rs`: la ventana, `toggle`, la colocación (área de trabajo), el ocultar al
  perder el foco, los comandos `bar_*` y el hilo del atajo.
  - La ventana tiene `WS_EX_TOOLWINDOW` para no salir en Alt+Tab ni contar como
    ventana principal.
- `front.rs` (Windows, con stubs para el resto):
  - La ventana de delante se lee en el hilo del atajo, antes de mostrar la barra.
    Se descartan el escritorio, la barra de tareas, las ventanas ocultas (cloaked) y
    las de este proceso.
  - Da el título y el ejecutable, la captura con `PrintWindow(PW_RENDERFULLCONTENT)`
    reducida a ≤ 1600 px y codificada en PNG (`png 0.18`, ya en el lockfile), y la
    lectura del portapapeles.
  - El portapapeles se lee con reintentos, `GlobalSize`,
    `IsClipboardFormatAvailable(13)` y, siempre, en un comando `async`.
  - La hora de la última copia sale de `AddClipboardFormatListener` en la ventana
    de mensajes.
- `life.rs`: instancia única, ventana de mensajes (`SensListener`: portapapeles,
  `SensShow`, `SensQuit`), bandeja, `quit`, valor `Run` y `--hidden`.
- `ui/src/bar/`: `main.ts`, `Bar.tsx`, `store.ts`, `Seam.tsx`, `Chips.tsx`,
  `copy.ts`, `bar.css` y sus tests. Reutiliza `newPane`, el store del chat (con un
  oyente propio filtrado por sesión), el del compositor, `useDictation`, `Said`,
  `Run`, `Ask`, `Mark`, `ICONS` e i18n. Sin E/S del rail, de ficheros ni de cambios
  en su reino.
- En la principal: `features/handover` (oye `bar-hand-over`), `notify` (salta la
  sesión vigilada), `settings` (dos interruptores y el estado del atajo), `main.ts`
  (no se muestra con `__SENS_HIDDEN__`).

## Fuera de alcance

- Cambiar el atajo desde Ajustes.
- Ver en la barra un hilo más largo que la última respuesta: para eso está Ctrl+⏎.
- Leer el directorio de un terminal ajeno para adivinar el proyecto.
