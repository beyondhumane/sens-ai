# El Canon aprende Unity

Entrega 1 de 3 del giro de Sens a desarrollo de videojuegos. Las otras dos,
que este documento no cubre, son el grafo de assets por GUID y el lazo con el
editor.

## La visión

Sens deja de ser *un circuito que obliga a un agente de código a reutilizar y
a escribir menos* y pasa a ser **el único entorno en el que un agente no puede
dejar un proyecto de Unity peor de lo que estaba**.

La naturaleza del argumento no cambia: sigue siendo un circuito determinista,
que el modelo no puede apagar ni rodear, que juzga lo que llega al disco y
retiene el turno que no pasa. Cambia su jurisdicción. Hasta hoy juzgaba el
código en abstracto — duplicación, reutilización, tamaño, forma. A partir de
aquí sabe además qué cuesta un frame, qué significa el ciclo de vida de un
MonoBehaviour y cuándo unos datos no son un componente.

Esa frase es la fuente de verdad para el README, la web, la identidad y el
paper cuando toque reescribirlos. No se reescriben en esta entrega: mientras
las reglas no existan, sería vender algo que no está hecho.

## Para qué

Un agente que escribe C# para Unity produce código que compila, pasa los tests
y cuesta frames. `GetComponent` dentro de `Update`, `Camera.main` en cada
cuadro, LINQ en el bucle caliente, parámetros de Animator por string: nada de
eso es un error que un test atrape, y todo eso lo arregla un senior de Unity
en la revisión. El Canon ya tiene el sitio donde pararlo antes de que llegue
al disco; lo que le falta es saber qué buscar.

## Antes de nada: los puntos de entrada de Unity

Hoy `rust/sens-index/src/lang/csharp.rs` reconoce como punto de entrada
`Main`, las clases que terminan en `Controller` o llevan `[ApiController]` y
los métodos con atributos HTTP. Los mensajes de Unity no están, y los llama el
motor, no el proyecto. La consecuencia es que en un proyecto de Unity
`dead_code` y la regla de huérfanos declaran muerto medio MonoBehaviour.

Se añaden como puntos de entrada:

- Los mensajes de Unity — `Awake`, `OnEnable`, `Start`, `FixedUpdate`,
  `Update`, `LateUpdate`, `OnDisable`, `OnDestroy`, `OnValidate`, `Reset`,
  `OnGUI`, las familias `OnCollision*`, `OnTrigger*` y `OnMouse*`, los
  `OnApplicationPause`/`Focus`/`Quit`, `OnAnimatorMove`, `OnAnimatorIK`, los
  `OnDrawGizmos*`, `OnBecameVisible`/`Invisible`, `OnWillRenderObject`,
  `OnPreRender`, `OnPostRender`, `OnRenderImage`, `OnRenderObject` — cuando el
  tipo que los declara deriva, transitivamente dentro del proyecto, de
  `MonoBehaviour`, `ScriptableObject`, `Editor` o `EditorWindow`.
- Los métodos con `[MenuItem]`, `[ContextMenu]`,
  `[RuntimeInitializeOnLoadMethod]`, `[InitializeOnLoadMethod]`, `[UnityTest]`,
  `[Test]`, `[SetUp]` o `[TearDown]`.

Es una corrección, no una función nueva, y se verifica sola: un componente
idiomático y limpio no debe producir ni un huérfano.

Los métodos que una escena o un prefab conectan por `UnityEvent` quedan fuera:
eso vive en el grafo de assets, que es la entrega 2.

## Detección del proyecto

`ProjectRules`, que hoy solo lleva `no_comments` y se guarda en
`.sens/canon/rules.json`, gana `unity` y la versión del editor leída de
`ProjectSettings/ProjectVersion.txt`. La detección es automática: el fichero
existe o no existe. La casilla en *Settings › Proyecto* solo aparece cuando se
ha detectado, y solo sirve para apagar las reglas; eso añade copy en los seis
idiomas de `project.copy.ts`.

La versión del editor hace falta porque hay reglas que dependen de la API
disponible: `FindObjectsByType` sustituye a `FindObjectsOfType` en 2023, y
Addressables solo se puede recomendar si el paquete está en
`Packages/manifest.json`.

## El camino caliente

La pieza de la que cuelga la mitad de las reglas. `GetComponent` no es malo;
`GetComponent` en código que corre cada frame sí. Y "cada frame" no es solo
`Update`: es todo lo que `Update` alcanza.

**Raíces por frame:** `Update`, `LateUpdate`, `FixedUpdate`, `OnGUI`,
`OnAnimatorIK`, `OnAnimatorMove`, `OnWillRenderObject`, `OnPreRender`,
`OnPostRender`, `OnRenderImage`, `OnRenderObject`, los `OnDrawGizmos*`, y el
cuerpo de una corutina que hace `yield return null` o `WaitForEndOfFrame`
dentro de un bucle.

**Caliente** es el cierre transitivo desde esas raíces sobre el grafo de
llamadas que el índice ya construye.

**La profundidad gradúa la severidad.** Esto es lo que evita que el circuito se
vuelva ruido, que es el riesgo real de toda esta entrega:

- Escrito directamente dentro de un método por frame: la severidad que la
  regla declara.
- Alcanzado a dos o tres saltos por llamadas que el índice resuelve: `Block`
  baja a `Consider` y `Consider` baja a `Note`.
- Más lejos, o con la resolución ambigua: `Note`.

