<!-- A dialog in the middle of the screen, above a dimmed and blurred app. It slides in from the
     left and leaves to the right. Escape and a click outside close it when `onclose` is given. -->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { motion } from '$lib/settings.svelte';
  import { fade, flyOut, slideIn } from '$lib/transitions';

  let {
    onclose,
    width = 460,
    label,
    children,
  }: { onclose?: () => void; width?: number; label: string; children: Snippet } = $props();

  let card: HTMLDivElement | undefined = $state();

  $effect(() => {
    card?.focus({ preventScroll: true });
  });

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && onclose) {
      e.preventDefault();
      onclose();
    } else if (e.key === 'Enter' && document.activeElement === card) {
      // Enter answers with the main action: the last button.
      e.preventDefault();
      const buttons = card?.querySelectorAll<HTMLButtonElement>('.actions button:not(:disabled)');
      buttons?.[buttons.length - 1]?.click();
    }
  }
</script>

<svelte:window {onkeydown} />

<div class="backdrop" transition:fade={{ duration: motion.ms(220) }} onclick={onclose} role="presentation"></div>
<div class="layer" role="presentation">
  <div
    bind:this={card}
    class="card"
    role="dialog"
    tabindex="-1"
    aria-modal="true"
    aria-label={label}
    style:width="min({width}px, calc(100vw - 48px))"
    in:slideIn={{ duration: motion.ms(420), delay: motion.ms(60), x: -36 }}
    out:flyOut={{ duration: motion.ms(200) }}
  >
    {@render children()}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 80;
    background: rgb(5 5 6 / 0.72);
    backdrop-filter: blur(6px);
  }
  .layer {
    position: fixed;
    inset: 0;
    z-index: 81;
    display: grid;
    place-items: center;
    pointer-events: none;
  }
  .card {
    position: relative;
    max-height: calc(100vh - 80px);
    overflow: auto;
    padding: 32px;
    border-radius: var(--rayon-lg);
    background: var(--carbone);
    box-shadow: var(--ombre);
    pointer-events: auto;
    outline: none;
  }
  /* The acid edge on the left, slanted like everything else. */
  .card::before {
    content: '';
    position: absolute;
    left: 0;
    top: 28px;
    width: 4px;
    height: 44px;
    background: var(--acide);
    transform: skewY(var(--penche));
  }
  .card :global(.actions) {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 28px;
  }
</style>
