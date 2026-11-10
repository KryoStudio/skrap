<!-- Fonts, theme, language and motion for the whole window. The window is shown only once the
     page has drawn, so it never flashes empty. -->
<script lang="ts">
  import '../app.css';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount, tick } from 'svelte';
  import { native } from '$lib/api';
  import { flow } from '$lib/flow.svelte';
  import { i18n, locale, resolveLang } from '$lib/i18n.svelte';
  import { settings } from '$lib/settings.svelte';

  let { children } = $props();

  $effect(() => {
    i18n.lang = resolveLang(settings.value.language);
    document.documentElement.lang = locale();
    void flow.loadRules(i18n.lang);
  });
  $effect(() => {
    document.documentElement.dataset.motion = settings.value.motion;
  });

  onMount(async () => {
    try {
      await settings.load();
    } catch (e) {
      console.error('settings failed', e);
      settings.loaded = true;
    }
    if (!native) return;
    await tick();
    // Two frames: the first paints the page, the second makes sure it is on screen.
    requestAnimationFrame(() =>
      requestAnimationFrame(() => {
        const win = getCurrentWindow();
        win.show().then(() => win.setFocus()).catch(() => {});
      }),
    );
  });
</script>

{#if settings.loaded}
  {@render children()}
{/if}
