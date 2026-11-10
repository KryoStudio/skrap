<!-- The Skrap mark: the block S slanted at 12°, its speed trail on the left. Geometry from
     skrap-logo/01-icone/skrap-icone.svg. -->
<script lang="ts">
  let {
    size = 28,
    color = 'var(--acide)',
    trail = true,
    class: className = '',
  }: { size?: number; color?: string; trail?: boolean; class?: string } = $props();
</script>

<svg class={className} width={size} height={size} viewBox="0 0 200 200" aria-hidden="true">
  <g transform="translate(18 0) skewX(-12)" fill={color}>
    <rect x="58" y="28" width="104" height="32" />
    <rect x="58" y="28" width="32" height="82" />
    <rect x="58" y="84" width="104" height="32" />
    <rect x="130" y="84" width="32" height="88" />
    <rect x="58" y="140" width="104" height="32" />
    {#if trail}
      <rect class="trail" x="14" y="92" width="30" height="16" />
      <rect class="trail far" x="0" y="92" width="8" height="16" opacity="0.6" />
    {/if}
  </g>
</svg>

<style>
  svg {
    display: block;
    flex: none;
    overflow: visible;
  }
</style>
