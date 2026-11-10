<!-- A box that fills with acid from left to right. -->
<script lang="ts">
  let {
    checked,
    disabled = false,
    label,
    onchange,
  }: { checked: boolean; disabled?: boolean; label: string; onchange: () => void } = $props();
</script>

<button
  class="box"
  class:checked
  role="checkbox"
  aria-checked={checked}
  aria-label={label}
  {disabled}
  onclick={(e) => {
    e.stopPropagation();
    onchange();
  }}
>
  <span class="fill"></span>
  <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5" /></svg>
</button>

<style>
  .box {
    position: relative;
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    flex: none;
    overflow: hidden;
    border-radius: 5px;
    box-shadow: inset 0 0 0 1.5px var(--trait-fort);
    transition: box-shadow var(--micro);
  }
  .box:hover:not(:disabled) {
    box-shadow: inset 0 0 0 1.5px var(--gris);
  }
  .box:disabled {
    opacity: 0.35;
  }
  .fill {
    position: absolute;
    inset: 0;
    background: var(--acide);
    transform: scaleX(0) skewX(var(--penche));
    transform-origin: left;
    transition: transform var(--micro) var(--entree);
  }
  .checked .fill {
    transform: scaleX(1.3) skewX(var(--penche));
  }
  .checked {
    box-shadow: none;
  }
  svg {
    position: relative;
    fill: none;
    stroke: var(--sur-acide);
    stroke-width: 3;
    stroke-linecap: round;
    stroke-linejoin: round;
    stroke-dasharray: 22;
    stroke-dashoffset: 22;
    transition: stroke-dashoffset var(--micro) var(--entree) 60ms;
  }
  .checked svg {
    stroke-dashoffset: 0;
  }
</style>
