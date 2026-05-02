<script lang="ts">
  import { uiStore } from "../stores/ui.svelte";
  import { docStore } from "../stores/document.svelte";
  import { prefsStore } from "../stores/prefs.svelte";
  import NewDocModal from "./modals/NewDocModal.svelte";
  import PrefsModal from "./modals/PrefsModal.svelte";
  import StyleEditorModal from "./modals/StyleEditorModal.svelte";

  let { currentEditingStyle = $bindable(), currentEditingCharStyle = $bindable(), currentEditingSwatch = $bindable() } = $props();
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

<!-- Add other modals as they are refactored -->
