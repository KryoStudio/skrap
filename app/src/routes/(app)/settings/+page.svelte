<!-- Settings: language, motion, the Reserve's delay, updates, and who made Skrap. -->
<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { native, type Motion } from '$lib/api';
  import { changelog } from '$lib/changelog';
  import { t } from '$lib/i18n.svelte';
  import { languages } from '$lib/locales';
  import { overview } from '$lib/overview.svelte';
  import { motion, settings } from '$lib/settings.svelte';
  import { slideIn } from '$lib/transitions';
  import { toast } from '$lib/ui.svelte';
  import { updates } from '$lib/updates.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Logo from '$lib/components/Logo.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import Toggle from '$lib/components/Toggle.svelte';
  import WhatsNew from '$lib/components/WhatsNew.svelte';

  const REPO = 'https://github.com/KryoStudio/skrap';
  const DAYS = [3, 7, 14, 30];

  let notes = $state(false);

  const languageOptions = $derived([
    { value: null as string | null, label: t('settings.languageSystem') },
    ...Object.entries(languages).map(([code, l]) => ({ value: code as string | null, label: l.name })),
  ]);
  const version = $derived(overview.value?.version ?? '');
  const release = $derived(changelog.find((r) => r.version === version) ?? changelog[0]);

  async function save(patch: Parameters<typeof settings.update>[0]) {
    try {
      await settings.update(patch);
      if ('reserveDays' in patch) void overview.refresh();
    } catch {
      toast({ kind: 'error', title: t('settings.saveFailed') });
    }
  }

  function open(url: string) {
    if (native) void openUrl(url);
    else window.open(url, '_blank');
  }

</script>

<section class="settings">
  <h1 class="titre" in:slideIn={{ duration: motion.ms(480), x: -40 }}>{t('settings.title')}</h1>

  <div class="group" in:slideIn={{ delay: motion.ms(60), duration: motion.ms(380) }}>
    <h2 class="eyebrow">{t('settings.general')}</h2>
    <div class="row">
      <div class="text"><strong>{t('settings.language')}</strong></div>
      <Segmented label={t('settings.language')} options={languageOptions} value={settings.value.language} onchange={(language) => save({ language })} />
    </div>
    <div class="row">
      <div class="text">
        <strong>{t('settings.motion')}</strong>
        <p>{t('settings.motionHint')}</p>
      </div>
      <Segmented
        label={t('settings.motion')}
        options={[
          { value: 'auto' as Motion, label: t('settings.motionAuto') },
          { value: 'reduced' as Motion, label: t('settings.motionReduced') },
        ]}
        value={settings.value.motion}
        onchange={(m) => save({ motion: m })}
      />
    </div>
  </div>

  <div class="group" in:slideIn={{ delay: motion.ms(100), duration: motion.ms(380) }}>
    <h2 class="eyebrow">{t('settings.reserve')}</h2>
    <div class="row">
      <div class="text">
        <strong>{t('settings.reserveDays')}</strong>
        <p>{t('settings.reserveDaysHint')}</p>
      </div>
      <Segmented
        label={t('settings.reserveDays')}
        options={DAYS.map((d) => ({ value: d, label: t('settings.days', { count: d }) }))}
        value={settings.value.reserveDays}
        onchange={(reserveDays) => save({ reserveDays })}
      />
    </div>
  </div>

  <div class="group" in:slideIn={{ delay: motion.ms(140), duration: motion.ms(380) }}>
    <h2 class="eyebrow">{t('settings.updates')}</h2>
    <div class="row">
      <div class="text">
        <strong>{t('settings.checkUpdates')}</strong>
        <p>{t('settings.checkUpdatesHint')}</p>
      </div>
      <Toggle label={t('settings.checkUpdates')} checked={settings.value.checkUpdates} onchange={(checkUpdates) => save({ checkUpdates })} />
    </div>
    <div class="row">
      <div class="text"><strong>{t('settings.version', { version })}</strong></div>
      <div class="buttons">
        <button class="btn btn-ghost" onclick={() => (notes = true)}>{t('settings.whatsNew')}</button>
        <button class="btn btn-ghost" disabled={updates.checking || !native} onclick={() => updates.check(false)}>
          {updates.checking ? t('update.checking') : t('settings.checkNow')}
        </button>
      </div>
    </div>
  </div>

  <div class="group about" in:slideIn={{ delay: motion.ms(180), duration: motion.ms(380) }}>
    <h2 class="eyebrow">{t('settings.about')}</h2>
    <div class="brand">
      <Logo size={44} />
      <div>
        <p class="name titre">SKRAP</p>
        <p class="by">by Kryo Studio</p>
      </div>
    </div>
    <p class="license">{t('settings.license')}</p>
    <div class="links">
      <button class="btn btn-ghost" onclick={() => open(REPO)}><Icon name="external" size={15} />{t('settings.source')}</button>
      <button class="btn btn-ghost" onclick={() => open(`${REPO}/issues`)}><Icon name="alert" size={15} />{t('settings.issues')}</button>
    </div>
    <p class="credits">{t('settings.fonts')}</p>
  </div>
</section>

{#if notes && release}
  <WhatsNew {release} onclose={() => (notes = false)} />
{/if}

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 860px;
    padding: 40px 56px 56px;
  }
  h1 {
    font-size: 64px;
    margin-bottom: 16px;
  }
  .group {
    padding: 8px 22px 10px;
    border-radius: var(--rayon-lg);
    background: var(--carbone);
  }
  .group > h2 {
    padding: 14px 0 6px;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 32px;
    padding: 14px 0;
    box-shadow: 0 -1px 0 var(--trait);
  }
  .group > h2 + .row {
    box-shadow: none;
  }
  .text {
    min-width: 0;
  }
  strong {
    font-weight: 600;
  }
  .text p {
    margin-top: 3px;
    font-size: 12.5px;
    color: var(--gris);
    line-height: 1.5;
    max-width: 440px;
  }
  .buttons {
    display: flex;
    gap: 8px;
  }

  .about {
    padding-bottom: 22px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-top: 8px;
  }
  .name {
    font-size: 30px;
  }
  .by {
    margin-top: 2px;
    font-size: 11px;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: var(--gris);
  }
  .license {
    margin-top: 16px;
    color: var(--os-2);
  }
  .links {
    display: flex;
    gap: 8px;
    margin-top: 16px;
  }
  .credits {
    margin-top: 16px;
    font-size: 12px;
    color: var(--gris-2);
  }
</style>
