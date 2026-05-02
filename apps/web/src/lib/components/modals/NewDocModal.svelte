<script lang="ts">
  import { uiStore } from "../../stores/ui.svelte";

  let { onCreate } = $props();

  const DIN_A4 = { w: 595.27, h: 841.89 };
  let settings = $state({
    name: "Dokument 1",
    width: DIN_A4.w,
    height: DIN_A4.h,
    pages: 1,
    facingPages: true,
    columns: 2,
    gutter: 12,
    margins: { top: 36, bottom: 36, inside: 36, outside: 36 },
    bleed: 0
  });

  function handleCancel() {
    uiStore.showNewDocModal = false;
  }
</script>

<div class="modal-backdrop">
  <div class="modal">
    <div class="modal-header">Neu</div>
    <div class="modal-body">
      <label>Name <input type="text" bind:value={settings.name} /></label>
      <label>Seiten <input type="number" bind:value={settings.pages} /></label>
      <label><input type="checkbox" bind:checked={settings.facingPages} /> Doppelseiten</label>
    </div>
    <div class="modal-footer">
      <button onclick={handleCancel}>Abbrechen</button>
      <button class="primary" onclick={() => onCreate(settings)}>Erstellen</button>
    </div>
  </div>
</div>

<style>
  .modal-backdrop { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 1000; }
  .modal { background: #2d2d2d; width: 400px; border-radius: 4px; box-shadow: 0 10px 30px rgba(0,0,0,0.5); border: 1px solid #444; color: #ccc; }
  .modal-header { padding: 12px; background: #333; font-weight: bold; border-bottom: 1px solid #444; }
  .modal-body { padding: 15px; display: flex; flex-direction: column; gap: 12px; }
  .modal-footer { padding: 12px; border-top: 1px solid #444; display: flex; justify-content: flex-end; gap: 10px; }
  input { background: #3c3c3c; border: 1px solid #555; color: white; padding: 4px 8px; border-radius: 2px; }
  button.primary { background: #007acc; color: white; border: none; padding: 6px 12px; border-radius: 2px; cursor: pointer; }
  button:not(.primary) { background: transparent; border: 1px solid #555; color: #ccc; padding: 6px 12px; border-radius: 2px; cursor: pointer; }
</style>
