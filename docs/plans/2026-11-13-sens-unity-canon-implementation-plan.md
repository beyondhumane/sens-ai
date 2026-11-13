# El Canon aprende Unity — Plan de implementación

- **Fecha:** 2026-11-13
- **Diseño de referencia:** [../specs/2026-11-13-sens-unity-canon-design.md](../specs/2026-11-13-sens-unity-canon-design.md)
- **Estado:** Hitos 0, 1 y 2 hechos; el 1, dentro del [plan del rol de diseño](2026-11-13-sens-design-role-implementation-plan.md). Del 2 salieron el caso del fichero que se escribe y la graduación, que pasan al 3 con la primera regla que los usa.

Cada hito se prueba solo y deja el proyecto en un estado defendible. El orden
sigue las dependencias: primero que un proyecto de Unity entre bien en el
índice, luego que el índice entienda quién llama a qué, y solo encima de eso
las reglas.

---

## Decisiones que el diseño dejaba abiertas

| Área | Decisión | Motivo |
|---|---|---|
| Dónde vive la herencia de tipos | Pase posterior en `build::build`, después de recoger símbolos y antes de `Index::assemble` | `extract` trabaja fichero a fichero y no puede resolver una clase base que vive en otro |
| Cómo se recorre el camino caliente | Invertir `Index::raw_references`, que ya guarda el símbolo que llama en `from`, y BFS desde las raíces por frame | El grafo de llamadas ya está construido; no hace falta estructura nueva |
| Severidad al nacer | Las quince reglas nacen en `Note` y se promueven a su severidad de diseño cuando se hayan medido sobre un proyecto Unity real | Una regla que deniega sobre código legítimo rompe la confianza en el circuito entero, y recuperarla cuesta más que esperar |
| Carpetas a saltar | La lista de salto depende de si la raíz es un proyecto Unity | `Library` o `Build` pueden ser carpetas legítimas en un proyecto que no es de Unity |

---

## Hito 0 — Un proyecto de Unity entra en el índice sin ahogarlo

**Objetivo:** indexar `Assets/` y los paquetes embebidos, y nada más.

- `rust/sens-index/src/project.rs`: `unity_version(root)` lee `m_EditorVersion`
  de `ProjectSettings/ProjectVersion.txt`. La presencia del fichero es la
  detección; su contenido es la versión.
- En el mismo módulo, `skipping(root, base)` devuelve el predicado de
  `filter_entry`: la lista base de quien llama, más lo que el editor genera
  cuando la raíz es de Unity — `Library`, `Temp`, `obj`, `Logs`,
  `UserSettings`, `Build`, `Builds`, `MemoryCaptures`, `Recordings`.
  `Library/PackageCache` es el que importa: son miles de `.cs` de los paquetes
  de Unity.
- Lo usan los tres que recorren la raíz, no solo `collect`: `build.rs`,
  `entries.rs` y `reflective.rs`. `entries.rs` es el que se rompía peor, porque
  cada paquete de Unity en `Library/PackageCache` trae su propio
  `package.json` y habría inventado puntos de entrada falsos.
- La raíz nunca se salta por llamarse como una carpeta de la lista, que es lo
  que pasaba hasta ahora con un proyecto en `P:\Builds\MiJuego`.
- `addressables(root)` no se escribe aquí: no la usa nadie hasta el hito 4, y
  el Canon de este repositorio no deja código sin cablear.

**Se prueba:** un proyecto de prueba con `Assets/Scripts/Player.cs`,
`Library/PackageCache/com.unity.x/Foo.cs` y `Temp/Generated.cs`. El índice
contiene el primero y ninguno de los otros dos. En un proyecto que no es de
Unity, un fichero en `Library/` se sigue indexando.

---

## Hito 1 — Los mensajes de Unity son puntos de entrada

**Objetivo:** que `dead_code` y la regla de huérfanos dejen de declarar muerto
medio MonoBehaviour.

- `rust/sens-index/src/lang/csharp.rs`: al declarar un tipo, guardar los
  nombres de su lista de bases. `Extra` gana el campo; `declare` lo traslada
  al símbolo.
