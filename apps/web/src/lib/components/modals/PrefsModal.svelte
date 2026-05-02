<script lang="ts">
  import { prefsStore } from "../../stores/prefs.svelte";
  import { uiStore } from "../../stores/ui.svelte";

  let { onSave } = $props();

  function handleCancel() {
    uiStore.showPrefsModal = false;
  }
</script>

<div class="modal-backdrop">
  <div class="modal">
    <div class="modal-header">Einstellungen</div>
    <div class="modal-body">
      <label>
        Erscheinungsbild
        <select bind:value={prefsStore.prefs.theme}>
          <option value="dark">Dunkel</option>
          <option value="light">Hell</option>
        </select>
      </label>
      <label>
        Standardeinheit
        <select bind:value={prefsStore.prefs.default_unit}>
          <option value="pt">Punkt (pt)</option>
          <option value="mm">Millimeter (mm)</option>
          <option value="cm">Zentimeter (cm)</option>
          <option value="in">Zoll (in)</option>
        </select>
      </label>
      <label>
        Auto-Save Intervall (Sekunden)
        <input type="number" bind:value={prefsStore.prefs.autosave_interval} />
      </label>
    </div>
    <div class="modal-footer">
      <button onclick={handleCancel}>Abbrechen</button>
      <button class="primary" onclick={onSave}>Speichern</button>
    </div>
  </div>
</div>

<style>
  .modal-backdrop { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 1000; }
  .modal { background: #2d2d2d; width: 400px; border-radius: 4px; box-shadow: 0 10px 30px rgba(0,0,0,0.5); border: 1px solid #444; color: #ccc; }
  .modal-header { padding: 12px; background: #333; font-weight: bold; border-bottom: 1px solid #444; }
  .modal-body { padding: 15px; display: flex; flex-direction: column; gap: 12px; }
  .modal-footer { padding: 12px; border-top: 1px solid #444; display: flex; justify-content: flex-end; gap: 10px; }
  input, select { background: #3c3c3c; border: 1px solid #555; color: white; padding: 4px 8px; border-radius: 2px; }
  button.primary { background: #007acc; color: white; border: none; padding: 6px 12px; border-radius: 2px; cursor: pointer; }
  button:not(.primary) { background: transparent; border: 1px solid #555; color: #ccc; padding: 6px 12px; border-radius: 2px; cursor: pointer; }
</style>
