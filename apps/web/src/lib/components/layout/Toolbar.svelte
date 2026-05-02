<script lang="ts">
  import { uiStore } from "../../stores/ui.svelte";
</script>

<aside class="toolbar">
  <button 
    class:active={uiStore.activeTool === 'select'} 
    onclick={() => uiStore.activeTool = 'select'}
    title="Auswählen (V)"
  >V</button>
  <button 
    class:active={uiStore.activeTool === 'text'} 
    onclick={() => uiStore.activeTool = 'text'}
    title="Text (T)"
  >T</button>
  <button 
    class:active={uiStore.activeTool === 'image'} 
    onclick={() => uiStore.activeTool = 'image'}
    title="Rahmen (F)"
  >F</button>
</aside>

<style>
  .toolbar { position: absolute; left: 0; top: 32px; bottom: 25px; width: 40px; background: #2d2d2d; border-right: 1px solid #111; display: flex; flex-direction: column; align-items: center; padding-top: 10px; gap: 5px; z-index: 10; }
  .toolbar button { width: 30px; height: 30px; background: transparent; border: none; color: #ccc; border-radius: 4px; cursor: pointer; }
  .toolbar button.active { background: #007acc; color: white; }
  .toolbar button:hover:not(.active) { background: #3d3d3d; }
</style>
