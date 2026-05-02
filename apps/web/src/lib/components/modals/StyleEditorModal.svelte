<script lang="ts">
  import { docStore } from "../../stores/document.svelte";
  import { uiStore } from "../../stores/ui.svelte";

  let { onFinish } = $props();

  function handleAddVariation(tag: string, value: number) {
    // let s = docStore.doc.styles.paragraph_styles.find(x => x.name === uiStore.showStyleEditorModal); 
  }
</script>

<!-- Simplified placeholder for now, will refine -->
<div class="modal-backdrop">
  <div class="modal">
    <div class="modal-header">Stil bearbeiten</div>
    <div class="modal-body">
      <!-- Edit fields here -->
    </div>
    <div class="modal-footer">
      <button class="primary" onclick={onFinish}>Fertig</button>
    </div>
  </div>
</div>

<style>
  .modal-backdrop { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 1000; }
  .modal { background: #2d2d2d; width: 400px; border-radius: 4px; border: 1px solid #444; color: #ccc; }
  .modal-header { padding: 12px; background: #333; font-weight: bold; border-bottom: 1px solid #444; }
  .modal-body { padding: 15px; }
  .modal-footer { padding: 12px; border-top: 1px solid #444; display: flex; justify-content: flex-end; }
  button.primary { background: #007acc; color: white; border: none; padding: 6px 12px; border-radius: 2px; cursor: pointer; }
</style>
