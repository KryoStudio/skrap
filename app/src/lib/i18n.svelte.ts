import { fallback, languages, type Key, type Lang } from './locales';

export type { Key, Lang };

/** The first of the PC's preferred languages that Skrap speaks, else English. */
function detect(): Lang {
  const tags = typeof navigator === 'undefined' ? [] : navigator.languages?.length ? navigator.languages : [navigator.language];
  for (const tag of tags) {
    const base = tag.toLowerCase().split('-')[0];
    if (Object.hasOwn(languages, base)) return base as Lang;
  }
  return 'en';
}

export const systemLang: Lang = detect();

export const i18n = $state({ lang: systemLang as Lang });

/** A saved language this version does not have (removed, or from a newer one) → Windows'. */
export const resolveLang = (code: string | null | undefined): Lang =>
  code && Object.hasOwn(languages, code) ? (code as Lang) : systemLang;

/** BCP 47 locale of the current language, for Intl formats. */
export const locale = (): string => languages[i18n.lang].locale ?? i18n.lang;

export const hasKey = (key: string): key is Key => key in fallback;

/** Translates `key`, filling `{name}` placeholders. With a numeric `count`, picks the plural
 *  variant (`<key>.one`…) when there is one. Missing → English. */
export function t(key: Key, vars: Record<string, string | number> = {}): string {
  const table: Record<string, string | undefined> = languages[i18n.lang].strings;
  const base: Record<string, string> = fallback;
  let id: string = key;
  if (typeof vars.count === 'number') {
    const category = new Intl.PluralRules(locale()).select(vars.count);
    if (category !== 'other' && (table[`${key}.${category}`] ?? base[`${key}.${category}`])) id = `${key}.${category}`;
  }
  const text = table[id] ?? base[id] ?? key;
  return text.replace(/\{(\w+)\}/g, (_, name) => {
    const value = vars[name];
    return typeof value === 'number' ? formatNumber(value) : String(value ?? '');
  });
}

export function formatNumber(value: number, digits = 0): string {
  return value.toLocaleString(locale(), { maximumFractionDigits: digits, minimumFractionDigits: digits });
}

const UNITS = ['unit.b', 'unit.kb', 'unit.mb', 'unit.gb', 'unit.tb'] as const;

/** A size split for display, as File Explorer counts it (1 KB = 1,024 bytes): "4,2" + "Go". */
export function sizeParts(bytes: number): { value: string; unit: string } {
  let value = Math.max(0, bytes);
  let unit = 0;
  while (value >= 1000 && unit < UNITS.length - 1) {
    value /= 1024;
    unit++;
  }
  // Two digits under 10 GB-like values read better as "4,2"; bytes and KB need none.
  const digits = unit <= 1 ? 0 : value < 10 ? 1 : value < 100 ? 1 : 0;
  return { value: formatNumber(value, digits), unit: t(UNITS[unit]) };
}

export function formatSize(bytes: number): string {
  const { value, unit } = sizeParts(bytes);
  return `${value} ${unit}`;
}

/** "0,6 s", "12 s", "1 min 4 s". */
export function formatDuration(ms: number): string {
  const seconds = ms / 1000;
  if (seconds < 10) return `${formatNumber(seconds, 1)} s`;
  if (seconds < 60) return `${Math.round(seconds)} s`;
  const minutes = Math.floor(seconds / 60);
  return `${minutes} min ${Math.round(seconds % 60)} s`;
}

/** "3 minutes ago", "yesterday", then a date. */
export function formatAgo(timestamp: number, now = Date.now()): string {
  const minutes = Math.round((now - timestamp) / 60_000);
  const rtf = new Intl.RelativeTimeFormat(locale(), { numeric: 'auto' });
  if (minutes < 1) return rtf.format(0, 'second');
  if (minutes < 60) return rtf.format(-minutes, 'minute');
  if (minutes < 60 * 24) return rtf.format(-Math.round(minutes / 60), 'hour');
  if (minutes < 60 * 24 * 7) return rtf.format(-Math.round(minutes / (60 * 24)), 'day');
  return formatDate(timestamp);
}

/** "17 November" (with the year when it is not this one). */
export function formatDate(timestamp: number): string {
  const date = new Date(timestamp);
  const sameYear = date.getFullYear() === new Date().getFullYear();
  return date.toLocaleDateString(locale(), { day: 'numeric', month: 'long', ...(sameYear ? {} : { year: 'numeric' }) });
}

export function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString(locale(), { hour: '2-digit', minute: '2-digit' });
}

/** "Today, 14:32", "Yesterday, 09:10", else "12 November, 14:32". */
export function formatMoment(timestamp: number, now = Date.now()): string {
  const day = (ms: number) => new Date(ms).toDateString();
  const time = formatTime(timestamp);
  if (day(timestamp) === day(now)) return t('reserve.today', { time });
  if (day(timestamp) === day(now - 86_400_000)) return t('reserve.yesterday', { time });
  return `${formatDate(timestamp)}, ${time}`;
}
