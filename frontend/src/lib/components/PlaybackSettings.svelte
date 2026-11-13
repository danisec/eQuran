<script lang="ts">
  import { tick } from 'svelte';
  import CacheSection from './CacheSection.svelte';
  import Modal from './Modal.svelte';
  import QariLanguageFields from './QariLanguageFields.svelte';
  import VoiceStatus from './VoiceStatus.svelte';
  import { t } from '$lib/i18n';
  import { playbackState, refreshCacheStatus, refreshTtsVoiceStatus } from '$lib/stores/playback.svelte';
  import { surahState } from '$lib/stores/surah.svelte';
  import { closeSettings, panelState } from '$lib/stores/panels.svelte';

  // Refresh when the modal opens, and again whenever the surah or reciter
  // changes while it stays open, so the figure is never stale.
  let wasOpen = $state(false);
  $effect(() => {
    if (!panelState.settingsOpen) {
      wasOpen = false;
      return;
    }
    const surah = surahState.selected.nomor;
    const totalAyat = surahState.selected.jumlahAyat;
    const qari = playbackState.qari;

    if (!wasOpen) {
      wasOpen = true;
      void refreshTtsVoiceStatus();
      void tick();
    }
    void refreshCacheStatus(surah, totalAyat, qari);
  });
</script>

<Modal
  open={panelState.settingsOpen}
  title={t('settings.title')}
  subtitle={t('settings.subtitle')}
  labelledBy="settings-heading"
  onClose={closeSettings}
>
  <div class="flex flex-col gap-4">
    <VoiceStatus />
    <QariLanguageFields />
    <CacheSection />
  </div>
</Modal>
