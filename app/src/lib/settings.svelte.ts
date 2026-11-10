import { prefersReducedMotion } from 'svelte/motion';
import { api, type Settings, type SettingsPatch } from './api';

class SettingsStore {
  value = $state<Settings>({
    language: null,
    checkUpdates: true,
    reserveDays: 7,
    motion: 'auto',
    onboarded: true,
    lastSeenVersion: null,
  });
  loaded = $state(false);

  async load() {
    this.value = await api.getSettings();
    this.loaded = true;
  }

  /** Saves a change; the page shows it at once and goes back if saving fails. */
  async update(patch: SettingsPatch) {
    const before = this.value;
    this.value = { ...before, ...patch };
    try {
      this.value = await api.updateSettings(patch);
    } catch (e) {
      this.value = before;
      throw e;
    }
  }
}

export const settings = new SettingsStore();

/** Motion as the user wants it: Windows' animation setting, unless "Reduced" is chosen. */
export const motion = {
  get reduced() {
    return settings.value.motion === 'reduced' || prefersReducedMotion.current;
  },
  /** A duration, or 0 when animations are reduced. */
  ms(duration: number) {
    return this.reduced ? 0 : duration;
  },
};
