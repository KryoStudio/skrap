<!-- The window has no native frame: this bar drags it and holds its buttons. While an update
     waits (put off with "Later"), a small reminder sits here. -->
<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { fly } from 'svelte/transition';
  import { native } from '$lib/api';
  import { t } from '$lib/i18n.svelte';
  import { motion } from '$lib/settings.svelte';
  import { updates } from '$lib/updates.svelte';
  import Icon from './Icon.svelte';

  const win = native ? getCurrentWindow() : null;
  let maximized = $state(false);

  $effect(() => {
    if (!win) return;
    let stop: (() => void) | undefined;
    let alive = true;
    const sync = async () => (maximized = await win.isMaximized());
    void sync();
    win.onResized(sync).then((unlisten) => (alive ? (stop = unlisten) : unlisten()));
    return () => {
      alive = false;
      stop?.();
    };
  });
</script>

<header data-tauri-drag-region>
  <div class="left" data-tauri-drag-region>
    {#if updates.version && !updates.open}
      <button class="pill" onclick={() => (updates.open = true)} in:fly={{ x: -10, duration: motion.ms(240) }}>
        <Icon name="download" size={13} stroke={2.2} />
        {t('update.pill', { version: updates.version })}
      </button>
    {/if}
  </div>

  {#if win}
    <div class="controls">
      <button onclick={() => win.minimize()} aria-label={t('window.minimize')}>
        <svg viewBox="0 0 10 10" width="10" height="10"><path d="M0 5.5h10" /></svg>
      </button>
      <button onclick={() => win.toggleMaximize()} aria-label={t(maximized ? 'window.restore' : 'window.maximize')}>
        {#if maximized}
          <svg viewBox="0 0 10 10" width="10" height="10">
            <rect x="0.5" y="2.5" width="7" height="7" rx="1" />
            <path d="M2.5 2.5v-1a1 1 0 0 1 1-1h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-1" />
          </svg>
        {:else}
          <svg viewBox="0 0 10 10" width="10" height="10"><rect x="0.5" y="0.5" width="9" height="9" rx="1.5" /></svg>
        {/if}
      </button>
      <button class="close" onclick={() => win.close()} aria-label={t('window.close')}>
        <svg viewBox="0 0 10 10" width="10" height="10"><path d="M0.5 0.5l9 9M9.5 0.5l-9 9" /></svg>
      </button>
    </div>
  {/if}
</header>

<style>
  header {
    display: flex;
    align-items: stretch;
    justify-content: space-between;
    height: var(--titlebar);
    flex: none;
  }
  .left {
    display: flex;
    align-items: center;
    flex: 1;
    padding-left: 20px;
  }

  .pill {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 26px;
    padding: 0 12px;
    border-radius: 999px;
    color: var(--acide);
    background: var(--acide-doux);
    box-shadow: inset 0 0 0 1px var(--acide-ligne);
    font-size: 12px;
    font-weight: 600;
    transition: background var(--micro);
  }
  .pill:hover {
    background: rgb(214 255 61 / 0.18);
  }

  .controls {
    display: flex;
  }
  .controls button {
    display: grid;
    place-items: center;
    width: 46px;
    color: var(--gris);
    transition:
      background var(--micro),
      color var(--micro);
  }
  .controls svg {
    fill: none;
    stroke: currentColor;
    stroke-width: 1;
  }
  .controls button:hover {
    background: rgb(255 255 255 / 0.06);
    color: var(--os);
  }
  .controls .close:hover {
    background: #c42b1c;
    color: #fff;
  }
</style>
