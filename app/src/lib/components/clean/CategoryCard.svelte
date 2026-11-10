<!-- One category of the results (System, Games and launchers...), its rules inside. -->
<script lang="ts">
  import type { Category } from '$lib/api';
  import { flow } from '$lib/flow.svelte';
  import { formatSize, t } from '$lib/i18n.svelte';
  import Icon, { type IconName } from '../Icon.svelte';
  import RuleRow from './RuleRow.svelte';

  let { category }: { category: Category } = $props();

  const icons: Record<Category, IconName> = { system: 'system', gaming: 'gaming', browsers: 'browsers', apps: 'apps' };
  const rows = $derived(flow.inCategory(category));
  const bytes = $derived(flow.categoryBytes(category));
  const cleanable = $derived(rows.filter(({ finding }) => !finding.skip).map(({ rule }) => rule.id));
  const allTicked = $derived(cleanable.length > 0 && cleanable.every((id) => flow.ticked.has(id)));

  function tickAll() {
    for (const id of cleanable) {
      if (allTicked) flow.ticked.delete(id);
      else flow.ticked.add(id);
    }
  }
</script>

<article class="card">
  <header>
    <span class="icon"><Icon name={icons[category]} size={18} /></span>
    <h2>{t(`cat.${category}`)}</h2>
    <span class="size chiffres">{formatSize(bytes)}</span>
    {#if cleanable.length > 1}
      <button class="all" onclick={tickAll}>{t(allTicked ? 'results.untickAll' : 'results.tickAll')}</button>
    {/if}
  </header>
  <div class="rows">
    {#each rows as { rule, finding } (rule.id)}
      <RuleRow {rule} {finding} />
    {/each}
  </div>
</article>

<style>
  .card {
    padding: 8px;
    border-radius: var(--rayon-lg);
    background: var(--carbone);
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 12px 8px;
  }
  .icon {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 9px;
    color: var(--os-2);
    background: rgb(255 255 255 / 0.05);
  }
  h2 {
    font-size: 15px;
    font-weight: 600;
  }
  .size {
    color: var(--gris);
    font-size: 13px;
  }
  .all {
    margin-left: auto;
    padding: 5px 9px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--gris);
  }
  .all:hover {
    color: var(--os);
    background: rgb(255 255 255 / 0.05);
  }
  .rows {
    display: flex;
    flex-direction: column;
  }
</style>
