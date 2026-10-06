export const locales = ["en", "es", "fr", "de", "ja", "zh"] as const;

export type Locale = (typeof locales)[number];

export type Translated = Exclude<Locale, "en">;

export const translated = locales.filter((locale): locale is Translated => locale !== "en");

export const names: Record<Locale, string> = {
  en: "English",
  es: "Español",
  fr: "Français",
  de: "Deutsch",
  ja: "日本語",
  zh: "简体中文",
};

export const htmlLang: Record<Locale, string> = {
  en: "en",
  es: "es",
  fr: "fr",
  de: "de",
  ja: "ja",
  zh: "zh-Hans",
};

export const ogLocale: Record<Locale, string> = {
  en: "en_GB",
  es: "es_ES",
  fr: "fr_FR",
  de: "de_DE",
  ja: "ja_JP",
  zh: "zh_CN",
};

export const dateLocale: Record<Locale, string> = {
  en: "en-GB",
  es: "es-ES",
  fr: "fr-FR",
  de: "de-DE",
  ja: "ja-JP",
  zh: "zh-CN",
};

const base = (): string => (import.meta.env?.BASE_URL ?? "/").replace(/\/$/, "");

export const withBase = (path: string): string => `${base()}${path}`;

export const pathFor = (locale: Locale, path: string): string => withBase(locale === "en" ? path : `/${locale}${path}`);

export function copy<T>(table: { en: T } & { [Language in Translated]: NoInfer<T> }): (locale: Locale) => T {
  return (locale) => table[locale];
}

export const dayIn = (locale: Locale, isoDay: string): string =>
  new Date(`${isoDay}T00:00:00Z`).toLocaleDateString(dateLocale[locale], { day: "numeric", month: "long", year: "numeric", timeZone: "UTC" });

export const routes = () => translated.map((lang) => ({ params: { lang } }));
