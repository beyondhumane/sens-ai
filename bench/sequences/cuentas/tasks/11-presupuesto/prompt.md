Quiero ponerme un presupuesto al mes por categoría: `presupuesto <fichero-de-presupuestos> <categoría> <importe>`.

- Se guarda en un CSV con cabecera `categoria,importe` y líneas como `Comida,100.00`. Si el fichero no existe, se crea.
- Si la categoría ya tenía presupuesto, se cambia en su sitio. Las categorías se comparan sin distinguir mayúsculas ni acentos, y se queda el nombre que acabas de escribir.
- El importe se escribe y se valida como en `añadir`.

Responde `Presupuesto de Comida: 100,00 € al mes`.
