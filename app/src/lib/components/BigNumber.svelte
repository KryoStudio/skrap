<!-- A size in Inter Display Black Italic that runs up to its value like a speedometer, digits of
     fixed width so it never jumps. -->
<script lang="ts">
  import { untrack } from 'svelte';
  import { cubicOut } from 'svelte/easing';
  import { Tween } from 'svelte/motion';
  import { sizeParts } from '$lib/i18n.svelte';
  import { motion } from '$lib/settings.svelte';

  let {
    bytes,
    size = 96,
    duration = 600,
    from,
  }: { bytes: number; size?: number; duration?: number; from?: number } = $props();

  // Starts from `from` once (a scan's last count, or 0), then follows `bytes`.
  const shown = new Tween(untrack(() => from ?? 0), { easing: cubicOut });

  $effect(() => {
    void shown.set(bytes, { duration: motion.ms(duration) });
  });

  const parts = $derived(sizeParts(shown.current));
</script>

<span class="number titre chiffres" style:font-size="{size}px">
  {parts.value}<span class="unit">{parts.unit}</span>
</span>

<style>
  .number {
    display: inline-flex;
    align-items: baseline;
    gap: 0.14em;
    white-space: nowrap;
    color: var(--os);
  }
  .unit {
    font-size: 0.62em;
  }
</style>
