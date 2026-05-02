<script lang="ts">
  import { docStore } from "../../stores/document.svelte";
  import type { ColorSwatch } from "../../types";

  let { swatch, onFinish } = $props<{ swatch: ColorSwatch, onFinish: () => void }>();

  function toggleType() {
    docStore.toggleSwatchType(swatch);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onFinish();
    if (e.key === 'Enter') onFinish();
  }

  function validateValue(val: number): number {
    return Math.max(0, Math.min(1, val));
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="modal-backdrop" onclick={onFinish} onkeydown={handleKeydown} role="presentation">
  <div 
    class="modal" 
    onclick={(e) => e.stopPropagation()} 
    role="dialog" 
    aria-modal="true" 
    aria-labelledby="modal-title"
  >
    <div class="modal-header" id="modal-title">Farbfeld bearbeiten</div>
    <div class="modal-body">
      <label>
        Name 
        <input type="text" bind:value={swatch.name} placeholder="Farbfeld Name" required />
      </label>
      
      <div class="type-selector">
        {#if 'Rgb' in swatch.color}
          <span>Typ: RGB</span>
        {:else if 'Cmyk' in swatch.color}
          <span>Typ: CMYK</span>
        {:else if 'Spot' in swatch.color}
          <span>Typ: Vollton (Spot)</span>
        {/if}
        <button onclick={toggleType} title="Farbmodell wechseln">Typ ändern</button>
      </div>

      {#if 'Rgb' in swatch.color}
        <div class="inputs">
          <label>R <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Rgb.r} onchange={(e) => swatch.color.Rgb.r = validateValue(parseFloat(e.currentTarget.value))} /></label>
          <label>G <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Rgb.g} onchange={(e) => swatch.color.Rgb.g = validateValue(parseFloat(e.currentTarget.value))} /></label>
          <label>B <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Rgb.b} onchange={(e) => swatch.color.Rgb.b = validateValue(parseFloat(e.currentTarget.value))} /></label>
        </div>
      {:else if 'Cmyk' in swatch.color}
        <div class="inputs">
          <label>C <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Cmyk.c} onchange={(e) => swatch.color.Cmyk.c = validateValue(parseFloat(e.currentTarget.value))} /></label>
          <label>M <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Cmyk.m} onchange={(e) => swatch.color.Cmyk.m = validateValue(parseFloat(e.currentTarget.value))} /></label>
          <label>Y <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Cmyk.y} onchange={(e) => swatch.color.Cmyk.y = validateValue(parseFloat(e.currentTarget.value))} /></label>
          <label>K <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Cmyk.k} onchange={(e) => swatch.color.Cmyk.k = validateValue(parseFloat(e.currentTarget.value))} /></label>
        </div>
      {:else if 'Spot' in swatch.color}
        <div class="inputs">
          <label>Tint <input type="number" min="0" max="1" step="0.01" bind:value={swatch.color.Spot.tint} onchange={(e) => swatch.color.Spot.tint = validateValue(parseFloat(e.currentTarget.value))} /></label>
          <fieldset>
            <legend>Ersatz-CMYK (für Vorschau)</legend>
            <label>C <input type="number" min="0" max="1" step="0.01" value={swatch.color.Spot.alternate_cmyk[0]} onchange={(e) => { swatch.color.Spot.alternate_cmyk[0] = validateValue(parseFloat(e.currentTarget.value)); }} /></label>
            <label>M <input type="number" min="0" max="1" step="0.01" value={swatch.color.Spot.alternate_cmyk[1]} onchange={(e) => { swatch.color.Spot.alternate_cmyk[1] = validateValue(parseFloat(e.currentTarget.value)); }} /></label>
            <label>Y <input type="number" min="0" max="1" step="0.01" value={swatch.color.Spot.alternate_cmyk[2]} onchange={(e) => { swatch.color.Spot.alternate_cmyk[2] = validateValue(parseFloat(e.currentTarget.value)); }} /></label>
            <label>K <input type="number" min="0" max="1" step="0.01" value={swatch.color.Spot.alternate_cmyk[3]} onchange={(e) => { swatch.color.Spot.alternate_cmyk[3] = validateValue(parseFloat(e.currentTarget.value)); }} /></label>
          </fieldset>
        </div>
      {/if}
    </div>
    <div class="modal-footer">
      <button class="primary" onclick={onFinish} disabled={!swatch.name}>Fertig</button>
    </div>
  </div>
</div>

<style>
  .modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; }
  .modal { background: #2d2d2d; border: 1px solid #444; width: 320px; border-radius: 4px; box-shadow: 0 8px 16px rgba(0,0,0,0.4); outline: none; }
  .modal-header { padding: 12px; border-bottom: 1px solid #444; font-weight: bold; }
  .modal-body { padding: 16px; display: flex; flex-direction: column; gap: 12px; }
  .modal-footer { padding: 12px; border-top: 1px solid #444; display: flex; justify-content: flex-end; }
  label { display: flex; flex-direction: column; gap: 4px; font-size: 11px; color: #aaa; }
  input { background: #3c3c3c; border: 1px solid #555; color: white; padding: 4px 8px; border-radius: 2px; }
  input:invalid { border-color: #ff5555; }
  .inputs { display: grid; grid-template-columns: repeat(2, 1fr); gap: 8px; }
  .type-selector { display: flex; justify-content: space-between; align-items: center; background: #333; padding: 8px; border-radius: 4px; }
  fieldset { border: 1px solid #444; padding: 8px; display: grid; grid-template-columns: repeat(2, 1fr); gap: 4px; }
  legend { font-size: 10px; color: #888; }
  button.primary { background: #007acc; color: white; border: none; padding: 6px 12px; border-radius: 2px; cursor: pointer; }
  button.primary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
