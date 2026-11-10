// In-app updates: signed builds published on GitHub Releases (.github/workflows/release.yml).
//
// At launch, when a new version is out, it is offered in the middle of the screen. "Later" puts
// that version off for a day; the title bar keeps a small reminder. Installing downloads, checks
// the signature against the key in tauri.conf.json, installs and restarts.
import { relaunch } from '@tauri-apps/plugin-process';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { native } from './api';
import { t } from './i18n.svelte';
import { toast } from './ui.svelte';

const LATER_KEY = 'skrap.update-later';
const LATER_MS = 24 * 60 * 60 * 1000;

/** Whether `version` was put off less than a day ago. */
export function postponed(saved: string | null, version: string, now: number): boolean {
  if (!saved) return false;
  try {
    const later = JSON.parse(saved) as { version?: unknown; until?: unknown };
    return later.version === version && typeof later.until === 'number' && now < later.until;
  } catch {
    return false;
  }
}

/** The release notes' first real paragraph, without what only concerns a download from the
 *  release page (the installer's name, its checksum, the virus scan). */
export function notesSummary(notes: string | null | undefined, max = 240): string {
  if (!notes) return '';
  const download = /setup\.exe|sha-?256|virustotal|checksum|download/i;
  const paragraph = notes
    .replace(/\r/g, '')
    .split(/\n\s*\n/)
    .map((p) =>
      p
        .replace(/!?\[([^\]]*)\]\([^)]*\)/g, '$1')
        .replace(/^\s{0,3}(#{1,6}\s+|[-*+]\s+|>\s?)/gm, '')
        .replace(/[*_`~]/g, '')
        .replace(/\s+/g, ' ')
        .trim(),
    )
    .find((p) => p && !download.test(p));
  if (!paragraph) return '';
  return paragraph.length > max ? `${paragraph.slice(0, max - 1).trimEnd()}…` : paragraph;
}

class Updates {
  available = $state<Update | null>(null);
  /** The dialog is up. */
  open = $state(false);
  checking = $state(false);
  /** Download progress 0..1, then 1 while installing; null otherwise. */
  installing = $state<number | null>(null);

  /** The new version and its notes (set by a check, or by the browser preview). */
  version = $state('');
  private notes = $state<string | null>(null);
  readonly summary = $derived(notesSummary(this.notes));

  /** At launch (`silent`): offers what it finds unless put off. From the settings: answers. */
  async check(silent: boolean) {
    if (!native || this.checking || this.installing !== null) return;
    this.checking = true;
    try {
      this.available = await check();
      const version = this.available?.version;
      this.version = version ?? '';
      this.notes = this.available?.body ?? null;
      if (!version) {
        if (!silent) toast({ kind: 'success', title: t('update.upToDate') });
      } else if (!silent || !postponed(read(LATER_KEY), version, Date.now())) {
        this.open = true;
      }
    } catch (e) {
      console.warn('update check failed', e);
      if (!silent) toast({ kind: 'error', title: t('update.failed') });
    } finally {
      this.checking = false;
    }
  }

  /** Puts this version off for a day. */
  later() {
    if (this.installing !== null) return;
    this.open = false;
    const version = this.version;
    if (version) write(LATER_KEY, JSON.stringify({ version, until: Date.now() + LATER_MS }));
  }

  /** The browser preview: a pretend update, installed with a pretend download. */
  preview() {
    this.version = '0.2.0';
    this.notes = 'Disk analysis arrives: a map of what takes up space, read in seconds.\n\n**SHA-256**: …';
    this.open = true;
  }

  async install() {
    if (!native && this.version) {
      this.installing = 0;
      const timer = setInterval(() => {
        this.installing = Math.min(1, (this.installing ?? 0) + 0.04);
        if (this.installing >= 1) clearInterval(timer);
      }, 60);
      return;
    }
    const update = this.available;
    if (!update || this.installing !== null) return;
    this.open = true;
    this.installing = 0;
    let total = 0;
    let done = 0;
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') total = event.data.contentLength ?? 0;
        else if (event.event === 'Progress') {
          done += event.data.chunkLength;
          if (total) this.installing = Math.min(0.99, done / total);
        } else if (event.event === 'Finished') this.installing = 1;
      });
      // The installer usually closes Skrap itself; if not, restart into the new version.
      await relaunch();
    } catch (e) {
      console.warn('update install failed', e);
      toast({ kind: 'error', title: t('update.failed') });
      this.installing = null;
    }
  }
}

function read(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function write(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {
    // No storage: the offer comes back at the next launch.
  }
}

export const updates = new Updates();
