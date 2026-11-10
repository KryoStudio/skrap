<!-- The main window: side bar, title bar, pages, and the dialogs above them. At launch, in this
     order: the welcome (first launch), "What's new" (first launch after an update), then the
     update offer when a new version is out. -->
<script lang="ts">
  import { getVersion } from '@tauri-apps/api/app';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { native } from '$lib/api';
  import { demoScreen } from '$lib/demo';
  import { releaseNotes, type Release } from '$lib/changelog';
  import { flow } from '$lib/flow.svelte';
  import { overview } from '$lib/overview.svelte';
  import { reserve } from '$lib/reserve.svelte';
  import { motion, settings } from '$lib/settings.svelte';
  import { slideIn } from '$lib/transitions';
  import { updates } from '$lib/updates.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import TitleBar from '$lib/components/TitleBar.svelte';
  import Toasts from '$lib/components/Toasts.svelte';
  import UpdateDialog from '$lib/components/UpdateDialog.svelte';
  import Welcome from '$lib/components/Welcome.svelte';
  import WhatsNew from '$lib/components/WhatsNew.svelte';

  let { children } = $props();

  let version = $state('');
  let welcome = $state(false);
  let whatsNew = $state<Release | null>(null);

  onMount(() => {
    flow.listen();
    reserve.listen();
    void overview.refresh();
    void reserve.load();
    void greet();

    if (!native) {
      // Browser preview: open the screen asked for in the address.
      if (['scanning', 'results', 'cleaning', 'done'].includes(demoScreen)) void flow.scan();
      if (demoScreen === 'cleaning' || demoScreen === 'done') {
        const wait = setInterval(() => {
          if (flow.phase !== 'results') return;
          clearInterval(wait);
          void flow.clean();
        }, 100);
      }
      if (demoScreen === 'update') updates.preview();
    }
    // The update check waits for the page to settle; the offer waits for the other dialogs.
    const check = setTimeout(() => {
      if (settings.value.checkUpdates) void updates.check(true);
    }, 1500);
    return () => clearTimeout(check);
  });

  async function greet() {
    version = native ? await getVersion() : '0.1.0';
    if (!settings.value.onboarded) {
      welcome = true;
    } else if (settings.value.lastSeenVersion !== version) {
      const release = releaseNotes(version);
      if (release) whatsNew = release;
      else void settings.update({ lastSeenVersion: version }).catch(() => {});
    }
  }

  function closeWelcome() {
    welcome = false;
    void settings.update({ onboarded: true, lastSeenVersion: version }).catch(() => {});
  }

  function closeWhatsNew() {
    whatsNew = null;
    void settings.update({ lastSeenVersion: version }).catch(() => {});
  }

  function onkeydown(e: KeyboardEvent) {
    const key = e.key.toLowerCase();
    if (e.ctrlKey && key === ',') {
      e.preventDefault();
      void goto('/settings');
    } else if (!import.meta.env.DEV && (e.key === 'F5' || e.key === 'F7' || (e.ctrlKey && ['r', 'p', 'u', 'g', 'j', 'f'].includes(key)))) {
      // A desktop app, not a web page: no reload, print, view-source or caret browsing.
      e.preventDefault();
    }
  }

  function oncontextmenu(e: MouseEvent) {
    const editable = (e.target as HTMLElement).closest('input, textarea');
    if (!editable && !import.meta.env.DEV) e.preventDefault();
  }
</script>

<svelte:window {onkeydown} {oncontextmenu} />

<div class="app">
  <Sidebar />
  <div class="column">
    <TitleBar />
    <main>
      {#key page.url.pathname}
        <div class="route" in:slideIn={{ duration: motion.ms(300), x: -14, skew: 0 }}>
          {@render children()}
        </div>
      {/key}
    </main>
  </div>
</div>

{#if welcome}
  <Welcome onclose={closeWelcome} />
{:else if whatsNew}
  <WhatsNew release={whatsNew} onclose={closeWhatsNew} />
{:else if updates.open}
  <UpdateDialog />
{/if}
<ConfirmDialog />
<Toasts />

<style>
  .app {
    display: flex;
    height: 100vh;
  }
  .column {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  main {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    scrollbar-gutter: stable;
  }
  .route {
    display: flex;
    flex-direction: column;
    min-height: 100%;
  }
  .route > :global(*) {
    flex: 1;
  }
</style>
