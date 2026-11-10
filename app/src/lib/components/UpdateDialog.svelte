<!-- A new version, in the middle of the screen: what it brings, then "Update" or "Later". While
     it installs, an acid bar fills and Skrap restarts on its own. -->
<script lang="ts">
  import { formatNumber, t } from '$lib/i18n.svelte';
  import { updates } from '$lib/updates.svelte';
  import Logo from './Logo.svelte';
  import Modal from './Modal.svelte';

  const installing = $derived(updates.installing);
  const percent = $derived(Math.round((installing ?? 0) * 100));
</script>

<Modal label={t('update.eyebrow')} onclose={installing === null ? () => updates.later() : undefined} width={480}>
  <div class="head">
    <div class="mark"><Logo size={56} /></div>
    <div>
      <p class="eyebrow acid">{t('update.eyebrow')}</p>
      <h2 class="titre">{t('update.title', { version: updates.version })}</h2>
    </div>
  </div>

  <p class="body">{updates.summary || t('update.body')}</p>

  {#if installing !== null}
    <div class="progress" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={percent}>
      <span style:transform="scaleX({Math.max(0.02, installing)})"></span>
    </div>
    <p class="status chiffres">
      {installing >= 1 ? t('update.installing') : t('update.downloading', { percent: formatNumber(percent) })}
    </p>
  {:else}
    <div class="actions">
      <button class="btn btn-ghost" onclick={() => updates.later()}>{t('update.later')}</button>
      <button class="btn btn-primary" onclick={() => updates.install()}>{t('update.now')}</button>
    </div>
  {/if}
</Modal>

<style>
  .head {
    display: flex;
    align-items: center;
    gap: 18px;
  }
  .mark {
    display: grid;
    place-items: center;
    width: 76px;
    height: 76px;
    flex: none;
    border-radius: 18px;
    background: var(--noir);
    box-shadow: inset 0 0 0 1px var(--trait-fort);
    animation: arrive 760ms var(--entree) both 120ms;
  }
  @keyframes arrive {
    from {
      transform: translateX(-36px) skewX(var(--penche));
      opacity: 0;
    }
  }
  .acid {
    color: var(--acide);
  }
  h2 {
    margin-top: 6px;
    font-size: 40px;
  }
  .body {
    margin-top: 20px;
    color: var(--os-2);
    line-height: 1.6;
  }

  .progress {
    position: relative;
    height: 10px;
    margin-top: 28px;
    overflow: hidden;
    border-radius: 2px;
    background: var(--carbone-3);
    transform: skewX(var(--penche));
  }
  .progress span {
    position: absolute;
    inset: 0;
    transform-origin: left;
    background: repeating-linear-gradient(
        115deg,
        transparent 0 10px,
        rgb(11 11 12 / 0.18) 10px 20px
      ),
      var(--acide);
    background-size: 40px 100%;
    transition: transform 200ms linear;
    animation: stripes 700ms linear infinite;
  }
  @keyframes stripes {
    to {
      background-position: 40px 0;
    }
  }
  .status {
    margin-top: 12px;
    font-size: 13px;
    color: var(--gris);
  }
</style>
