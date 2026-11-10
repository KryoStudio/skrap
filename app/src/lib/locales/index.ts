// The languages Skrap speaks. To add one: copy en.ts to <code>.ts, translate it, add a line
// below, and give the rules in rules/*.toml their texts in it (docs/RULES.md). Until it is
// complete, type it `Partial<Strings>`: the missing texts show in English.
import { en } from './en';
import { fr } from './fr';

export type Key = keyof typeof en;
export type Strings = Record<Key, string>;
/** A complete language: every key, plus the plural forms it needs beyond English's. */
export type Translation = Strings & Record<string, string>;

export type Language = {
  /** In the language itself. */
  name: string;
  strings: Partial<Strings>;
  /** The exact locale for number and date formats, when the code alone is ambiguous. */
  locale?: string;
};

const table = {
  en: { name: 'English', strings: en },
  fr: { name: 'Français', strings: fr, locale: 'fr-FR' },
} satisfies Record<string, Language>;

export type Lang = keyof typeof table;

export const languages: Record<Lang, Language> = table;

/** English is the reference: every key exists there. */
export const fallback: Strings = en;
