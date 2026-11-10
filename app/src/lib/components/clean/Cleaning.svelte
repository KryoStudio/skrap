<!-- The clean: each rule's line flies off to the right with an acid trail as it is done, and the
     big number of what is cleaned runs up. -->
<script lang="ts">
  import { flow } from '$lib/flow.svelte';
  import { formatSize, t } from '$lib/i18n.svelte';
  import { motion } from '$lib/settings.svelte';
  import { slideIn } from '$lib/transitions';
  import BigNumber from '../BigNumber.svelte';

  const p = $derived(flow.cleanProgress);
</script>

<section class="cleaning">
  <div class="counter">
    <BigNumber bytes={p.bytes} size={112} duration={260} />
    <p class="label titre">{t('clean.cleaning')}</p>
    <p class="path">{p.current}</p>
  </div>

  <ul>
    {#each flow.cleaning as id, i (id)}
      {@const rule = flow.rule(id)}
      {@const finding = flow.finding(id)}
      {#if rule && finding}
        <li
          class:gone={i < p.stepsDone}
          class:now={i === p.stepsDone}
          in:slideIn={{ delay: motion.ms(i * 30), duration: motion.ms(320) }}
        >
          <span class="name"><b>{rule.group}</b> · {rule.name}</span>
          <span class="size chiffres">{formatSize(finding.bytes)}</span>
        </li>
      {/if}
    {/each}
  </ul>

  <div class="foot">
    <button class="btn btn-ghost" onclick={() => flow.cancel()}>{t('clean.stop')}</button>
  </div>
</section>

<style>
  .cleaning {
    display: flex;
    flex-direction: column;
    gap: 32px;
    padding: 40px 56px 40px;
    overflow: hidden;
  }
  .label {
    margin-top: 6px;
    font-size: 30px;
    color: var(--acide);
  }
  .path {
    height: 18px;
    margin-top: 16px;
    overflow: hidden;
    font-size: 12px;
    color: var(--gris-2);
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  ul {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-width: 720px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li {
    position: relative;
    display: flex;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 16px;
    border-radius: var(--rayon-sm);
    background: var(--carbone);
    transition:
      transform 620ms var(--sortie),
      opacity 620ms var(--sortie),
      background var(--micro);
  }
  /* The trail, attached behind the line, stretches as it leaves. */
  li::before {
    content: '';
    position: absolute;
    top: 50%;
    right: 100%;
    width: 0;
    height: 3px;
    margin-top: -1.5px;
    background: linear-gradient(90deg, transparent, var(--acide));
    transition: width 620ms var(--sortie);
  }
  li.now {
    background: var(--carbone-2);
    box-shadow: inset 3px 0 0 var(--acide);
  }
  li.gone {
    transform: translateX(calc(100vw - 200px)) skewX(var(--penche));
    opacity: 0;
  }
  li.gone::before {
    width: 360px;
  }
  .name {
    color: var(--os-2);
  }
  b {
    font-weight: 600;
    color: var(--os);
  }
  .size {
    font-weight: 600;
  }
  .foot {
    margin-top: auto;
  }
</style>
