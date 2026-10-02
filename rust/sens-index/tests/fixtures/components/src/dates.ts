export function formatDate(iso: string): string {
  return iso.split("-").reverse().join("/");
}
