// The bridge to the Rust side (src-tauri/src/commands.rs). In a plain browser (`npm run dev`
// without Tauri), a demo stands in for it so the whole interface can be tried.
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { demo, demoEvents } from './demo';

export const native = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export type Category = 'system' | 'gaming' | 'browsers' | 'apps';
export type Risk = 'safe' | 'review' | 'expert';
export type Motion = 'auto' | 'reduced';

export interface Settings {
  language: string | null;
  checkUpdates: boolean;
  reserveDays: number;
  motion: Motion;
  onboarded: boolean;
  lastSeenVersion: string | null;
}

export type SettingsPatch = Partial<Settings>;

export interface Disk {
  root: string;
  label: string;
  total: number;
  free: number;
  system: boolean;
}

export interface Stats {
  cleanedBytes: number;
  cleans: number;
  lastScan: number | null;
  lastClean: number | null;
}

export interface Overview {
  version: string;
  elevated: boolean;
  disks: Disk[];
  stats: Stats;
  reserve: { bytes: number; files: number; batches: number; nextRelease: number | null };
}

export interface RuleView {
  id: string;
  category: Category;
  risk: Risk;
  group: string;
  name: string;
  explain: string;
  ticked: boolean;
  admin: boolean;
  special: boolean;
}

export type Skip = { kind: 'running'; exe: string };

export interface Finding {
  rule: string;
  bytes: number;
  files: number;
  skip: Skip | null;
}

export interface ScanReport {
  findings: Finding[];
  started: number;
  durationMs: number;
  cancelled: boolean;
}

export interface Progress {
  files: number;
  bytes: number;
  stepsDone: number;
  stepsTotal: number;
  current: string;
}

export type FailReason = 'in_use' | 'denied' | 'changed' | 'protected' | 'no_reserve' | 'exists' | 'running' | 'other';

export interface Failure {
  rule: string;
  reason: FailReason;
  files: number;
  bytes: number;
  exe: string | null;
}

export interface CleanDone {
  freed: number;
  reserved: number;
  files: number;
  batch: string | null;
  failures: Failure[];
  durationMs: number;
  cancelled: boolean;
  reserveUntil: number | null;
}

export interface FileView {
  path: string;
  size: number;
}

export interface RuleFiles {
  files: FileView[];
  total: number;
}

export interface Batch {
  id: string;
  created: number;
  bytes: number;
  files: number;
  rules: string[];
  expires: number;
}

export interface ReservedFile {
  path: string;
  size: number;
  rule: string;
}

export interface RestoreReport {
  files: number;
  bytes: number;
  failures: [string, FailReason][];
}

export interface Events {
  'scan-progress': Progress;
  'scan-found': Finding;
  'scan-done': ScanReport;
  'clean-progress': Progress;
  'clean-done': CleanDone;
  'reserve-changed': number;
}

const real = {
  getSettings: () => invoke<Settings>('get_settings'),
  updateSettings: (patch: SettingsPatch) => invoke<Settings>('update_settings', { patch }),
  getOverview: () => invoke<Overview>('get_overview'),
  getRules: (lang: string) => invoke<RuleView[]>('get_rules', { lang }),
  startScan: () => invoke<void>('start_scan'),
  startClean: (rules: string[]) => invoke<void>('start_clean', { rules }),
  cancelJob: () => invoke<void>('cancel_job'),
  getRuleFiles: (rule: string, limit: number) => invoke<RuleFiles>('get_rule_files', { rule, limit }),
  getReserve: () => invoke<Batch[]>('get_reserve'),
  getReserveFiles: (batch: string, limit: number) => invoke<ReservedFile[]>('get_reserve_files', { batch, limit }),
  restoreBatch: (batch: string) => invoke<RestoreReport>('restore_batch', { batch }),
  releaseBatch: (batch: string | null) => invoke<number>('release_batch', { batch }),
  reveal: (path: string) => invoke<void>('reveal', { path }),
};

export type Api = typeof real;

export const api: Api = native ? real : demo;

/** Listens to an event from the Rust side; resolves to the function that stops listening. */
export function on<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): Promise<UnlistenFn> {
  if (!native) return Promise.resolve(demoEvents.on(event, handler));
  return listen<Events[K]>(event, (e) => handler(e.payload));
}

/** A command's error code (`busy`, `no_scan`…), for the texts. */
export function errorCode(e: unknown): string {
  return typeof e === 'string' ? e : 'generic';
}
