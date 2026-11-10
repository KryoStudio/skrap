<!-- One rule in the results: tick it, read why it can go, see its files. -->
<script lang="ts">
  import type { Finding, RuleView } from '$lib/api';
  import { flow } from '$lib/flow.svelte';
  import { formatSize, t } from '$lib/i18n.svelte';
  import { settings } from '$lib/settings.svelte';
  import Checkbox from '../Checkbox.svelte';
  import Icon from '../Icon.svelte';
  import RiskTag from '../RiskTag.svelte';
  import FileList from './FileList.svelte';

  let { rule, finding }: { rule: RuleView; finding: Finding } = $props();

  let open = $state(false);
  const cleanable = $derived(!finding.skip);
  const checked = $derived(flow.ticked.has(rule.id));
</script>

<div class="row" class:off={!cleanable} class:checked>
  <Checkbox {checked} disabled={!cleanable} label={`${rule.group} · ${rule.name}`} onchange={() => flow.toggle(rule.id)} />
  <div class="main">
    <div class="line">
      <span class="name"><span class="group">{rule.group}</span> · {rule.name}</span>
      <RiskTag risk={rule.risk} />
    </div>
    <p class="explain">{rule.explain}</p>
    {#if finding.skip}
      <p class="note warn"><Icon name="lock" size={13} />{t('results.running', { app: rule.group })}</p>
    {:else if rule.special}
      <p class="note"><Icon name="info" size={13} />{t('results.binNote')}</p>
    {:else if rule.risk !== 'safe'}
      <p class="note"><Icon name="restore" size={13} />{t('results.reserveNote', { days: settings.value.reserveDays })}</p>
    {/if}
    {#if rule.admin && cleanable}
      <p class="note"><Icon name="shield" size={13} />{t('results.admin')}</p>
    {/if}
  </div>
  <div class="side">
    <span class="size chiffres">{formatSize(finding.bytes)}</span>
    {#if !rule.special && finding.files > 0}
      <button class="more" class:open onclick={() => (open = !open)} aria-expanded={open}>
        {t('results.files', { count: finding.files })}
        <Icon name="chevron-down" size={14} />
      </button>
    {:else if rule.special}
      <span class="count">{t('results.items', { count: finding.files })}</span>
    {/if}
  </div>
</div>
{#if open}
  <FileList rule={rule.id} />
{/if}

<style>
  .row {
    display: grid;
    grid-template-columns: 20px 1fr auto;
    gap: 14px;
    padding: 14px 12px;
    border-radius: var(--rayon-sm);
    transition: background var(--micro);
  }
  .row:hover {
    background: rgb(255 255 255 / 0.025);
  }
  .row :global(.box) {
    margin-top: 1px;
  }
  .main {
    min-width: 0;
  }
  .line {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }
  .name {
    font-weight: 600;
  }
  .group {
    color: var(--os);
  }
  .off .name {
    color: var(--gris);
  }
  .explain {
    margin-top: 4px;
    font-size: 12.5px;
    color: var(--gris);
    line-height: 1.5;
  }
  .note {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 6px;
    font-size: 12px;
    color: var(--gris);
  }
  .note.warn {
    color: var(--ambre);
  }
  .side {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
  }
  .size {
    font-weight: 600;
    font-size: 14.5px;
    transition: color var(--micro);
  }
  .checked .size {
    color: var(--acide);
  }
  .off .size {
    color: var(--gris);
  }
  .more,
  .count {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 6px;
    margin-right: -6px;
    border-radius: 5px;
    font-size: 12px;
    color: var(--gris);
  }
  .more:hover {
    color: var(--os);
    background: rgb(255 255 255 / 0.05);
  }
  .more :global(svg) {
    transition: transform var(--micro);
  }
  .more.open :global(svg) {
    transform: rotate(180deg);
  }
</style>
