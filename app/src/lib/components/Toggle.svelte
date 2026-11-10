<script lang="ts">
  let { checked, label, onchange }: { checked: boolean; label: string; onchange: (value: boolean) => void } = $props();
</script>

<button class="toggle" class:on={checked} role="switch" aria-checked={checked} aria-label={label} onclick={() => onchange(!checked)}>
  <span class="knob"></span>
</button>

<style>
  .toggle {
    position: relative;
    width: 44px;
    height: 24px;
    flex: none;
    border-radius: 999px;
    background: var(--carbone-3);
    transition: background var(--micro);
  }
  .toggle.on {
    background: var(--acide);
  }
  .knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--os);
    transition:
      transform var(--transition) var(--entree),
      background var(--micro);
  }
  .on .knob {
    transform: translateX(20px);
    background: var(--sur-acide);
  }
</style>
