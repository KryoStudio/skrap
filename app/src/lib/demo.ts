// A stand-in for the Rust side in a plain browser (`npm run dev`): plausible rules, a scan that
// takes its time, a clean, a Reserve. Nothing here touches a file.
import type { Api, Batch, CleanDone, Events, Finding, Progress, RuleView, Settings } from './api';

type Handler<K extends keyof Events> = (payload: Events[K]) => void;

const listeners = new Map<string, Set<Handler<keyof Events>>>();

export const demoEvents = {
  on<K extends keyof Events>(event: K, handler: Handler<K>): () => void {
    const set = listeners.get(event) ?? new Set();
    set.add(handler as Handler<keyof Events>);
    listeners.set(event, set);
    return () => set.delete(handler as Handler<keyof Events>);
  },
  emit<K extends keyof Events>(event: K, payload: Events[K]) {
    listeners.get(event)?.forEach((handler) => handler(payload));
  },
};

const MB = 1024 ** 2;
const DAY = 86_400_000;

const rules: (RuleView & { bytes: number; files: number; running?: string })[] = [
  { id: 'system.windows.user_temp', category: 'system', risk: 'safe', group: 'Windows', name: 'Temporary files', explain: 'What programs leave behind while they work. Only files untouched for a day go.', ticked: true, admin: false, special: false, bytes: 1640 * MB, files: 8211 },
  { id: 'system.windows.thumbnails', category: 'system', risk: 'safe', group: 'Windows', name: 'Thumbnail cache', explain: 'The previews File Explorer shows for photos and videos. Rebuilt as you browse.', ticked: true, admin: false, special: false, bytes: 212 * MB, files: 9 },
  { id: 'system.windows.update_cache', category: 'system', risk: 'safe', group: 'Windows Update', name: 'Downloaded updates', explain: 'Update files Windows already installed.', ticked: true, admin: true, special: false, bytes: 880 * MB, files: 41 },
  { id: 'system.windows.recycle_bin', category: 'system', risk: 'review', group: 'Windows', name: 'Recycle Bin', explain: 'What you already deleted. Emptied for good: it does not go through the Reserve.', ticked: false, admin: false, special: true, bytes: 67 * MB, files: 12 },
  { id: 'gaming.nvidia.shader_cache', category: 'gaming', risk: 'review', group: 'NVIDIA', name: 'Shader cache', explain: 'Shaders the NVIDIA driver compiled for your games. The next launch of each game can stutter for a moment.', ticked: false, admin: false, special: false, bytes: 2680 * MB, files: 1904 },
  { id: 'gaming.steam.web_cache', category: 'gaming', risk: 'safe', group: 'Steam', name: 'Store and community cache', explain: 'Pages and images of the Steam store, library and community.', ticked: true, admin: false, special: false, bytes: 120 * MB, files: 3550 },
  { id: 'gaming.epic.web_cache', category: 'gaming', risk: 'safe', group: 'Epic Games', name: 'Launcher cache', explain: 'Pages and images of the Epic Games store.', ticked: true, admin: false, special: false, bytes: 118 * MB, files: 2210 },
  { id: 'browsers.chrome.cache', category: 'browsers', risk: 'safe', group: 'Chrome', name: 'Cache', explain: 'Copies of pages, images and scripts from the sites you visited. You stay signed in.', ticked: true, admin: false, special: false, bytes: 873 * MB, files: 6120 },
  { id: 'browsers.edge.cache', category: 'browsers', risk: 'safe', group: 'Edge', name: 'Cache', explain: 'Copies of pages, images and scripts from the sites you visited. You stay signed in.', ticked: true, admin: false, special: false, bytes: 115 * MB, files: 2400 },
  { id: 'apps.discord.cache', category: 'apps', risk: 'safe', group: 'Discord', name: 'Cache', explain: 'Images, GIFs and videos seen in your chats.', ticked: true, admin: false, special: false, bytes: 679 * MB, files: 3011, running: 'Discord.exe' },
];

/** `?demo=<screen>` in the address opens the preview there: welcome (default), home, scanning,
 *  results, cleaning, done, update, whatsnew. `&still` turns animations off (screenshots). */
const params = new URLSearchParams(typeof location === 'undefined' ? '' : location.search);
export const demoScreen = params.get('demo') ?? 'welcome';

let settings: Settings = {
  language: null,
  checkUpdates: true,
  reserveDays: 7,
  motion: params.has('still') ? 'reduced' : 'auto',
  onboarded: demoScreen !== 'welcome',
  lastSeenVersion: demoScreen === 'whatsnew' ? '0.0.9' : '0.1.0',
};
let stats = { cleanedBytes: 12.4 * 1024 * MB, cleans: 6, lastScan: Date.now() - 2 * DAY, lastClean: Date.now() - 2 * DAY };
let reserve: Batch[] = [
  { id: 'demo-1', created: Date.now() - 2 * DAY, bytes: 1250 * MB, files: 812, rules: ['gaming.nvidia.shader_cache'], expires: Date.now() + 5 * DAY },
];
let found: Finding[] = [];
let cancelled = false;

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

function progress(files: number, bytes: number, done: number, total: number, current: string): Progress {
  return { files, bytes, stepsDone: done, stepsTotal: total, current };
}

