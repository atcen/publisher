<script lang="ts">
  import { docStore } from "../../stores/document.svelte";
  import { uiStore } from "../../stores/ui.svelte";
  import { prefsStore } from "../../stores/prefs.svelte";
  import type { ColorSwatch, ParagraphStyle, CharacterStyle } from "../../types";

  let { onEditSwatch, onEditParaStyle, onEditCharStyle } = $props();

  function handleAddSwatch() {
    const s = docStore.addSwatch();
    onEditSwatch(s);
  }

  function handleAddParaStyle() {
    const s = docStore.addParagraphStyle();
    onEditParaStyle(s);
  }

  function handleAddCharStyle() {
    const s = docStore.addCharacterStyle();
    onEditCharStyle(s);
  }

  function getSwatchColor(swatchName: string): string {
    const s = docStore.doc.swatches.find(x => x.name === swatchName);
    if (!s) return "transparent";
    if ('Rgb' in s.color) return `rgb(${s.color.Rgb.r*255},${s.color.Rgb.g*255},${s.color.Rgb.b*255})`;
    if ('Cmyk' in s.color) {
      const {c,m,y,k} = s.color.Cmyk;
      return `rgb(${255*(1-c)*(1-k)},${255*(1-m)*(1-k)},${255*(1-y)*(1-k)})`;
    }
    if ('Spot' in s.color) {
      const {alternate_cmyk, tint} = s.color.Spot;
      const [c,m,y,k] = alternate_cmyk;
      return `rgb(${255*(1-c*tint)*(1-k*tint)},${255*(1-m*tint)*(1-k*tint)},${255*(1-y*tint)*(1-k*tint)})`;
    }
    return "gray";
  }

  function convertUnit(val: number, from: string, to: string): number {
    const toPt = { pt: 1, mm: 2.83465, cm: 28.3465, in: 72 };
    return (val * toPt[from as keyof typeof toPt]) / toPt[to as keyof typeof toPt];
  }

  function handlePlaceImage() {
    // This will be handled in the main app for now or moved to a dedicated service
    window.dispatchEvent(new CustomEvent('place-image'));
  }

</script>

