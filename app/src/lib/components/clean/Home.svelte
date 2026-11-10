<!-- Home: one big Scan button, the disks, what was cleaned so far, what waits in the Reserve. -->
<script lang="ts">
  import { flow } from '$lib/flow.svelte';
  import { formatAgo, formatDate, formatSize, t } from '$lib/i18n.svelte';
  import { overview } from '$lib/overview.svelte';
  import { motion } from '$lib/settings.svelte';
  import { slideIn } from '$lib/transitions';
  import BigNumber from '../BigNumber.svelte';
  import Icon from '../Icon.svelte';

  const o = $derived(overview.value);
  const lastScan = $derived(o?.stats.lastScan ? t('home.lastScan', { when: formatAgo(o.stats.lastScan) }) : t('home.neverScanned'));
  const titleLines = $derived(t('home.title').split('\n'));
</script>

<section class="home">
  <div class="streaks" aria-hidden="true">
    <span style="top: 14%; left: 52%; width: 260px"></span>
    <span style="top: 31%; left: 74%; width: 180px"></span>
    <span style="top: 58%; left: 62%; width: 320px"></span>
    <span style="top: 77%; left: 80%; width: 140px"></span>
  </div>

  <div class="hero">
    <p class="eyebrow" in:slideIn={{ duration: motion.ms(360) }}>{lastScan}</p>
    <h1 class="titre">
      {#each titleLines as line, i (i)}
        <span class="line" in:slideIn={{ delay: motion.ms(60 + i * 70), duration: motion.ms(520), x: -40 }}>{line}</span>
      {/each}
    </h1>
    <p class="lead" in:slideIn={{ delay: motion.ms(200), duration: motion.ms(420) }}>{t('home.lead')}</p>

    <div class="go" in:slideIn={{ delay: motion.ms(260), duration: motion.ms(520), x: -60 }}>
      <button class="scan" onclick={() => flow.scan()}>
        <span class="label titre">{t('home.scan')}</span>
        <span class="arrow"><Icon name="arrow-right" size={26} stroke={2.6} /></span>
      </button>
    </div>
  </div>

  {#if o}
    <div class="tiles">
      <div class="tile" in:slideIn={{ delay: motion.ms(340), duration: motion.ms(420) }}>
        <p class="eyebrow">{t('home.cleaned')}</p>
        <BigNumber bytes={o.stats.cleanedBytes} size={40} duration={900} />
        <p class="sub">{t('home.cleans', { count: o.stats.cleans })}</p>
      </div>
      <a class="tile link" href="/reserve" in:slideIn={{ delay: motion.ms(380), duration: motion.ms(420) }}>
        <p class="eyebrow">{t('home.reserve')}</p>
        <BigNumber bytes={o.reserve.bytes} size={40} duration={900} />
        <p class="sub">
          {o.reserve.nextRelease ? t('home.reserveUntil', { date: formatDate(o.reserve.nextRelease) }) : t('home.reserveEmpty')}
        </p>
      </a>
      {#each o.disks as disk, i (disk.root)}
        {@const used = disk.total > 0 ? 1 - disk.free / disk.total : 0}
        <div class="tile disk" in:slideIn={{ delay: motion.ms(420 + i * 40), duration: motion.ms(420) }}>
          <p class="eyebrow">
            {disk.root.replace('\\', '')}
            {disk.system ? `· ${t('home.systemDisk')}` : disk.label ? `· ${disk.label}` : ''}
          </p>
          <p class="free"><BigNumber bytes={disk.free} size={40} duration={900} /> <span>{t('home.free')}</span></p>
          <div class="bar" class:low={disk.free / Math.max(1, disk.total) < 0.1}>
            <span style:transform="scaleX({used})"></span>
          </div>
          <p class="sub">{t('home.freeOf', { free: formatSize(disk.free), total: formatSize(disk.total) })}</p>
        </div>
      {/each}
    </div>
  {/if}
</section>

<style>
  .home {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 56px;
    padding: 40px 56px 48px;
  }

  .streaks span {
    position: absolute;
    height: 3px;
    background: var(--trait);
    transform: skewX(var(--penche));
  }

  .hero {
    position: relative;
  }
  h1 {
    display: flex;
    flex-direction: column;
    margin-top: 14px;
    font-size: clamp(52px, 6.4vw, 84px);
  }
  .line {
    display: block;
  }
  .lead {
    max-width: 460px;
    margin-top: 18px;
    font-size: 15px;
    color: var(--gris);
    line-height: 1.55;
  }

  .go {
    margin-top: 36px;
  }
  /* The Scan button: a slanted acid block, a glint on hover, a quiet pulse while it waits. */
  .scan {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 18px;
    height: 76px;
    padding: 0 38px 0 44px;
    border-radius: 6px;
    color: var(--sur-acide);
    background: var(--acide);
    transform: skewX(var(--penche));
    transition: transform var(--micro) var(--entree);
    animation: ready 2.6s ease-out infinite 1.2s;
  }
  .scan > * {
    transform: skewX(12deg);
  }
  .scan::after {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: linear-gradient(100deg, transparent 30%, rgb(255 255 255 / 0.65) 50%, transparent 70%);
    transform: translateX(-120%);
    pointer-events: none;
  }
  .scan:hover {
    transform: skewX(var(--penche)) translateX(4px);
  }
  .scan:hover::after {
    transform: translateX(120%);
    transition: transform 750ms var(--entree);
  }
  .scan:active {
    transform: skewX(var(--penche)) scale(0.97);
  }
  .label {
    font-size: 34px;
    letter-spacing: 0.01em;
    text-transform: uppercase;
  }
  .arrow {
    display: grid;
    transition: transform var(--micro) var(--entree);
  }
  .scan:hover .arrow {
    transform: skewX(12deg) translateX(5px);
  }
  @keyframes ready {
    0% {
      box-shadow: 0 0 0 0 rgb(214 255 61 / 0.22);
    }
    70%,
    100% {
      box-shadow: 0 0 0 16px rgb(214 255 61 / 0);
    }
  }

  .tiles {
    position: relative;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px;
    margin-top: auto;
  }
  .tile {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 18px 20px;
    border-radius: var(--rayon);
    background: var(--carbone);
    text-decoration: none;
  }
  .tile.link {
    transition: background var(--micro);
  }
  .tile.link:hover {
    background: var(--carbone-2);
  }
  .sub {
    font-size: 12.5px;
    color: var(--gris);
  }
  .free {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .free > span {
    color: var(--gris);
    font-size: 13px;
  }
  .bar {
    height: 6px;
    overflow: hidden;
    border-radius: 1px;
    background: var(--carbone-3);
    transform: skewX(var(--penche));
  }
  .bar span {
    display: block;
    height: 100%;
    background: var(--os-2);
    transform-origin: left;
    transition: transform 900ms var(--entree);
  }
  .bar.low span {
    background: var(--ambre);
  }
</style>