- Los métodos con `[MenuItem]`, `[ContextMenu]`,
  `[RuntimeInitializeOnLoadMethod]`, `[InitializeOnLoadMethod]`, `[UnityTest]`,
  `[Test]`, `[SetUp]` o `[TearDown]` se marcan `entry` ya en `extract`: el
  atributo se ve en el propio fichero. `attrs` ya existe y los lee.
- Pase posterior en `build::build`: cerrar transitivamente la herencia dentro
  del proyecto, sembrada con `MonoBehaviour`, `ScriptableObject`, `Editor`,
  `EditorWindow`, `StateMachineBehaviour`, `PropertyDrawer` y
  `ScriptableWizard`, y marcar `entry` en los métodos de esos tipos cuyo
  nombre simple es un mensaje de Unity.
- La lista de mensajes vive en un solo sitio, porque el hito 2 necesita el
  subconjunto que corre por frame.

**Se prueba:** `PlayerController : MonoBehaviour` en un fichero y
`BossController : PlayerController` en otro. `Update` es punto de entrada en
los dos. `orphans::findings` y `dead_code` no devuelven nada sobre ninguno. Un
método privado que nadie llama sigue saliendo como muerto: el hito no silencia
la regla, le enseña quién llama.

---

## Hito 2 — El camino caliente

**Objetivo:** responder, para un símbolo, a qué distancia está de un método que
corre cada frame.

- `rust/sens-canon/src/hot.rs`: construir la adyacencia hacia delante
  invirtiendo `raw_references` — para cada símbolo, cada sitio trae en `from`
  el símbolo que llama — y hacer un BFS desde las raíces por frame declaradas
  en el diseño. Una llamada cuyo nombre comparten varios métodos del proyecto
  no está resuelta: lo que solo se alcanza a través de ella queda lejos.
- Las corutinas: un método que devuelve `IEnumerator` y hace `yield return
  null`, `WaitForEndOfFrame` o `WaitForFixedUpdate` dentro de un bucle es raíz
  por frame.
- El resultado tiene cuatro valores: corre cada frame, a 1–3 llamadas de una
  raíz, más lejos o por una llamada sin resolver, o frío.
- El primer consumidor es `file_outline`: en un proyecto de Unity marca cada
  método que corre cada frame y a cuántas llamadas está, para que el agente lo
  sepa antes de escribir.

**Se prueba:** un método por frame, otros a una, dos, tres y cuatro llamadas,
uno alcanzado por una llamada ambigua y otros que solo alcanza `Start`. Un
ciclo de llamadas entre dos métodos calientes que no cuelga el BFS. Una
corutina que cede cada frame dentro de un bucle frente a una que cede una sola
vez. Y el `file_outline` de un script de Unity con sus marcas.

---

## Hito 3 — Las reglas que deniegan

**Objetivo:** U1 a U8 funcionando, nacidas en `Note`.

- `Rule` gana `U1`..`U15`. `ProjectRules` gana `unity` y la versión del editor.
- `rust/sens-canon/src/unity.rs` con la forma de `comments.rs`: una función
  por regla sobre el AST del contenido nuevo, todas compartiendo el conjunto
  caliente del hito 2, y un `findings` que las compone.
- Se enchufa en `judge_change` detrás de `rules.unity`, al lado de
  `comments::findings`.
- Lo que el hito 2 dejó para su primer consumidor. El caso dentro del fichero
  que se está escribiendo: el índice es anterior a la escritura, así que un
  método nuevo llamado desde `Update` en el mismo fichero no está en el grafo;
  se resuelve con el AST del contenido nuevo, que `build::parse` ya devuelve.
  Y la graduación, en un solo sitio para que ninguna regla la reimplemente:
  corre cada frame deja la severidad que la regla declara, de 1 a 3 llamadas
  baja `Block` a `Consider` y `Consider` a `Note`, más lejos o sin resolver es
  `Note`, y frío no da hallazgo.
- Cada mensaje nombra el arreglo. Donde el índice puede, rellena `Target` con
  lo que el proyecto ya tiene: el pool existente, el `static readonly int` del
  `StringToHash` ya declarado.
