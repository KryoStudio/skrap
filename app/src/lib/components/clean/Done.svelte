<!-- After a clean: "Clean.", what was cleaned, what waits in the Reserve, and honestly what was
     left in place and why. -->
<script lang="ts">
  import type { Failure } from '$lib/api';
  import { flow } from '$lib/flow.svelte';
  import { formatDate, formatSize, t } from '$lib/i18n.svelte';
  import { motion } from '$lib/settings.svelte';
  import { slideIn } from '$lib/transitions';
  import BigNumber from '../BigNumber.svelte';
  import Icon from '../Icon.svelte';
  import FlyLogo from './FlyLogo.svelte';

  const done = $derived(flow.done);
  const cleaned = $derived(done ? done.freed + done.reserved : 0);

  function reason(failure: Failure): string {
    if (failure.reason === 'running') return t('fail.running', { app: flow.rule(failure.rule)?.group ?? failure.exe ?? '' });
    return t(`fail.${failure.reason}`);
  }
</script>

{#if flow.crossing}
  <FlyLogo onend={() => (flow.crossing = false)} />
{/if}

{#if done && !flow.crossing}
  <section class="done">
    <h1 class="titre" in:slideIn={{ duration: motion.ms(560), x: -60 }}>
      {cleaned > 0 ? t('done.title') : t('done.nothing')}
    </h1>
    {#if cleaned > 0}
      <span class="underline" aria-hidden="true"></span>

      <div class="freed" in:slideIn={{ delay: motion.ms(160), duration: motion.ms(460) }}>
        <BigNumber bytes={cleaned} from={0} size={72} duration={1100} />
        <span class="titre">{t('done.freed')}</span>
      </div>

      <p class="reserve" in:slideIn={{ delay: motion.ms(240), duration: motion.ms(420) }}>
        {#if done.reserved > 0 && done.reserveUntil}
          {t('done.inReserve', { size: formatSize(done.reserved), date: formatDate(done.reserveUntil) })}
          <a href="/reserve">{t('done.seeReserve')} <Icon name="arrow-right" size={14} /></a>
        {:else}
          {t('done.allFreed')}
        {/if}
      </p>
    {/if}

    {#if done.cancelled}
      <p class="note">{t('done.cancelled')}</p>
    {/if}

    {#if done.failures.length > 0}
      <div class="left" in:slideIn={{ delay: motion.ms(320), duration: motion.ms(420) }}>
        <h2>{t('done.left')}</h2>
        <p class="hint">{t('done.leftHint')}</p>
        <ul>
          {#each done.failures as failure (`${failure.rule}-${failure.reason}`)}
            {@const rule = flow.rule(failure.rule)}
            <li>
              <span class="what">{rule ? `${rule.group} · ${rule.name}` : failure.rule}</span>
              <span class="why">{reason(failure)}</span>
              <span class="size chiffres">{formatSize(failure.bytes)}</span>
            </li>
          {/each}
        </ul>
      </div>
    {/if}

    <div class="actions" in:slideIn={{ delay: motion.ms(380), duration: motion.ms(420) }}>
      <button class="btn btn-primary finish" onclick={() => flow.reset()}>{t('done.finish')}</button>
    </div>
  </section>
{/if}

<style>
  .done {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 44px 56px 44px;
  }
  h1 {
    font-size: clamp(72px, 9vw, 120px);
  }
  .underline {
    display: block;
    width: min(420px, 60%);
    height: 10px;
    margin-top: 14px;
    background: var(--acide);
    transform: skewX(var(--penche));
    transform-origin: left;
    animation: draw 700ms var(--entree) both 200ms;
  }
  @keyframes draw {
    from {
      transform: skewX(var(--penche)) scaleX(0);
    }
  }
  .freed {
    display: flex;
    align-items: baseline;
    gap: 14px;
    margin-top: 36px;
  }
  .freed > .titre {
    font-size: 30px;
    color: var(--acide);
  }
  .reserve {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 14px;
    margin-top: 12px;
    color: var(--os-2);
    font-size: 15px;
  }
  .reserve a {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--acide);
    font-weight: 600;
    text-decoration: none;
  }
  .reserve a:hover {
    text-decoration: underline;
  }
  .note {
    margin-top: 16px;
    color: var(--ambre);
  }

  .left {
    width: min(720px, 100%);
    margin-top: 36px;
    padding: 18px 20px;
    border-radius: var(--rayon-lg);
    background: var(--carbone);
  }
  h2 {
    font-size: 15px;
    font-weight: 600;
  }
  .hint {
    margin-top: 2px;
    font-size: 12.5px;
    color: var(--gris);
  }
  ul {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin: 12px 0 0;
    padding: 0;
    list-style: none;
  }
  li {
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: 16px;
    padding: 8px 0;
    font-size: 13px;
    box-shadow: 0 1px 0 var(--trait);
  }
  li:last-child {
    box-shadow: none;
  }
  .why {
    color: var(--ambre);
  }
  .size {
    min-width: 64px;
    text-align: right;
    color: var(--gris);
  }

  .actions {
    margin-top: auto;
    padding-top: 36px;
  }
  .finish {
    height: 46px;
    padding: 0 30px;
    font-size: 18px;
  }
</style>
