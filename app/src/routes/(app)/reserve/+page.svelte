<!-- The Reserve: each clean that kept files, until when, and one click to bring it all back. -->
<script lang="ts">
  import { api, type Batch, type ReservedFile } from '$lib/api';
  import { flow } from '$lib/flow.svelte';
  import { formatDate, formatMoment, formatSize, t } from '$lib/i18n.svelte';
  import { reserve } from '$lib/reserve.svelte';
  import { motion, settings } from '$lib/settings.svelte';
  import { flyOut, slideIn } from '$lib/transitions';
  import BigNumber from '$lib/components/BigNumber.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Logo from '$lib/components/Logo.svelte';

  let open = $state<string | null>(null);
  let files = $state<ReservedFile[]>([]);

  $effect(() => {
    void reserve.load();
  });

  async function toggle(batch: Batch) {
    if (open === batch.id) {
      open = null;
      return;
    }
    open = batch.id;
    files = [];
    files = await api.getReserveFiles(batch.id, 60);
  }

  /** The rules a batch came from, by their names. */
  function origin(batch: Batch): string {
    const names = batch.rules.map((id) => {
      const rule = flow.rule(id);
      return rule ? `${rule.group} · ${rule.name}` : id;
    });
    return [...new Set(names)].join(', ');
  }
</script>

<section class="reserve">
  <header>
    <div>
      <h1 class="titre" in:slideIn={{ duration: motion.ms(480), x: -40 }}>{t('reserve.title')}</h1>
      <p class="lead">{t('reserve.lead', { days: settings.value.reserveDays })}</p>
    </div>
    {#if reserve.batches.length > 0}
      <div class="total">
        <BigNumber bytes={reserve.bytes} size={52} />
        <p class="files chiffres">{t('reserve.files', { count: reserve.files })}</p>
      </div>
    {/if}
  </header>

  {#if reserve.loaded && reserve.batches.length === 0}
    <div class="empty" in:slideIn={{ duration: motion.ms(420) }}>
      <Logo size={64} color="var(--carbone-3)" />
      <h2>{t('reserve.empty')}</h2>
      <p>{t('reserve.emptyBody')}</p>
    </div>
  {:else}
    <ul class="batches">
      {#each reserve.batches as batch, i (batch.id)}
        <li
          class:leaving={reserve.leaving.includes(batch.id)}
          in:slideIn={{ delay: motion.ms(i * 40), duration: motion.ms(380) }}
          out:flyOut={{ duration: motion.ms(320), x: -80 }}
        >
          <div class="batch">
            <div class="when">
              <strong>{formatMoment(batch.created)}</strong>
              <span class="origin">{origin(batch)}</span>
              <span class="expires"><Icon name="clock" size={13} />{t('reserve.expires', { date: formatDate(batch.expires) })}</span>
            </div>
            <div class="size">
              <span class="titre chiffres">{formatSize(batch.bytes)}</span>
              <span class="count chiffres">{t('reserve.files', { count: batch.files })}</span>
            </div>
            <div class="actions">
              <button class="btn btn-ghost small" onclick={() => toggle(batch)} aria-expanded={open === batch.id}>
                {open === batch.id ? t('reserve.hide') : t('reserve.show')}
              </button>
              <button class="btn btn-ghost small" onclick={() => reserve.release(batch)}>{t('reserve.release')}</button>
              <button class="btn btn-line small" onclick={() => reserve.restore(batch)} disabled={reserve.leaving.includes(batch.id)}>
                <Icon name="restore" size={14} />{t('reserve.restore')}
              </button>
            </div>
          </div>
          {#if open === batch.id}
            <ul class="files-list" in:slideIn={{ duration: motion.ms(240), x: -10, skew: 0 }}>
              {#each files as file (file.path)}
                <li>
                  <button onclick={() => api.reveal(file.path)} title={t('results.reveal')}>
                    <span class="path"><span dir="ltr">{file.path}</span></span>
                    <span class="chiffres">{formatSize(file.size)}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </li>
      {/each}
    </ul>

    {#if reserve.batches.length > 1}
      <div class="all">
        <button class="btn btn-ghost" onclick={() => reserve.release(null)}>
          <Icon name="trash" size={15} />{t('reserve.releaseAll')}
        </button>
      </div>
    {/if}
  {/if}
</section>

<style>
  .reserve {
    padding: 40px 56px 48px;
  }
  header {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 32px;
    margin-bottom: 32px;
  }
  h1 {
    font-size: 64px;
  }
  .lead {
    max-width: 520px;
    margin-top: 14px;
    color: var(--gris);
    font-size: 15px;
    line-height: 1.55;
  }
  .total {
    text-align: right;
  }
  .files {
    margin-top: 4px;
    font-size: 12.5px;
    color: var(--gris);
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
    padding: 48px 40px;
    border-radius: var(--rayon-lg);
    background: var(--carbone);
  }
  .empty h2 {
    margin-top: 10px;
    font-size: 18px;
    font-weight: 600;
  }
  .empty p {
    color: var(--gris);
    max-width: 440px;
    line-height: 1.55;
  }

  .batches {
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-width: 1040px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .batches > li {
    padding: 18px 20px;
    border-radius: var(--rayon-lg);
    background: var(--carbone);
    transition:
      transform 420ms var(--entree),
      opacity 420ms;
  }
  /* Restoring: the batch goes back where it came from, to the left. */
  .batches > li.leaving {
    transform: translateX(-24px);
    opacity: 0.5;
  }
  .batch {
    display: grid;
    grid-template-columns: 1fr auto auto;
    align-items: center;
    gap: 24px;
  }
  .when {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }
  strong {
    font-weight: 600;
    font-size: 15px;
  }
  .origin {
    overflow: hidden;
    font-size: 12.5px;
    color: var(--os-2);
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .expires {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--gris);
  }
  .size {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
  }
  .size .titre {
    font-size: 26px;
  }
  .count {
    font-size: 12px;
    color: var(--gris);
  }
  .actions {
    display: flex;
    gap: 6px;
  }
  .small {
    height: 32px;
    padding: 0 12px;
    font-size: 12.5px;
  }

  .files-list {
    max-height: 240px;
    overflow-y: auto;
    margin: 14px 0 0;
    padding: 8px;
    border-radius: var(--rayon-sm);
    background: var(--noir);
    list-style: none;
  }
  .files-list button {
    display: flex;
    gap: 16px;
    width: 100%;
    padding: 5px 8px;
    border-radius: 5px;
    font-size: 12px;
    color: var(--gris);
    text-align: left;
  }
  .files-list button:hover {
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

  .all {
    margin-top: 20px;
  }
</style>
