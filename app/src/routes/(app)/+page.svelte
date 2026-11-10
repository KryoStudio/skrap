<!-- Clean: home, scan, results, clean, done. -->
<script lang="ts">
  import { flow } from '$lib/flow.svelte';
  import { motion } from '$lib/settings.svelte';
  import { fade } from '$lib/transitions';
  import Cleaning from '$lib/components/clean/Cleaning.svelte';
  import Done from '$lib/components/clean/Done.svelte';
  import Home from '$lib/components/clean/Home.svelte';
  import Results from '$lib/components/clean/Results.svelte';
  import Scanning from '$lib/components/clean/Scanning.svelte';
</script>

{#key flow.phase}
  <div class="phase" in:fade={{ duration: motion.ms(180) }}>
    {#if flow.phase === 'idle'}
      <Home />
    {:else if flow.phase === 'scanning'}
      <Scanning />
    {:else if flow.phase === 'results'}
      <Results />
    {:else if flow.phase === 'cleaning'}
      <Cleaning />
    {:else}
      <Done />
    {/if}
  </div>
{/key}

<style>
  .phase {
    display: flex;
    flex-direction: column;
  }
  .phase > :global(section) {
    flex: 1;
  }
</style>