async function runScan() {
  cancelled = false;
  found = [];
  const started = Date.now();
  let files = 0;
  let bytes = 0;
  for (const [i, rule] of rules.entries()) {
    const steps = 8;
    for (let s = 1; s <= steps; s++) {
      if (cancelled) break;
      await wait(32);
      files += Math.round(rule.files / steps);
      bytes += rule.bytes / steps;
      demoEvents.emit('scan-progress', progress(files, bytes, i, rules.length, `C:\\Users\\you\\AppData\\Local\\${rule.group}\\Cache\\${(s * 7919).toString(16)}`));
    }
    const finding: Finding = { rule: rule.id, bytes: rule.bytes, files: rule.files, skip: rule.running ? { kind: 'running', exe: rule.running } : null };
    found.push(finding);
    demoEvents.emit('scan-found', finding);
  }
  stats = { ...stats, lastScan: started };
  demoEvents.emit('scan-done', { findings: found, started, durationMs: Date.now() - started, cancelled });
}

async function runClean(ids: string[]) {
  cancelled = false;
  let bytes = 0;
  let files = 0;
  let reserved = 0;
  let freed = 0;
  for (const [i, id] of ids.entries()) {
    if (cancelled) break;
    const rule = rules.find((r) => r.id === id);
    if (!rule) continue;
    for (let s = 1; s <= 6; s++) {
      await wait(45);
      bytes += rule.bytes / 6;
      files += Math.round(rule.files / 6);
      demoEvents.emit('clean-progress', progress(files, bytes, i, ids.length, `C:\\Users\\you\\AppData\\Local\\${rule.group}\\${s}.bin`));
    }
    if (rule.risk === 'safe' || rule.special) freed += rule.bytes;
    else reserved += rule.bytes;
  }
  demoEvents.emit('clean-progress', progress(files, bytes, ids.length, ids.length, ''));
  const until = Date.now() + settings.reserveDays * DAY;
  if (reserved > 0) reserve = [{ id: `demo-${Date.now()}`, created: Date.now(), bytes: reserved, files: 300, rules: ids, expires: until }, ...reserve];
  stats = { ...stats, cleanedBytes: stats.cleanedBytes + freed + reserved, cleans: stats.cleans + 1, lastClean: Date.now() };
  const done: CleanDone = {
    freed,
    reserved,
    files,
    batch: reserved > 0 ? 'demo' : null,
    failures: [{ rule: 'system.windows.user_temp', reason: 'in_use', files: 14, bytes: 38 * MB, exe: null }],
    durationMs: 900,
    cancelled,
    reserveUntil: reserved > 0 ? until : null,
  };
  demoEvents.emit('clean-done', done);
}

export const demo: Api = {
  getSettings: async () => settings,
  updateSettings: async (patch) => (settings = { ...settings, ...patch }),
  getOverview: async () => ({
    version: '0.1.0',
    elevated: false,
    disks: [
      { root: 'C:\\', label: 'Windows', total: 1000 * 1024 ** 3, free: 212 * 1024 ** 3, system: true },
      { root: 'D:\\', label: 'Jeux', total: 2000 * 1024 ** 3, free: 640 * 1024 ** 3, system: false },
    ],
    stats,
    reserve: {
      bytes: reserve.reduce((sum, b) => sum + b.bytes, 0),
      files: reserve.reduce((sum, b) => sum + b.files, 0),
      batches: reserve.length,
      nextRelease: reserve.length ? Math.min(...reserve.map((b) => b.expires)) : null,
    },
  }),
  getRules: async () => rules.map(({ bytes: _b, files: _f, running: _r, ...rule }) => rule),
  startScan: async () => void runScan(),
  startClean: async (ids) => void runClean(ids),
  cancelJob: async () => void (cancelled = true),
  getRuleFiles: async (rule, limit) => {
    const r = rules.find((x) => x.id === rule);
    const count = Math.min(limit, r?.files ?? 0);
    return {
      total: r?.files ?? 0,
      files: Array.from({ length: count }, (_, i) => ({
        path: `C:\\Users\\you\\AppData\\Local\\${r?.group ?? 'App'}\\Cache\\data_${(i * 2654435761).toString(16).slice(0, 6)}`,
        size: Math.round(((r?.bytes ?? 0) / Math.max(1, r?.files ?? 1)) * (count - i) * 1.4),
      })),
    };
  },
  getReserve: async () => reserve,
  getReserveFiles: async (batch, limit) =>
    Array.from({ length: Math.min(limit, 12) }, (_, i) => ({ path: `C:\\Users\\you\\AppData\\Local\\NVIDIA\\DXCache\\${batch}_${i}.toc`, size: (12 - i) * 4 * MB, rule: 'gaming.nvidia.shader_cache' })),
  restoreBatch: async (batch) => {
    const b = reserve.find((x) => x.id === batch);
    reserve = reserve.filter((x) => x.id !== batch);
    demoEvents.emit('reserve-changed', 0);
    return { files: b?.files ?? 0, bytes: b?.bytes ?? 0, failures: [] };
  },
  releaseBatch: async (batch) => {
    const freed = reserve.filter((x) => batch === null || x.id === batch).reduce((s, b) => s + b.bytes, 0);
    reserve = batch === null ? [] : reserve.filter((x) => x.id !== batch);
    demoEvents.emit('reserve-changed', freed);
    return freed;
  },
  reveal: async () => {},
};
