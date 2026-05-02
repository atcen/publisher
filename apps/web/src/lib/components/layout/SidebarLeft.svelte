<script lang="ts">
  import { docStore } from "../../stores/document.svelte";
  import { uiStore } from "../../stores/ui.svelte";

  function handleLayerDrop(idx: number) {
    if (uiStore.draggedLayerIndex !== null && uiStore.draggedLayerIndex !== idx) {
      docStore.moveLayer(uiStore.draggedLayerIndex, idx);
    }
    uiStore.draggedLayerIndex = null;
  }

  function handlePageDrop(idx: number) {
    if (uiStore.draggedPageIndex !== null && uiStore.draggedPageIndex !== idx) {
      docStore.movePage(uiStore.draggedPageIndex, idx);
    }
    uiStore.draggedPageIndex = null;
  }
</script>

<aside class="sidebar-left">
  <div class="panel-header">
    Elternseiten 
    <button class="header-btn" onclick={() => docStore.addParentPage()}>+</button>
  </div>
  <div class="pages-list">
    {#each docStore.doc.parent_pages as parent}
      <div class="page-thumb">
        <div class="thumb-box parent"></div>
        <div class="parent-meta">
          <input 
            class="parent-name-input" 
            type="text" 
            value={parent.name} 
            onchange={(e) => docStore.renameParentPage(parent.id, (e.target as HTMLInputElement).value)} 
          />
          <select 
            class="parent-based-on" 
            bind:value={parent.based_on_id} 
            onchange={() => docStore.markModified()}
          >
            <option value={undefined}>[Keine]</option>
            {#each docStore.doc.parent_pages.filter(p => p.id !== parent.id) as op}
              <option value={op.id}>Basis: {op.name}</option>
            {/each}
          </select>
        </div>
        <button class="delete-btn" onclick={() => docStore.deleteParentPage(parent.id)}>×</button>
      </div>
    {/each}
  </div>

  <div class="panel-header" style="margin-top: 20px;">
    Seiten 
    <button class="header-btn" onclick={() => docStore.addPage()}>+</button>
  </div>
  <div class="pages-list">
    {#each docStore.doc.spreads.flatMap(s => s.pages) as page, index}
      <div 
        class="page-thumb" 
        class:active={uiStore.activePageIndex === index} 
        onclick={() => uiStore.activePageIndex = index} 
        draggable="true" 
        ondragstart={() => uiStore.draggedPageIndex = index} 
        ondragover={(e) => { e.preventDefault(); return false; }} 
        ondrop={() => handlePageDrop(index)}
      >
        <div class="thumb-box"></div>
        <span>Seite {index + 1}</span>
        <button class="delete-btn" onclick={() => docStore.deletePage(index)}>×</button>
      </div>
    {/each}
  </div>

  <div class="panel-header" style="margin-top: 20px;">
    Ebenen 
    <button class="header-btn" onclick={() => docStore.addLayer()}>+</button>
  </div>
  <div class="layers-list">
    {#each docStore.doc.layers as layer, i}
      <div 
        class="layer-item" 
        draggable="true" 
        ondragstart={() => uiStore.draggedLayerIndex = i} 
        ondragover={(e) => { e.preventDefault(); return false; }} 
        ondrop={() => handleLayerDrop(i)}
      >
        <input type="checkbox" bind:checked={layer.visible} /> 
        <input type="checkbox" bind:checked={layer.locked} /> 
        <div class="layer-color" style="background: {layer.color}"></div> 
        <input class="layer-name-input" type="text" bind:value={layer.name} />
      </div>
    {/each}
  </div>
</aside>

<style>
  .sidebar-left { width: 240px; background: #252526; display: flex; flex-direction: column; border-right: 1px solid #111; color: #ccc; }
  .panel-header { background: #333; padding: 6px 12px; font-size: 11px; text-transform: uppercase; font-weight: bold; color: #aaa; display: flex; align-items: center; justify-content: space-between; }
  .header-btn { background: transparent; border: none; color: #aaa; cursor: pointer; font-size: 14px; }
  .pages-list { display: flex; flex-direction: column; }
  .page-thumb { padding: 10px; display: flex; flex-direction: column; align-items: center; gap: 5px; position: relative; cursor: pointer; border-bottom: 1px solid #333; }
  .page-thumb.active { background: #007acc22; border-left: 2px solid #007acc; }
  .thumb-box { width: 50px; height: 70px; background: #444; border: 1px solid #555; }
  .thumb-box.parent { border-color: #007acc; background: #2d2d2d; }
  .parent-meta { flex: 1; display: flex; flex-direction: column; gap: 4px; overflow: hidden; }
  .parent-based-on { background: transparent; border: 1px solid #444; color: #888; font-size: 9px; padding: 2px; }
  .delete-btn { position: absolute; top: 2px; right: 2px; background: #c43; color: white; border: none; border-radius: 50%; width: 14px; height: 14px; font-size: 10px; cursor: pointer; display: none; }
  .page-thumb:hover .delete-btn { display: flex; align-items: center; justify-content: center; }
  .layers-list { display: flex; flex-direction: column; }
  .layer-item { display: flex; align-items: center; padding: 4px 12px; gap: 8px; border-bottom: 1px solid #333; cursor: grab; font-size: 12px; }
  .layer-color { width: 12px; height: 12px; border-radius: 2px; }
  .layer-name-input, .parent-name-input { background: transparent; border: none; color: #eee; flex: 1; padding: 2px; font-size: 11px; }
  .parent-name-input { width: 100px; border-bottom: 1px solid transparent; }
  .parent-name-input:focus { border-bottom-color: #007acc; outline: none; }
</style>
