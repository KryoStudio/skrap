// What the home screen and the sidebar show: disks, totals, the Reserve.
import { api, type Overview } from './api';

class OverviewStore {
  value = $state<Overview | null>(null);

  async refresh() {
    try {
      this.value = await api.getOverview();
    } catch (e) {
      console.warn('overview failed', e);
    }
  }

  get systemDisk() {
    return this.value?.disks.find((d) => d.system) ?? this.value?.disks[0] ?? null;
  }
}

export const overview = new OverviewStore();
