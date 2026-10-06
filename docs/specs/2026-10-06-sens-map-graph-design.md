# El grafo del mapa

Entrega 3 de 3, sobre el `Map` de la entrega 1 y los ciclos de la entrega 2.

## Para qué

Que una persona que abre un proyecto que no conoce vea de un vistazo qué está en
el centro, qué está aislado y qué está enredado. Las listas de la pestaña Mapa
sirven para buscar algo concreto; el grafo sirve para orientarse. Al modelo no le
aporta nada: él ya recibe el mapa en texto.

## Qué se ve

La pestaña Mapa tiene un selector Lista / Grafo (se recuerda entre arranques).
En Grafo:

- Cada área es un nodo con su nombre (las dos últimas carpetas; el nombre entero
  en el título) y su número de ficheros.
- Cada dependencia entre áreas es una flecha del área que usa a la usada, más
  gruesa cuanto más la usa.
- Las áreas con un fichero en un ciclo de imports llevan el icono `repeat` de
  Lucide en ámbar, y su nombre accesible lo dice.
- Pulsar un área (o Intro / Espacio sobre ella) la selecciona: ella y sus
  flechas toman el color de foco, el resto se atenúa, y debajo aparece su ficha
  con los ficheros, desde la que se ve el impacto de cada uno como en la lista.
- Debajo, en los dos modos, la lista de ciclos de imports con sus ficheros.

## Disposición

Por capas y sin dependencias nuevas: un área que no usa ninguna otra va abajo, y
cada área va una capa por encima de la más alta de las que usa. Dentro de una
capa, cada área se coloca hacia la media de las posiciones de lo que usa. Un
ciclo entre áreas no la rompe: la visita en curso no cuenta. Es determinista: el
mismo mapa se dibuja igual.

Una flecha que salta capas se curva hacia un lado para no pasar por detrás de
los nodos intermedios; una entre áreas de la misma capa se dibuja como un arco.

## Identidad

Según la sección de visualización de datos de la guía: nodos neutros con las
superficies del tema, color de foco solo en lo seleccionado y sus flechas,
atenuación fuerte de lo demás, ámbar con icono para los ciclos (nunca color
solo), sin paleta por categorías. Probado en oscuro y en claro.

## Datos

`canon_map` añade `links` (área que usa, área usada, peso) y `cycles` (los
ficheros de cada ciclo). `Area.uses` en `sens-index` lleva ahora el peso de cada
dependencia junto al área.

## Pruebas

- `layout.test.ts`: capas, ciclos entre áreas, enlaces a áreas que no se
  dibujan, determinismo, nombre corto.
- `MapPanel.test.tsx`: el grafo, la selección con ratón y teclado, la flecha
  iluminada, la ficha del área, el modo recordado, y el aviso de ciclo.
- `view.rs`: `links` y `cycles` en la vista.
