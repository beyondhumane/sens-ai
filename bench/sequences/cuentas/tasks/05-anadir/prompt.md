Quiero apuntar gastos sin abrir el CSV: `añadir <fichero> <fecha> <importe> <categoría> <descripción>`.

- La fecha va como 2026-03-19 y tiene que existir.
- El importe va con punto decimal, mayor que cero y con dos decimales como mucho.
- El gasto se escribe al final del fichero, como los demás (`2026-03-19,4.20,Comida,Churros`).

Si va bien responde `Añadido: 19/03/2026  Comida  4,20 €  Churros`. Si algo no vale, no toca el fichero y lo dice: `Fecha no válida: 2026-02-30` o `Importe no válido: -3`.
