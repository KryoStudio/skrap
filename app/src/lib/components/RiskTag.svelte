<script lang="ts">
  import type { Risk } from '$lib/api';
  import { t } from '$lib/i18n.svelte';

  let { risk }: { risk: Risk } = $props();
</script>

<span class="tag {risk}">{t(`risk.${risk}`)}</span>

<style>
  .tag {
    display: inline-flex;
    align-items: center;
    height: 19px;
    padding: 0 7px;
    border-radius: 4px;
    font-size: 10.5px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    white-space: nowrap;
  }
  .safe {
    color: var(--acide);
    background: var(--acide-doux);
  }
  .review {
    color: var(--ambre);
    background: var(--ambre-doux);
  }
  .expert {
    color: var(--magenta);
    background: rgb(255 61 139 / 0.12);
  }
</style>
