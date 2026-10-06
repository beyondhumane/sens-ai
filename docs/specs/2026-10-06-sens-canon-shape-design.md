# La forma del proyecto en el Canon

Entrega 2 de 3, sobre el `Map` de la entrega 1
(`2026-10-06-sens-project-map-design.md`).

## Para qué

Que el Canon vea, al cerrar un turno, cuándo el cambio empeora la estructura
del proyecto y no solo el código que toca: si deja dos ficheros importándose
el uno al otro, o si hace que un área dependa de otra con la que no tenía nada.

## Las dos reglas

- **R9, ciclo de imports nuevo** (`Consider`): el turno cierra un ciclo que no
  existía, o hace crecer uno existente. Frena una vez con la explicación y los
  ficheros del ciclo; si el modelo lo mantiene tras verlo, pasa y queda en el
  registro como `Kept`. Cuenta en "ciclos de imports parados".
- **R10, acoplamiento nuevo entre áreas** (`Note`): un área pasa a usar otra que
  no usaba. Solo se anota en la revisión del turno, con la puerta del área
  usada, porque cruzar áreas suele ser legítimo.

## Qué es nuevo

El circuito guarda la forma del proyecto (`Shape`: áreas, pares de áreas que
se usan y ciclos) en `.sens/canon/state.json`, en el mismo momento en que fija
la línea base del código muerto: en el primer mensaje y cada vez que aprueba un
turno. Al cerrar el turno, `judge_turn` compara la forma actual con esa.

- Un ciclo es nuevo si sus ficheros no caben dentro de ningún ciclo anterior.
- Un acoplamiento es nuevo si el par no estaba y las dos áreas ya existían
  antes. Un área recién creada, o una carpeta que se divide al crecer, no
  dispara notas.
- Sin línea base (estado de una versión anterior), las dos reglas callan hasta
  la siguiente aprobación.

## Dónde se buscan ciclos

Solo en TypeScript/JavaScript y Python, y sin ficheros de test. Ahí un ciclo
hace daño: orden de carga, valores `undefined` al importar, `ImportError`. En
Rust, C#, Java o Kotlin los módulos de un mismo crate o paquete se usan entre sí
con normalidad. En el propio repo de Sens había 7 ciclos y 5 eran de Rust: con
esos lenguajes la regla frenaría cada módulo nuevo.

El grafo es el de los imports resueltos, sin las aristas por nombre único que
usa el mapa: un ciclo tiene que ser un ciclo de imports de verdad.

## Pruebas

- `map.rs`: el ciclo de tres ficheros se encuentra; el de Rust y el test no.
- `shape.rs`: un ciclo nuevo se frena y el ya conocido no; un acoplamiento
  nuevo se anota con la puerta y un área nueva no.
- `circuit.rs`: el turno que cierra un ciclo se frena, pasa si el modelo
  insiste y queda registrado como `Kept`.
- UI: el contador de ciclos parados en la sección del proyecto.
