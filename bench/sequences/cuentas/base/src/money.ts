const euros = new Intl.NumberFormat("es-ES", { style: "currency", currency: "EUR" });

export function formatMoney(cents: number): string {
  return euros.format(cents / 100);
}
