<script lang="ts">
  import { t } from '$lib/i18n.svelte';
  import { answer, dialog } from '$lib/ui.svelte';
  import Modal from './Modal.svelte';
</script>

{#if dialog.open && dialog.options}
  <Modal label={dialog.options.title} onclose={() => answer(false)} width={420}>
    <h2 class="titre">{dialog.options.title}</h2>
    <p>{dialog.options.body}</p>
    <div class="actions">
      <button class="btn btn-ghost" onclick={() => answer(false)}>{t('confirm.cancel')}</button>
      <button class="btn btn-line" onclick={() => answer(true)}>{dialog.options.confirm}</button>
    </div>
  </Modal>
{/if}

<style>
  h2 {
    font-size: 28px;
  }
  p {
    margin-top: 12px;
    color: var(--os-2);
    line-height: 1.55;
  }
</style>
