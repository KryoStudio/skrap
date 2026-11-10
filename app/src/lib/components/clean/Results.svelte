<!-- What the scan found: the total, then each category's rules with their size and reason. The
     bar at the bottom says what is selected and cleans it. -->
<script lang="ts">
  import { CATEGORIES, flow } from '$lib/flow.svelte';
  import { formatDuration, formatNumber, formatSize, t } from '$lib/i18n.svelte';
  import { motion } from '$lib/settings.svelte';
  import { slideIn } from '$lib/transitions';
  import BigNumber from '../BigNumber.svelte';
  import Icon from '../Icon.svelte';
  import CategoryCard from './CategoryCard.svelte';

  const shown = $derived(CATEGORIES.filter((c) => flow.inCategory(c).length > 0));
  const files = $derived(flow.findings.reduce((sum, f) => sum + f.files, 0));
  const nothing = $derived(flow.foundBytes === 0);
</script>

<section class="results">
  <header>
    {#if nothing}
      <h1 class="titre spotless" in:slideIn={{ duration: motion.ms(520), x: -40 }}>{t('results.clean.title')}</h1>
      <p class="sub">{t('results.clean.body')}</p>
    {:else}
      <div class="total" in:slideIn={{ duration: motion.ms(520), x: -40 }}>
        <BigNumber bytes={flow.foundBytes} from={flow.scanProgress.bytes} size={104} />
        <span class="titre junk">{t('results.junk')}</span>
      </div>
      <p class="sub">{flow.onlySafeTicked ? t('results.allSafe') : t('results.someReview')}</p>
    {/if}
    {#if flow.report}
      <p class="meta chiffres">
        {t('results.meta', { files: formatNumber(files), time: formatDuration(flow.report.durationMs) })}
      </p>
    {/if}
  </header>

  <div class="cards">
    {#each shown as category, i (category)}
      <div in:slideIn={{ delay: motion.ms(120 + i * 40), duration: motion.ms(380) }}>
        <CategoryCard {category} />
      </div>
    {/each}
  </div>

  <footer>
    <div class="selected">
      <p class="eyebrow">{t('results.selected')}</p>
      <div class="sum">
        <BigNumber bytes={flow.selectedBytes} size={34} duration={300} />
        {#if flow.selectedToReserve > 0}
          <span class="reserve">{t('results.toReserve', { size: formatSize(flow.selectedToReserve) })}</span>
        {/if}
      </div>
    </div>
    <div class="actions">
      <button class="btn btn-ghost" onclick={() => flow.scan()}>
        <Icon name="restore" size={15} />{t('results.rescan')}
      </button>
      <button class="btn btn-primary clean" disabled={flow.ticked.size === 0} onclick={() => flow.clean()}>
        {t('results.clean')}
        <Icon name="arrow-right" size={18} stroke={2.6} />
      </button>
    </div>
  </footer>
</section>

<style>
  .results {
    display: flex;
    flex-direction: column;
  }
  header {
    padding: 36px 56px 28px;
  }
  .total {
    display: flex;
    align-items: baseline;
    flex-wrap: wrap;
    gap: 6px 18px;
  }
  .junk {
    font-size: 40px;
    color: var(--acide);
  }
  .spotless {
    font-size: 84px;
  }
  .sub {
    margin-top: 14px;
    font-size: 15px;
    color: var(--os-2);
  }
  .meta {
    margin-top: 6px;
    font-size: 12.5px;
    color: var(--gris-2);
  }

  .cards {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 0 56px 28px;
    max-width: 1040px;
  }

  footer {
    position: sticky;
    bottom: 0;
    z-index: 5;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    margin-top: auto;
    padding: 16px 56px;
    background: rgb(11 11 12 / 0.9);
    backdrop-filter: blur(12px);
    box-shadow: 0 -1px 0 var(--trait);
  }
  .sum {
    display: flex;
    align-items: baseline;
    gap: 14px;
    margin-top: 4px;
  }
  .reserve {
    font-size: 12.5px;
    color: var(--ambre);
  }
  .actions {
    display: flex;
    gap: 10px;
  }
  .clean {
    height: 48px;
    padding: 0 26px;
    font-size: 20px;
    transform: skewX(var(--penche));
  }
  .clean:active:not(:disabled) {
    transform: skewX(var(--penche)) scale(0.97);
  }
  .clean > :global(*) {
    transform: skewX(12deg);
  }
</style>
