<script lang="ts">
  import CustomSelect from './CustomSelect.svelte';
  import CustomToggle from './CustomToggle.svelte';
  import { t } from '$lib/i18n';
  import {
    playbackState,
    setLang,
    setPrefetch,
    setQari,
    setReadTafsir,
    setTtsEnabled
  } from '$lib/stores/playback.svelte';

  const qariOptions = $derived([
    { value: 'misyari', label: 'Mishary Rashid Alafasy', description: t('settings.qari.misyari') },
    { value: 'sudais', label: 'Abdurrahman As-Sudais', description: t('settings.qari.sudais') },
    { value: 'dossari', label: 'Ibrahim Al-Dossari', description: t('settings.qari.dossari') },
    { value: 'juhany', label: 'Abdullah Al-Juhany', description: t('settings.qari.juhany') },
    { value: 'qasim', label: 'Abdul Muhsin Al-Qasim', description: t('settings.qari.qasim') },
    { value: 'yasser', label: 'Yasser Al-Dosari', description: t('settings.qari.yasser') }
  ]);

  const languageOptions = $derived([
    { value: 'id', label: t('settings.lang.id'), description: t('settings.lang.idHint') },
    { value: 'en', label: t('settings.lang.en'), description: t('settings.lang.enHint') }
  ]);

  let isTtsVoiceReady = $derived(playbackState.ttsVoiceStatus === 'ready');
  let ttsToggleDisabled = $derived(
    playbackState.ttsVoiceStatus === 'checking' || playbackState.ttsVoiceStatus === 'failed'
  );
</script>

<div class="flex flex-col gap-4">
  <CustomSelect
    label={t('settings.qari')}
    value={playbackState.qari}
    options={qariOptions}
    onChange={setQari}
  />

  <CustomSelect
    label={t('settings.language')}
    value={playbackState.lang}
    options={languageOptions}
    onChange={setLang}
  />

  {#if isTtsVoiceReady || ttsToggleDisabled}
    <CustomToggle
      label={t('settings.tts')}
      description={t('settings.ttsHint')}
      checked={playbackState.ttsEnabled}
      disabled={ttsToggleDisabled}
      onChange={setTtsEnabled}
    />

    <CustomToggle
      label={t('settings.prefetch')}
      description={t('settings.prefetchHint')}
      checked={playbackState.prefetch}
      disabled={ttsToggleDisabled}
      onChange={setPrefetch}
    />

    <CustomToggle
      label={t('settings.readTafsir')}
      description={t('settings.readTafsirHint')}
      checked={playbackState.readTafsir}
      disabled={ttsToggleDisabled}
      onChange={setReadTafsir}
    />
  {/if}
</div>
