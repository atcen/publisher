<script lang="ts">
  import { docStore } from "../../stores/document.svelte";
  import type { ColorSwatch } from "../../types";

  let { swatch, onFinish } = $props<{ swatch: ColorSwatch, onFinish: () => void }>();

  function toggleType() {
    docStore.toggleSwatchType(swatch);
  }
</script>

<div class="modal-backdrop">
  <div class="modal">
    <div class="modal-header">Farbfeld bearbeiten</div>
    <div class="modal-body">
      <label>Name 
        <input type="text" bind:value={swatch.name} />
      </label>
      
      <div class="type-selector">
        {#if 'Rgb' in swatch.color}
          <span>Typ: RGB</span>
        {:else if 'Cmyk' in swatch.color}
          <span>Typ: CMYK</span>
        {:else if 'Spot' in swatch.color}
          <span>Typ: Vollton (Spot)</span>
        {/if}
        <button onclick={toggleType}>Typ ändern</button>
      </div>

      {#if 'Rgb' in swatch.color}
        <div class="inputs">
          <label>R <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Rgb.r} /></label>
          <label>G <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Rgb.g} /></label>
          <label>B <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Rgb.b} /></label>
        </div>
      {:else if 'Cmyk' in swatch.color}
        <div class="inputs">
          <label>C <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Cmyk.c} /></label>
          <label>M <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Cmyk.m} /></label>
          <label>Y <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Cmyk.y} /></label>
          <label>K <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Cmyk.k} /></label>
        </div>
      {:else if 'Spot' in swatch.color}
        <div class="inputs">
          <label>Tint <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Spot.tint} /></label>
          <fieldset>
            <legend>Ersatz-CMYK (für Vorschau)</legend>
            <label>C <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Spot.alternate_cmyk[0]} /></label>
            <label>M <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Spot.alternate_cmyk[1]} /></label>
            <label>Y <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Spot.alternate_cmyk[2]} /></label>
            <label>K <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Spot.alternate_cmyk[3]} /></label>
          </fieldset>
        </div>
      {/if}
    </div>
    <div class="modal-footer">
      <button class="primary" onclick={onFinish}>Fertig</button>
    </div>
  </div>
</div>

<style>
  .modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; }
  .modal { background: #2d2d2d; border: 1px solid #444; width: 320px; border-radius: 4px; box-shadow: 0 8px 16px rgba(0,0,0,0.4); }
  .modal-header { padding: 12px; border-bottom: 1px solid #444; font-weight: bold; }
  .modal-body { padding: 16px; display: flex; flex-direction: column; gap: 12px; }
  .modal-footer { padding: 12px; border-top: 1px solid #444; display: flex; justify-content: flex-end; }
  label { display: flex; flex-direction: column; gap: 4px; font-size: 11px; color: #aaa; }
  input { background: #3c3c3c; border: 1px solid #555; color: white; padding: 4px 8px; border-radius: 2px; }
  .inputs { display: grid; grid-template-columns: repeat(2, 1fr); gap: 8px; }
  .type-selector { display: flex; justify-content: space-between; align-items: center; background: #333; padding: 8px; border-radius: 4px; }
  fieldset { border: 1px solid #444; padding: 8px; display: grid; grid-template-columns: repeat(2, 1fr); gap: 4px; }
  legend { font-size: 10px; color: #888; }
  button.primary { background: #007acc; color: white; border: none; padding: 6px 12px; border-radius: 2px; cursor: pointer; }
</style>
