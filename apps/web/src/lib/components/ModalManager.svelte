<script lang="ts">
  import { uiStore } from "../stores/ui.svelte";
  import { docStore } from "../stores/document.svelte";
  import { prefsStore } from "../stores/prefs.svelte";
  import NewDocModal from "./modals/NewDocModal.svelte";
  import PrefsModal from "./modals/PrefsModal.svelte";
  import StyleEditorModal from "./modals/StyleEditorModal.svelte";
  import SwatchModal from "./modals/SwatchModal.svelte";

  let { currentEditingStyle = $bindable(), currentEditingCharStyle = $bindable(), currentEditingSwatch = $bindable(), onCloseSwatch } = $props();
</script>

{#if uiStore.showNewDocModal}
  <NewDocModal onCreate={(s: any) => docStore.createNew(s)} />
{/if}

{#if uiStore.showPrefsModal}
  <PrefsModal onSave={() => prefsStore.save()} />
{/if}

{#if uiStore.showStyleEditorModal}
  <StyleEditorModal onFinish={() => uiStore.showStyleEditorModal = false} />
{/if}

{#if uiStore.showSwatchModal && currentEditingSwatch}
  <SwatchModal 
    swatch={currentEditingSwatch} 
    onFinish={() => { uiStore.showSwatchModal = false; onCloseSwatch(); }} 
  />
{/if}

<!-- Add other modals as they are refactored -->
