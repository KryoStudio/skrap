// The main path: home → scan → results → clean → done. One instance for the whole app, so a
// scan keeps going (and its results stay) while another page is open.
import { SvelteSet } from 'svelte/reactivity';
import { api, errorCode, on, type CleanDone, type Category, type Finding, type Progress, type RuleView, type ScanReport } from './api';
import { hasKey, t } from './i18n.svelte';
import { overview } from './overview.svelte';
import { toast } from './ui.svelte';

export type Phase = 'idle' | 'scanning' | 'results' | 'cleaning' | 'done';

export const CATEGORIES: Category[] = ['system', 'gaming', 'browsers', 'apps'];

/** The scan animation runs at least this long, so its moment reads even on a fast disk. */
const MIN_SCAN_MS = 1800;
/** Time for the last rows to fly off before the logo crosses the screen. */
const CLEAN_OUTRO_MS = 650;

const EMPTY: Progress = { files: 0, bytes: 0, stepsDone: 0, stepsTotal: 0, current: '' };

class Flow {
  phase = $state<Phase>('idle');
  rules = $state<RuleView[]>([]);
  /** Findings as the scan sends them, then the full report's. */
  findings = $state<Finding[]>([]);
  report = $state<ScanReport | null>(null);
  scanProgress = $state<Progress>(EMPTY);
  ticked = new SvelteSet<string>();
  /** The rules being cleaned, in the order they are shown and cleaned. */
  cleaning = $state<string[]>([]);
  cleanProgress = $state<Progress>(EMPTY);
  done = $state<CleanDone | null>(null);
  /** The logo crossing the screen between the clean and its recap. */
  crossing = $state(false);

  private scanStarted = 0;
  private listening = false;

  rule(id: string): RuleView | undefined {
    return this.rules.find((r) => r.id === id);
  }

  finding(id: string): Finding | undefined {
    return this.findings.find((f) => f.rule === id);
  }

  /** Found rules of a category, in the rules' order. */
  inCategory(category: Category): { rule: RuleView; finding: Finding }[] {
    return this.rules
      .filter((rule) => rule.category === category)
      .flatMap((rule) => {
        const finding = this.finding(rule.id);
        return finding ? [{ rule, finding }] : [];
      });
  }

  categoryBytes(category: Category): number {
    return this.inCategory(category).reduce((sum, { finding }) => sum + finding.bytes, 0);
  }

  /** What can be ticked: found, and not held by a running program. */
  cleanable(id: string): boolean {
    const finding = this.finding(id);
    return !!finding && !finding.skip;
  }

  get foundBytes(): number {
    return this.findings.reduce((sum, f) => sum + f.bytes, 0);
  }

  get selectedBytes(): number {
    return this.findings.filter((f) => this.ticked.has(f.rule)).reduce((sum, f) => sum + f.bytes, 0);
  }

  /** Of the selection, what goes to the Reserve rather than being deleted. */
  get selectedToReserve(): number {
    return this.findings
      .filter((f) => this.ticked.has(f.rule))
      .filter((f) => {
        const rule = this.rule(f.rule);
        return rule && rule.risk !== 'safe' && !rule.special;
      })
      .reduce((sum, f) => sum + f.bytes, 0);
  }

  /** Whether every ticked rule is Safe. */
  get onlySafeTicked(): boolean {
    return [...this.ticked].every((id) => this.rule(id)?.risk === 'safe');
  }

  async loadRules(lang: string) {
    try {
      this.rules = await api.getRules(lang);
    } catch (e) {
      console.error('rules failed', e);
    }
  }

  /** Listens to the Rust side once, for the whole app. */
  listen() {
    if (this.listening) return;
    this.listening = true;
    on('scan-progress', (p) => (this.scanProgress = p));
    on('scan-found', (finding) => {
      if (this.phase === 'scanning') this.findings = [...this.findings, finding];
    });
    on('scan-done', (report) => void this.scanDone(report));
    on('clean-progress', (p) => (this.cleanProgress = p));
    on('clean-done', (done) => void this.cleanDone(done));
  }

  async scan() {
    if (this.phase === 'scanning' || this.phase === 'cleaning') return;
    this.findings = [];
    this.report = null;
    this.done = null;
    this.scanProgress = EMPTY;
    this.ticked.clear();
    this.phase = 'scanning';
    this.scanStarted = performance.now();
    try {
      await api.startScan();
    } catch (e) {
      this.fail(e);
      this.phase = 'idle';
    }
  }

  private async scanDone(report: ScanReport) {
    if (this.phase !== 'scanning') return;
    const left = MIN_SCAN_MS - (performance.now() - this.scanStarted);
    if (left > 0 && !report.cancelled) await new Promise((r) => setTimeout(r, left));
    if (this.phase !== 'scanning') return;
    if (report.cancelled) {
      this.phase = 'idle';
      return;
    }
    this.report = report;
    this.findings = report.findings;
    this.ticked.clear();
    for (const finding of report.findings) {
      if (this.rule(finding.rule)?.ticked && !finding.skip) this.ticked.add(finding.rule);
    }
    this.phase = 'results';
    void overview.refresh();
  }

  toggle(id: string) {
    if (!this.cleanable(id)) return;
    if (this.ticked.has(id)) this.ticked.delete(id);
    else this.ticked.add(id);
  }

  async clean() {
    if (this.phase !== 'results' || this.ticked.size === 0) return;
    // In the order the results show them.
    const order = CATEGORIES.flatMap((c) => this.inCategory(c).map(({ rule }) => rule.id)).filter((id) => this.ticked.has(id));
    this.cleaning = order;
    this.cleanProgress = { ...EMPTY, stepsTotal: order.length };
    this.done = null;
    this.phase = 'cleaning';
    try {
      await api.startClean(order);
    } catch (e) {
      this.fail(e);
      this.phase = 'results';
    }
  }

  private async cleanDone(done: CleanDone) {
    if (this.phase !== 'cleaning') return;
    this.cleanProgress = { ...this.cleanProgress, stepsDone: this.cleaning.length };
    await new Promise((r) => setTimeout(r, CLEAN_OUTRO_MS));
    this.done = done;
    this.crossing = true;
    this.phase = 'done';
    void overview.refresh();
  }

  cancel() {
    void api.cancelJob();
  }

  reset() {
    this.phase = 'idle';
    this.findings = [];
    this.report = null;
    this.done = null;
    this.crossing = false;
    this.ticked.clear();
  }

  private fail(e: unknown) {
    const key = `error.${errorCode(e)}`;
    toast({ kind: 'error', title: t(hasKey(key) ? key : 'error.generic') });
  }
}

export const flow = new Flow();
