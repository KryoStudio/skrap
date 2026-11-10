<!-- Short notices, bottom right. They slide in from the left and leave to the right. -->
<script lang="ts">
  import { flip } from 'svelte/animate';
  import { motion } from '$lib/settings.svelte';
  import { flyOut, slideIn } from '$lib/transitions';
  import { dismissToast, toasts } from '$lib/ui.svelte';
  import Icon from './Icon.svelte';
</script>

<div class="stack" aria-live="polite">
  {#each toasts as toast (toast.id)}
    <div
      class="toast {toast.kind}"
      role="status"
      in:slideIn={{ duration: motion.ms(320), x: -24 }}
      out:flyOut={{ duration: motion.ms(200) }}
      animate:flip={{ duration: motion.ms(240) }}
    >
      <span class="mark">
        <Icon name={toast.kind === 'error' ? 'alert' : toast.kind === 'success' ? 'check' : 'info'} size={15} stroke={2.2} />
      </span>
      <div class="text">
        <strong>{toast.title}</strong>
        {#if toast.body}<span>{toast.body}</span>{/if}
      </div>
      {#if toast.action}
        <button
          class="action"
          onclick={() => {
            toast.action?.run();
            dismissToast(toast.id);
          }}>{toast.action.label}</button
        >
      {/if}
      <button class="close" onclick={() => dismissToast(toast.id)} aria-label="×"><Icon name="x" size={14} /></button>
    </div>
  {/each}
</div>

<style>
  .stack {
    position: fixed;
    right: 20px;
    bottom: 20px;
    z-index: 90;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 8px;
    pointer-events: none;
  }
  .toast {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 280px;
    max-width: 420px;
    padding: 12px 8px 12px 14px;
    border-radius: var(--rayon);
    background: var(--carbone-2);
    box-shadow: var(--ombre);
    pointer-events: auto;
  }
  .mark {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    flex: none;
    border-radius: 7px;
    color: var(--acide);
    background: var(--acide-doux);
  }
  .error .mark {
    color: var(--ambre);
    background: var(--ambre-doux);
  }
  .info .mark {
    color: var(--os-2);
    background: rgb(255 255 255 / 0.07);
  }
  .text {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }
  strong {
    font-weight: 600;
    font-size: 13.5px;
  }
  .text span {
    font-size: 12.5px;
    color: var(--gris);
  }
  .action {
    padding: 6px 10px;
    border-radius: 6px;
    color: var(--acide);
    font-weight: 600;
    font-size: 13px;
  }
  .action:hover {
    background: var(--acide-doux);
  }
  .close {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 6px;
    color: var(--gris);
  }
  .close:hover {
    color: var(--os);
    background: rgb(255 255 255 / 0.06);
  }
</style>
