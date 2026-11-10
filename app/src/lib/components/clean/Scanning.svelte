<!-- The scan, to watch: the sweep crosses the screen, the counter runs up like a speedometer,
     the categories fill as their rules come in, the folder being read goes by underneath. -->
<script lang="ts">
  import { CATEGORIES, flow } from '$lib/flow.svelte';
  import { formatNumber, formatSize, t } from '$lib/i18n.svelte';
  import { motion } from '$lib/settings.svelte';
  import { slideIn } from '$lib/transitions';
  import BigNumber from '../BigNumber.svelte';
  import Icon, { type IconName } from '../Icon.svelte';
  import Sweep from '../Sweep.svelte';

  const p = $derived(flow.scanProgress);
  const totals = $derived(CATEGORIES.map((c) => ({ category: c, bytes: flow.categoryBytes(c) })));
  const max = $derived(Math.max(1, ...totals.map((c) => c.bytes)));
  const icons: Record<string, IconName> = { system: 'system', gaming: 'gaming', browsers: 'browsers', apps: 'apps' };

  /** A long path keeps its start and its end. */
  function shorten(path: string, max = 72): string {
    if (path.length <= max) return path;
    return `${path.slice(0, 22)}…${path.slice(-(max - 23))}`;
  }
</script>

<section class="scanning">
  <Sweep />

  <div class="counter">
    <BigNumber bytes={p.bytes} size={128} duration={260} />
    <p class="found titre">{t('scan.found')}</p>
    <p class="path">{p.current ? shorten(p.current) : t('scan.starting')}</p>
  </div>

  <div class="side">
    <ul>
      {#each totals as { category, bytes }, i (category)}
        <li class:empty={bytes === 0} in:slideIn={{ delay: motion.ms(i * 40), duration: motion.ms(360) }}>
          <Icon name={icons[category]} size={16} />
          <span class="name">{t(`cat.${category}`)}</span>
          <span class="size chiffres">{bytes > 0 ? formatSize(bytes) : '—'}</span>
          <span class="bar"><span style:transform="scaleX({bytes / max})"></span></span>
        </li>
      {/each}
    </ul>
    <div class="foot">
      <p class="chiffres">{t('scan.looking', { count: p.files })} · {formatNumber(Math.round((p.stepsDone / Math.max(1, p.stepsTotal)) * 100))} %</p>
      <button class="btn btn-ghost" onclick={() => flow.cancel()}>{t('scan.stop')}</button>
    </div>
  </div>
</section>

<style>
  .scanning {
    position: relative;
    display: grid;
    grid-template-columns: 1fr 320px;
    align-items: end;
    gap: 40px;
    padding: 40px 56px 48px;
    overflow: hidden;
  }

  .counter {
    position: relative;
    min-width: 0;
  }
  .found {
    margin-top: 6px;
    font-size: 30px;
    color: var(--acide);
  }
  .path {
    margin-top: 18px;
    overflow: hidden;
    font-size: 12px;
    color: var(--gris-2);
    white-space: nowrap;
    text-overflow: ellipsis;
    font-variant-numeric: tabular-nums;
  }

  .side {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 20px;
    border-radius: var(--rayon-lg);
    background: rgb(23 23 26 / 0.86);
    backdrop-filter: blur(8px);
  }
  ul {
    display: flex;
    flex-direction: column;
    gap: 14px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li {
    display: grid;
    grid-template-columns: 16px 1fr auto;
    align-items: center;
    gap: 6px 10px;
    color: var(--os);
    transition: color var(--transition);
  }
  li.empty {
    color: var(--gris-2);
  }
  .name {
    font-weight: 600;
    font-size: 13.5px;
  }
  .size {
    font-size: 13px;
  }
  .bar {
    grid-column: 1 / -1;
    height: 4px;
    overflow: hidden;
    background: var(--carbone-3);
    transform: skewX(var(--penche));
  }
  .bar span {
    display: block;
    height: 100%;
    background: var(--acide);
    transform-origin: left;
    transition: transform 420ms var(--entree);
  }
  .foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    font-size: 12px;
    color: var(--gris);
  }
</style>
