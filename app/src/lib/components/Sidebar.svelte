<!-- The side bar: the mark, the pages, the version. The active page's marker is an acid slash
     that slides from one entry to the next. -->
<script lang="ts">
  import { page } from '$app/state';
  import { t, formatSize, type Key } from '$lib/i18n.svelte';
  import { overview } from '$lib/overview.svelte';
  import { flow } from '$lib/flow.svelte';
  import Icon, { type IconName } from './Icon.svelte';
  import Logo from './Logo.svelte';

  const items: { href: string; label: Key; icon: IconName }[] = [
    { href: '/', label: 'nav.clean', icon: 'clean' },
    { href: '/reserve', label: 'nav.reserve', icon: 'reserve' },
  ];

  const path = $derived(page.url.pathname);
  const active = $derived(path === '/settings' ? -1 : Math.max(0, items.findIndex((i) => i.href === path)));
  const reserveBytes = $derived(overview.value?.reserve.bytes ?? 0);
  const busy = $derived(flow.phase === 'scanning' || flow.phase === 'cleaning');
</script>

<aside>
  <a class="brand" href="/" data-tauri-drag-region aria-label="Skrap">
    <Logo size={30} />
    <span class="titre">SKRAP</span>
  </a>

  <nav>
    {#if active >= 0}
      <span class="marker" style:transform="translateY({active * 44}px) skewX(var(--penche))"></span>
    {/if}
    {#each items as item, i (item.href)}
      <a href={item.href} class:on={active === i} aria-current={active === i ? 'page' : undefined}>
        <Icon name={item.icon} />
        <span class="label">{t(item.label)}</span>
        {#if item.href === '/' && busy}
          <span class="live"></span>
        {:else if item.href === '/reserve' && reserveBytes > 0}
          <span class="badge chiffres">{formatSize(reserveBytes)}</span>
        {/if}
      </a>
    {/each}
  </nav>

  <div class="foot">
    <a href="/settings" class:on={path === '/settings'} aria-current={path === '/settings' ? 'page' : undefined}>
      <Icon name="settings" />
      <span class="label">{t('nav.settings')}</span>
    </a>
    {#if overview.value}
      <p class="version chiffres">v{overview.value.version}</p>
    {/if}
  </div>
</aside>

<style>
  aside {
    display: flex;
    flex-direction: column;
    width: var(--sidebar);
    flex: none;
    padding: 0 12px 14px;
    background: var(--noir-2);
    box-shadow: inset -1px 0 0 var(--trait);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 72px;
    padding: 0 10px;
    text-decoration: none;
  }
  .brand .titre {
    font-size: 22px;
    letter-spacing: 0.01em;
    color: var(--os);
  }

  nav {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 10px;
  }
  .marker {
    position: absolute;
    left: 2px;
    top: 10px;
    width: 4px;
    height: 20px;
    border-radius: 1px;
    background: var(--acide);
    transition: transform var(--transition) var(--entree);
  }

  nav a,
  .foot a {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 40px;
    padding: 0 12px 0 16px;
    border-radius: var(--rayon-sm);
    color: var(--gris);
    text-decoration: none;
    font-weight: 600;
    transition:
      background var(--micro),
      color var(--micro);
  }
  nav a:hover,
  .foot a:hover {
    color: var(--os);
    background: rgb(255 255 255 / 0.04);
  }
  nav a.on,
  .foot a.on {
    color: var(--os);
    background: rgb(255 255 255 / 0.05);
  }
  nav a.on :global(svg),
  .foot a.on :global(svg) {
    color: var(--acide);
  }
  .label {
    flex: 1;
  }

  .badge {
    padding: 2px 7px;
    border-radius: 999px;
    font-size: 11px;
    color: var(--os-2);
    background: rgb(255 255 255 / 0.07);
  }
  .live {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--acide);
    animation: pulse 1.2s ease-in-out infinite;
  }
  @keyframes pulse {
    50% {
      opacity: 0.3;
    }
  }

  .foot {
    margin-top: auto;
  }
  .version {
    padding: 10px 16px 0;
    font-size: 11px;
    color: var(--gris-2);
  }
</style>
