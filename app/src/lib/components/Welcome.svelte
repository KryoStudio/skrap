<!-- First launch: who Skrap is, in three promises. -->
<script lang="ts">
  import { t, type Key } from '$lib/i18n.svelte';
  import { motion, settings } from '$lib/settings.svelte';
  import { slideIn } from '$lib/transitions';
  import Icon, { type IconName } from './Icon.svelte';
  import Logo from './Logo.svelte';
  import Modal from './Modal.svelte';

  let { onclose }: { onclose: () => void } = $props();

  const promises: { icon: IconName; title: Key; body: Key }[] = [
    { icon: 'eye', title: 'welcome.seeTitle', body: 'welcome.seeBody' },
    { icon: 'restore', title: 'welcome.backTitle', body: 'welcome.backBody' },
    { icon: 'lock', title: 'welcome.localTitle', body: 'welcome.localBody' },
  ];
</script>

<Modal label={t('welcome.title')} width={540}>
  <div class="logo"><Logo size={64} /></div>
  <h2 class="titre">{t('welcome.title')}</h2>
  <p class="lead">{t('welcome.lead')}</p>
  <ul>
    {#each promises as promise, i (promise.title)}
      <li in:slideIn={{ delay: motion.ms(320 + i * 80), duration: motion.ms(420), x: -24 }}>
        <span class="icon"><Icon name={promise.icon} size={18} /></span>
        <div>
          <strong>{t(promise.title)}</strong>
          <p>{t(promise.body, { days: settings.value.reserveDays })}</p>
        </div>
      </li>
    {/each}
  </ul>
  <div class="actions">
    <button class="btn btn-primary go" onclick={onclose}>{t('welcome.go')}</button>
  </div>
</Modal>

<style>
  .logo {
    animation: arrive 800ms var(--entree) both 100ms;
  }
  @keyframes arrive {
    from {
      transform: translateX(-48px) skewX(var(--penche));
      opacity: 0;
    }
  }
  h2 {
    margin-top: 18px;
    font-size: 52px;
  }
  .lead {
    margin-top: 10px;
    color: var(--os-2);
    font-size: 15px;
  }
  ul {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 24px 0 0;
    padding: 0;
    list-style: none;
  }
  li {
    display: flex;
    gap: 14px;
    padding: 14px;
    border-radius: var(--rayon);
    background: rgb(255 255 255 / 0.03);
    box-shadow: inset 0 0 0 1px var(--trait);
  }
  .icon {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    flex: none;
    border-radius: 9px;
    color: var(--acide);
    background: var(--acide-doux);
  }
  strong {
    font-weight: 600;
  }
  li p {
    margin-top: 2px;
    font-size: 13px;
    color: var(--gris);
    line-height: 1.5;
  }
  .go {
    min-width: 160px;
    height: 44px;
    font-size: 17px;
  }
</style>