<aside class="sidebar-right">
  <div class="panel-header">
    Farbfelder <button class="header-btn" onclick={handleAddSwatch}>+</button>
  </div>
  <div class="swatches-grid">
    {#each docStore.doc.swatches as swatch}
      <div 
        class="swatch-item" 
        class:is-spot={'Spot' in swatch.color} 
        onclick={() => { if (docStore.selectedFrames[0]) { docStore.selectedFrames[0].fill_color = swatch.name; docStore.markModified(); } }} 
        oncontextmenu={(e) => { e.preventDefault(); onEditSwatch(swatch); }}
      >
        <div class="swatch-color" style="background: {getSwatchColor(swatch.name)}"></div>
        <span class="swatch-name">{swatch.name}</span>
      </div>
    {/each}
  </div>

  <div class="panel-header" style="margin-top: 20px;">
    Formate 
    <button class="header-btn" onclick={handleAddParaStyle}>A+</button> 
    <button class="header-btn" onclick={handleAddCharStyle}>Z+</button>
  </div>
  <div class="styles-list">
    {#each docStore.doc.styles.paragraph_styles as s}
      <div class="style-item" onclick={() => onEditParaStyle(s)}>{s.name} (A)</div>
    {/each}
  </div>
  <div class="styles-list">
    {#each docStore.doc.styles.character_styles as s}
      <div class="style-item" onclick={() => onEditCharStyle(s)}>{s.name} (Z)</div>
    {/each}
  </div>

  <div class="panel-header" style="margin-top: 20px;">Eigenschaften</div>
  {#if docStore.selectedFrames.length === 1}
    {@const frame = docStore.selectedFrames[0]}
    <div class="properties">
      {#if frame.data.Text}
        <label>Absatzformat 
          <select bind:value={frame.data.Text.paragraph_style}>
            {#each docStore.doc.styles.paragraph_styles as s}
              <option value={s.name}>{s.name}</option>
            {/each}
          </select>
        </label>
        <label>
          <input type="checkbox" bind:checked={frame.data.Text.align_to_baseline_grid} /> 
          Am Grundraster ausrichten
        </label>
        <textarea bind:value={frame.data.Text.content}></textarea>
      {/if}
      
      {#if frame.data.Image}
        <div class="place-image-action">
          <button onclick={handlePlaceImage}>Bild platzieren...</button>
        </div>
      {/if}
      
      <div class="prop-group">
        <label>X ({prefsStore.prefs.default_unit}) 
          <input 
            type="number" 
            value={convertUnit(frame.x, 'pt', prefsStore.prefs.default_unit)} 
            oninput={(e) => { frame.x = convertUnit(parseFloat((e.target as HTMLInputElement).value), prefsStore.prefs.default_unit, 'pt'); docStore.markModified(); }} 
          />
        </label>
        <label>Y 
          <input 
            type="number" 
            value={convertUnit(frame.y, 'pt', prefsStore.prefs.default_unit)} 
            oninput={(e) => { frame.y = convertUnit(parseFloat((e.target as HTMLInputElement).value), prefsStore.prefs.default_unit, 'pt'); docStore.markModified(); }} 
          />
        </label>
      </div>
      <div class="prop-group">
        <label>W 
          <input 
            type="number" 
            value={convertUnit(frame.width, 'pt', prefsStore.prefs.default_unit)} 
            oninput={(e) => { frame.width = convertUnit(parseFloat((e.target as HTMLInputElement).value), prefsStore.prefs.default_unit, 'pt'); docStore.markModified(); }} 
          />
        </label>
        <label>H 
          <input 
            type="number" 
            value={convertUnit(frame.height, 'pt', prefsStore.prefs.default_unit)} 
            oninput={(e) => { frame.height = convertUnit(parseFloat((e.target as HTMLInputElement).value), prefsStore.prefs.default_unit, 'pt'); docStore.markModified(); }} 
          />
        </label>
      </div>
      <div class="prop-group">
        <label>Kontur <input type="number" bind:value={frame.stroke_width} /></label>
      </div>
    </div>
  {:else if uiStore.selectedFrameIds.length > 1}
    <div class="empty-state">{uiStore.selectedFrameIds.length} Objekte</div>
    <div class="properties">
      <button onclick={() => docStore.alignFrames('Left')}>L-Align</button>
      <button onclick={() => docStore.alignFrames('Top')}>T-Align</button>
    </div>
  {/if}

  <div class="panel-header" style="margin-top: 20px;">Layout</div>
  {#if docStore.activePage}
    {@const page = docStore.activePage}
    <div class="properties">
      <label>Elternseite 
        <select 
          value={page.applied_parent_id ?? ""} 
          onchange={(e) => { page.applied_parent_id = (e.target as HTMLSelectElement).value || undefined; docStore.markModified(); }}
        >
          <option value="">[Keine]</option>
          {#each docStore.doc.parent_pages as p}
            <option value={p.id}>{p.name}</option>
          {/each}
        </select>
      </label>
      <label>Raster-Preset 
        <select onchange={(e) => docStore.applyGridPreset(page, (e.target as HTMLSelectElement).value)}>
          <option value="">[Wählen]</option>
          <option value="TwelveColumn">12 Spalten</option>
          <option value="EightColumn">8 Spalten</option>
          <option value="GoldenRatio">Goldener Schnitt</option>
          <option value="Fibonacci">Fibonacci</option>
          <option value="Manuscript">Manuskript</option>
        </select>
      </label>
      <div class="prop-group">
        <label>Spalten <input type="number" min="1" bind:value={page.column_count} /></label>
        <label>Gutter <input type="number" bind:value={page.gutter_width} /></label>
      </div>
    </div>
  {/if}

  <div class="panel-header" style="margin-top: 20px;">Grundlinienraster</div>
  <div class="properties">
    <label><input type="checkbox" bind:checked={docStore.doc.baseline_grid.visible} /> Anzeigen</label>
    <div class="prop-group">
      <label>Abst. <input type="number" bind:value={docStore.doc.baseline_grid.line_height} /></label>
      <label>Vers. <input type="number" bind:value={docStore.doc.baseline_grid.offset} /></label>
    </div>
  </div>

  <div class="panel-header" style="margin-top: 20px;">Ansicht</div>
  <div class="properties">
    <label>Zoom <input type="range" min="0.1" max="2" step="0.1" bind:value={uiStore.zoom} /></label>
  </div>
</aside>

<style>
  .sidebar-right { width: 240px; background: #252526; display: flex; flex-direction: column; border-left: 1px solid #111; overflow-y: auto; color: #ccc; }
  .panel-header { background: #333; padding: 6px 12px; font-size: 11px; text-transform: uppercase; font-weight: bold; color: #aaa; display: flex; align-items: center; justify-content: space-between; }
  .header-btn { background: transparent; border: none; color: #aaa; cursor: pointer; font-size: 14px; }
  .swatches-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 4px; padding: 8px; }
  .swatch-item { display: flex; align-items: center; gap: 6px; padding: 4px; background: #333; border-radius: 2px; cursor: pointer; font-size: 11px; position: relative; }
  .swatch-item:hover { background: #444; }
  .swatch-item.is-spot::after { content: "•"; position: absolute; top: 2px; right: 4px; color: orange; font-size: 14px; }
  .swatch-color { width: 16px; height: 16px; border: 1px solid #111; }
  .swatch-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .style-item { padding: 4px 12px; border-bottom: 1px solid #333; cursor: pointer; font-size: 12px; }
  .style-item:hover { background: #3d3d3d; }
  .properties { padding: 8px 12px; display: flex; flex-direction: column; gap: 8px; font-size: 12px; }
  .prop-group { display: flex; gap: 8px; }
  .prop-group label { flex: 1; display: flex; flex-direction: column; gap: 4px; }
  .empty-state { padding: 20px; text-align: center; color: #666; font-style: italic; font-size: 12px; }
  input, select, textarea { background: #3c3c3c; border: 1px solid #555; color: white; padding: 4px 8px; border-radius: 2px; }
  textarea { height: 60px; resize: vertical; }
  button { background: #3c3c3c; border: 1px solid #555; color: #ccc; padding: 4px 8px; border-radius: 2px; cursor: pointer; }
  button:hover { background: #4d4d4d; }
</style>
