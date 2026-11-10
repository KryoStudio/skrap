<!-- The largest files a rule found; a click shows one in File Explorer. -->
<script lang="ts">
  import { api, type RuleFiles } from '$lib/api';
  import { formatSize, t } from '$lib/i18n.svelte';
  import { motion } from '$lib/settings.svelte';
  import { slideIn } from '$lib/transitions';

  let { rule }: { rule: string } = $props();

  const LIMIT = 40;
  let files = $state<RuleFiles | null>(null);

  $effect(() => {
    void api.getRuleFiles(rule, LIMIT).then((f) => (files = f));
  });
</script>

<div class="files" in:slideIn={{ duration: motion.ms(260), x: -12, skew: 0 }}>
  {#if files}
    {#if files.total > files.files.length}
      <p class="note">{t('results.largest', { count: files.files.length })}</p>
    {/if}
    <ul>
      {#each files.files as file (file.path)}
        <li>
          <button title={t('results.reveal')} onclick={() => api.reveal(file.path)}>
            <span class="path"><span dir="ltr">{file.path}</span></span>
            <span class="size chiffres">{formatSize(file.size)}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .files {
    margin: 2px 0 6px 34px;
    padding: 8px;
    border-radius: var(--rayon-sm);
    background: var(--noir);
  }
  .note {
    padding: 4px 8px 6px;
    font-size: 11.5px;
    color: var(--gris);
  }
  ul {
    max-height: 260px;
    overflow-y: auto;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  button {
    display: flex;
    gap: 16px;
    width: 100%;
    padding: 5px 8px;
    border-radius: 5px;
    text-align: left;
    font-size: 12px;
    color: var(--gris);
  }
  button:hover {
    color: var(--os);
    background: rgb(255 255 255 / 0.04);
  }
  .path {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    direction: rtl;
    text-align: left;
  }
  .size {
    flex: none;
  }
</style>
