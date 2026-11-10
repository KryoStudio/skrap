<!-- Shown once after an update: what this version brings. -->
<script lang="ts">
  import type { Release } from '$lib/changelog';
  import { i18n, t } from '$lib/i18n.svelte';
  import { motion } from '$lib/settings.svelte';
  import { slideIn } from '$lib/transitions';
  import Modal from './Modal.svelte';

  let { release, onclose }: { release: Release; onclose: () => void } = $props();

  const notes = $derived(release.notes[i18n.lang] ?? release.notes.en);
</script>

<Modal label={t('whatsNew.title', { version: release.version })} {onclose} width={500}>
  <p class="eyebrow acid">{t('whatsNew.eyebrow')}</p>
  <h2 class="titre">{t('whatsNew.title', { version: release.version })}</h2>
  <ul>
    {#each notes as note, i (note)}
      <li in:slideIn={{ delay: motion.ms(260 + i * 60), duration: motion.ms(360), x: -20 }}>{note}</li>
    {/each}
  </ul>
  <div class="actions">
    <button class="btn btn-primary" onclick={onclose}>{t('whatsNew.ok')}</button>
  </div>
</Modal>

<style>
  .acid {
    color: var(--acide);
  }
  h2 {
    margin-top: 8px;
    font-size: 34px;
  }
  ul {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin: 22px 0 0;
    padding: 0;
    list-style: none;
  }
  li {
    position: relative;
    padding-left: 22px;
    color: var(--os-2);
    line-height: 1.55;
  }
  li::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0.6em;
    width: 10px;
    height: 4px;
    background: var(--acide);
    transform: skewX(var(--penche));
  }
</style>
