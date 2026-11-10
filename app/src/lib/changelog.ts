// What each version brings, shown once after an update ("What's new") and in the settings.
// Newest first. Every version needs its lines in every language the app speaks.
import type { Lang } from './locales';

export interface Release {
  version: string;
  notes: Record<Lang, string[]>;
}

export const changelog: Release[] = [
  {
    version: '0.1.0',
    notes: {
      en: [
        'First version: one Scan button, every item with its size and the reason it can go.',
        'System, games and launchers, browsers and apps: 35 rules, none of them forced.',
        'The Reserve keeps what does not rebuild itself for 7 days, back in one click.',
        'Your Documents, Desktop, Pictures, Videos, game saves and cloud folders are never touched.',
      ],
      fr: [
        'Première version : un bouton Scanner, chaque élément avec sa taille et pourquoi il peut partir.',
        'Système, jeux et launchers, navigateurs et applis : 35 règles, aucune forcée.',
        'La Réserve garde 7 jours ce qui ne se recrée pas tout seul, de retour en un clic.',
        'Tes Documents, Bureau, Images, Vidéos, sauvegardes de jeux et dossiers cloud ne sont jamais touchés.',
      ],
    },
  },
];

export function releaseNotes(version: string): Release | undefined {
  return changelog.find((release) => release.version === version);
}
