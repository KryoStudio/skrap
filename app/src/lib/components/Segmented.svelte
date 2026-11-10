<!-- A choice between a few options; the acid block slides to the chosen one. -->
<script lang="ts" generics="T extends string | number | null">
  let {
    options,
    value,
    label,
    onchange,
  }: { options: { value: T; label: string }[]; value: T; label: string; onchange: (value: T) => void } = $props();

  const index = $derived(Math.max(0, options.findIndex((o) => o.value === value)));
</script>

<div class="segmented" role="radiogroup" aria-label={label} style:--count={options.length}>
  <span class="thumb" style:transform="translateX({index * 100}%)"></span>
  {#each options as option (option.value)}
    <button role="radio" aria-checked={option.value === value} class:on={option.value === value} onclick={() => onchange(option.value)}>
      {option.label}
    </button>
  {/each}
</div>

<style>
  .segmented {
    position: relative;
    display: grid;
    grid-template-columns: repeat(var(--count), 1fr);
    padding: 3px;
    border-radius: var(--rayon-sm);
    background: var(--noir);
    box-shadow: inset 0 0 0 1px var(--trait-fort);
  }
  .thumb {
    position: absolute;
    top: 3px;
    bottom: 3px;
    left: 3px;
    width: calc((100% - 6px) / var(--count));
    border-radius: 5px;
    background: var(--acide);
    transition: transform var(--transition) var(--entree);
  }
  button {
    position: relative;
    height: 30px;
    padding: 0 14px;
    font-size: 12.5px;
    font-weight: 600;
    color: var(--gris);
    white-space: nowrap;
    transition: color var(--micro);
  }
  button:hover:not(.on) {
    color: var(--os);
  }
  button.on {
    color: var(--sur-acide);
  }
</style>
