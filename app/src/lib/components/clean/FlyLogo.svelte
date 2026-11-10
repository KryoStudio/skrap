<!-- The end of a clean: the S crosses the screen from left to right, drifting, with its trail. -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { motion } from '$lib/settings.svelte';
  import Logo from '../Logo.svelte';

  let { onend }: { onend: () => void } = $props();

  onMount(() => {
    if (motion.reduced) onend();
  });
</script>

{#if !motion.reduced}
  <div class="fly" aria-hidden="true" onanimationend={onend}>
    <span class="trail"></span>
    <Logo size={220} />
  </div>
{/if}

<style>
  .fly {
    position: fixed;
    top: 50%;
    left: 0;
    z-index: 70;
    display: flex;
    align-items: center;
    margin-top: -110px;
    pointer-events: none;
    animation: cross 1000ms cubic-bezier(0.45, 0, 0.2, 1) both;
  }
  .trail {
    width: 60vw;
    height: 26px;
    margin-right: -40px;
    background: linear-gradient(90deg, transparent, rgb(214 255 61 / 0.85));
    transform: skewX(var(--penche));
  }
  @keyframes cross {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(100vw);
    }
  }
</style>