`Ask` no la usa ninguna regla de esta familia. Una regla de Unity o tiene un
arreglo mecánico, y entonces deniega, o es criterio, y entonces se anota; no
hay nada en medio que merezca parar el turno para preguntar.

El reparto entre escritura y cierre de turno es el que el Canon ya usa. Por
escritura, las reglas ven el AST del contenido nuevo más el índice para lo que
cruza ficheros. En el cierre, `judge_turn` lo recalcula con todo lo que el
turno dejó en disco.

## Las reglas

Familia `U` en el enum `Rule`, al lado de las `R` deterministas y las `S` del
revisor. Viven en un módulo `rust/sens-canon/src/unity.rs` con la misma forma
que `comments.rs`, y se componen en `judge_change` detrás de `rules.unity`.

Cada mensaje dice el arreglo. Donde el índice puede, nombra lo que el proyecto
ya tiene para reutilizar — el pool que ya existe, el `static readonly int` del
`StringToHash` ya declarado — usando `Target` tal como está.

### Deniegan

Inequívocas, con arreglo mecánico y coste real por frame.

| | |
| --- | --- |
| **U1** | `GetComponent`, `GetComponents`, `GetComponentInChildren` o `GetComponentInParent` en camino caliente sin que el resultado se guarde en un campo. Cachearlo en `Awake`. |
| **U2** | `Camera.main` en camino caliente. Guardar la cámara en un campo. |
| **U3** | `FindObjectOfType`, `FindObjectsOfType`, `FindObjectsByType`, `GameObject.Find`, `GameObject.FindWithTag` o `GameObject.FindGameObjectsWithTag` en camino caliente. Resolverlo una vez en `Awake` o inyectarlo con `[SerializeField]`. |
| **U4** | Asignación por frame en camino caliente: llamada a una extensión de `System.Linq`, `new` de `List`, `Dictionary` o array, concatenación o interpolación de strings, y `new` de tipos de Unity que asignan (`WaitForSeconds`, `Texture2D`, `Material`, `GUIStyle`). Izar a un campo o reusar un buffer. |
| **U5** | Consulta que asigna: `Physics.RaycastAll`, `SphereCastAll`, `OverlapSphere`, o lectura de `Mesh.vertices` y `Mesh.triangles`. La variante `NonAlloc` con un buffer reusado, o `Mesh.GetVertices(list)`. |
| **U6** | Propiedad de `Animator`, `Shader` o `Material` por string en camino caliente. `Animator.StringToHash` o `Shader.PropertyToID` en un campo estático. |
| **U7** | Mensaje de Unity vacío, como `void Update() {}`. Borrarlo: Unity paga la llamada igual. |
| **U8** | `async void` en un MonoBehaviour que no es manejador de un evento. `async Task` o `Awaitable`. |

`new` de una clase del propio proyecto queda fuera de U4 a propósito: es
demasiado ancho y produciría falsos positivos en el primer turno.

### Consider

Deniegan, pero son criterio que una persona puede revocar con una excepción.

- **U9** `Resources.Load`, cuando el proyecto tiene el paquete Addressables.
- **U10** `Instantiate` o `Destroy` en camino caliente. Un pool.
- **U11** `SendMessage` o `BroadcastMessage`. Referencia directa o evento.
- **U12** Comparación `tag ==`, que asigna. `CompareTag`.

### Note

- **U13** Campo `public` serializado donde `[SerializeField] private` dice lo
  mismo y cierra la API.
- **U14** `Debug.Log` con interpolación, o sin guardar, que viaja en la build.
  Si U4 ya ha saltado en esa misma línea por la interpolación, U14 calla: un
  hallazgo por problema.
- **U15** `OnGUI`, que corre varias veces por frame y asigna.

### Fuera de esta entrega

Corutinas sin handle que se escapan, `transform` leído varias veces en un
mismo método caliente, el MonoBehaviour que solo guarda datos y debería ser un
`ScriptableObject`, y todo DOTS, Jobs y Burst. Entran cuando hayamos medido
que las de arriba no producen falsos positivos sobre un proyecto real.

## El texto del Canon

`canon.md` tiene un presupuesto de cuarenta líneas que un test vigila. Se
añade `unity.md` al lado, incluido solo cuando `rules.unity`, con su propio
presupuesto corto y su propio test. Dice la escalera de Unity antes de
escribir: ¿esto son datos, y entonces van en un `ScriptableObject`?, ¿el
proyecto ya tiene este componente?, ¿qué corre por frame y qué no?

El objetivo del texto es que el modelo llegue sabiendo y el Canon casi nunca
tenga que denegar. Una denegación es el último recurso, no el mecanismo.

`VERSION` sube a `v1.2`.

## Tests

Como los módulos que ya existen: proyecto temporal, un `Change`, y se afirma
la regla, la severidad y que el mensaje nombra el arreglo.

Dos que importan más que el resto:

- Un MonoBehaviour idiomático y limpio — componentes cacheados en `Awake`,
  hashes de Animator estáticos, buffers reusados — que no debe producir ni un
  hallazgo.
- Un componente deliberadamente malo que debe producir exactamente el conjunto
  esperado de hallazgos, ni uno más.

Y uno por la graduación: la misma llamada escrita en `Update`, a dos saltos de
`Update` y en un método que nada por frame alcanza, con tres severidades
distintas.

## Lo siguiente

`sens-bench` no tiene tareas de Unity. Sin ellas no se puede medir que el brazo
con Sens gane, que es el método con el que el proyecto sostiene todo lo demás.
Es la entrega inmediatamente posterior a esta, antes del grafo de assets y del
lazo con el editor.
