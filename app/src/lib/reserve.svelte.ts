// The Reserve page's state: batches, restoring, freeing.
import { api, on, type Batch } from './api';
import { formatSize, t } from './i18n.svelte';
import { overview } from './overview.svelte';
import { confirm, toast } from './ui.svelte';

class ReserveStore {
  batches = $state<Batch[]>([]);
  loaded = $state(false);
  /** Batches on their way back (restoring), for the animation. */
  leaving = $state<string[]>([]);
  private listening = false;

  readonly bytes = $derived(this.batches.reduce((sum, b) => sum + b.bytes, 0));
  readonly files = $derived(this.batches.reduce((sum, b) => sum + b.files, 0));

  async load() {
    try {
      this.batches = await api.getReserve();
    } finally {
      this.loaded = true;
    }
  }

  listen() {
    if (this.listening) return;
    this.listening = true;
    on('reserve-changed', () => {
      void this.load();
      void overview.refresh();
    });
    on('clean-done', () => void this.load());
  }

  async restore(batch: Batch) {
    this.leaving = [...this.leaving, batch.id];
    try {
      const report = await api.restoreBatch(batch.id);
      if (report.files > 0) toast({ kind: 'success', title: t('reserve.restored', { count: report.files }) });
      if (report.failures.length > 0) toast({ kind: 'error', title: t('reserve.restoreFailed', { count: report.failures.length }) });
    } catch {
      toast({ kind: 'error', title: t('error.generic') });
    } finally {
      this.leaving = this.leaving.filter((id) => id !== batch.id);
      await this.load();
      void overview.refresh();
    }
  }

  async release(batch: Batch | null) {
    const size = formatSize(batch ? batch.bytes : this.bytes);
    const ok = await confirm({
      title: t(batch ? 'reserve.confirmTitle' : 'reserve.confirmAllTitle'),
      body: t('reserve.confirmBody', { size }),
      confirm: t(batch ? 'reserve.release' : 'reserve.releaseAll'),
    });
    if (!ok) return;
    try {
      const freed = await api.releaseBatch(batch?.id ?? null);
      toast({ kind: 'success', title: t('reserve.released', { size: formatSize(freed) }) });
    } catch {
      toast({ kind: 'error', title: t('error.generic') });
    }
    await this.load();
    void overview.refresh();
  }
}

export const reserve = new ReserveStore();