- `rust/sens-agent/src/canon/state.rs` persiste los campos nuevos en
  `.sens/canon/rules.json`, con la detección del hito 0 al abrir el proyecto.

**Se prueba:** la misma llamada escrita dentro de `Update`, en un método que
`Update` llama y en uno que solo alcanza `Start`, con tres resultados
distintos. Un MonoBehaviour idiomático y limpio — componentes cacheados en
`Awake`, hashes estáticos, buffers reusados — que no produce ni un hallazgo. Y
un componente deliberadamente malo que produce exactamente el conjunto
esperado, ni uno más. Las dos afirmaciones son el test, no un comentario.

---

## Hito 4 — Las reglas de criterio

**Objetivo:** U9 a U15.

- `addressables(root)` en `project.rs`, leyendo `Packages/manifest.json`. U9
  solo habla si es cierto.
- U12 distingue `tag ==` de `CompareTag`.
- U14 calla si U4 ya saltó en esa línea: un hallazgo por problema.
- U15 señala `OnGUI` como declaración, no como llamada.

**Se prueba:** un componente con `Resources.Load` en un proyecto sin el paquete
no produce U9, y en uno con el paquete sí. Un `Debug.Log($"{x}")` dentro de
`Update` produce un hallazgo, no dos.

---

## Hito 5 — El texto del Canon

**Objetivo:** que el modelo llegue sabiendo.

- `rust/sens-canon/src/unity.md`, incluido con `include_str!` y expuesto como
  una constante al lado de `CANON`.
- `rust/sens-agent`: cuando `rules.unity`, el texto se añade al Canon que se
  le da al modelo.
- Tests propios, con la misma forma que los de `canon.md`: presupuesto de
  líneas, y que la escalera de Unity aparece en orden.
- `VERSION` sube a `v1.2`.

**Se prueba:** los tests del presupuesto y del orden. Y que un proyecto que no
es de Unity recibe exactamente el texto que recibía antes.

---

## Hito 6 — La casilla

**Objetivo:** poder apagarlo.

- `rust/sens-app/src/canon.rs` ya expone `canon_rules` y `canon_set_rules`; los
  campos nuevos viajan solos.
- `rust/sens-app/ui/src/features/settings/project.copy.ts`: copy en los seis
  idiomas. La casilla solo se dibuja si se ha detectado Unity.
- `rust/sens-app/ui/src/dev/mock-tauri.ts`: el mock devuelve los campos nuevos.

**Se prueba:** los tests de jsdom que ya cubren el panel de proyecto, más uno
que afirma que la casilla no está cuando el proyecto no es de Unity.

---

## Hito 7 — Calibrar y promover

**Objetivo:** convertir las quince `Note` en las severidades del diseño, con
una medida detrás.

- Correr el circuito sobre un proyecto de Unity real y recoger todo lo que las
  reglas dicen, sin que nada deniegue.
- Revisar a mano cada hallazgo, como el paper hace con cada parada del
  circuito, y clasificarlo en acertado o falso positivo.
- Una regla sube a su severidad de diseño cuando no produce falsos positivos
  en esa pasada. La que los produzca se arregla o se queda en `Note`.
- El resultado se escribe en el propio spec, al lado de cada regla.

**Se prueba:** a mano, y es el punto del hito. Hace falta un proyecto de Unity
real para esto; sin él, el hito no se puede cerrar y las reglas se quedan en
`Note`.

---

## Lo que no entra

El grafo de assets por GUID y el lazo con el editor quedan para más adelante.
Las tareas de Unity en `sens-bench` van justo después de este plan: sin ellas
la afirmación de que el brazo con Sens gana no está medida. Y el README, la
web y el paper se reescriben cuando el hito 7 cierre, no antes.

## El riesgo

Es uno y es el mismo en todos los hitos: un falso positivo que deniega código
legítimo. Un agente al que el circuito para sin razón acaba peleándose con él,
y una persona que ve una denegación injusta deja de confiar en las justas. De
ahí que todo nazca en `Note`, que la profundidad gradúe, que el test del
componente limpio sea tan importante como el del malo, y que el hito 7 exista.
