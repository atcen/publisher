<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  type Pt = number;
...
  let activeTool = $state('select');

  async function handleOpen() {
    try {
      const path = await invoke("open_file");
      console.log("Opened:", path);
      // Here you would load the document from path
    } catch (e) {
      console.error(e);
    }
  }

  async function handleSave() {
    try {
      const path = await invoke("save_file");
      console.log("Saved to:", path);
      // Here you would save the document to path
    } catch (e) {
      console.error(e);
    }
  }

  let selectedFrame = $derived.by(() => {
    for (const spread of doc.spreads) {
      for (const page of spread.pages) {
        for (const frame of page.frames) {
          if (frame.id === selectedFrameId) return frame;
        }
      }
    }
    return null;
  });
</script>

<main>
  <nav class="menu-bar">
    <div class="logo">PUBLISHER</div>
    <div class="menu-items">
      <div class="menu-dropdown">
        <span>Datei</span>
        <div class="dropdown-content">
          <button onclick={handleOpen}>Öffnen...</button>
          <button onclick={handleSave}>Speichern</button>
        </div>
      </div>
      <span>Bearbeiten</span>
      <span>Layout</span>
      <span>Objekt</span>
      <span>Ansicht</span>
    </div>
    <div class="doc-title">{doc.metadata.name}</div>
  </nav>

  <aside class="toolbar">
    <button class:active={activeTool === 'select'} onclick={() => activeTool = 'select'} title="Auswahl (V)">
      <svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M7,2L19,12L13,13.5L16,19L14.5,20L11.5,14.5L7,18V2Z" /></svg>
    </button>
    <button class:active={activeTool === 'text'} onclick={() => activeTool = 'text'} title="Text (T)">
      <svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M13,12H20V13.5H13V12M13,9.5H20V11H13V9.5M13,14.5H20V16H13V14.5M21,2H3A2,2 0 0,0 1,4V20A2,2 0 0,0 3,22H21A2,2 0 0,0 23,20V4A2,2 0 0,0 21,2M21,20H3V4H21V20M8,8H6V11H3V13H6V16H8V13H11V11H8V8Z" /></svg>
    </button>
  </aside>

  <div class="content-area">
    <aside class="sidebar-left">
      <div class="panel-header">Seiten</div>
      <div class="pages-list">
        {#each doc.spreads as spread, i}
          <div class="page-thumb">
            <div class="thumb-box"></div>
            <span>Seite {i + 1}</span>
          </div>
        {/each}
      </div>
    </aside>

    <!-- Klick auf Workspace hebt Auswahl auf -->
    <div class="workspace-container" onclick={() => selectedFrameId = null} role="presentation">
      <div class="workspace" style="--zoom: {zoom}">
        {#each doc.spreads as spread}
          <div class="spread">
            {#each spread.pages as page}
              <div class="page" style="width: {page.width}px; height: {page.height}px;">
                {#each page.frames as frame}
                  {#if frame.Text}
                    <!-- svelte-ignore a11y-click-events-have-key-events -->
                    <!-- svelte-ignore a11y-no-static-element-interactions -->
                    <div 
                      class="frame text-frame"
                      class:selected={selectedFrameId === frame.id}
                      onclick={(e) => { e.stopPropagation(); selectedFrameId = frame.id; }}
                      style="
                        left: {frame.Text.x}px; 
                        top: {frame.Text.y}px; 
                        width: {frame.Text.width}px; 
                        height: {frame.Text.height}px;
                      "
                    >
                      {frame.Text.content}
                    </div>
                  {/if}
                {/each}
              </div>
            {/each}
          </div>
        {/each}
      </div>
    </div>

    <aside class="sidebar-right">
      <div class="panel-header">Eigenschaften</div>
      {#if selectedFrame && selectedFrame.Text}
        <div class="properties">
          <label>
            Inhalt
            <textarea bind:value={selectedFrame.Text.content}></textarea>
          </label>
          <div class="prop-group">
            <label>X <input type="number" bind:value={selectedFrame.Text.x} /></label>
            <label>Y <input type="number" bind:value={selectedFrame.Text.y} /></label>
          </div>
          <div class="prop-group">
            <label>W <input type="number" bind:value={selectedFrame.Text.width} /></label>
            <label>H <input type="number" bind:value={selectedFrame.Text.height} /></label>
          </div>
        </div>
      {:else}
        <div class="empty-state">Kein Objekt ausgewählt</div>
      {/if}

      <div class="panel-header" style="margin-top: 20px;">Ansicht</div>
      <div class="properties">
        <label>
          Zoom
          <input type="range" min="0.1" max="2" step="0.1" bind:value={zoom} />
        </label>
      </div>
    </aside>
  </div>

  <footer class="status-bar">
    <span>Bereit</span>
    <span>A4 (595.27 x 841.89 pt)</span>
    <span>{(zoom * 100).toFixed(0)}%</span>
  </footer>
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background-color: #1e1e1e;
    color: #ccc;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    overflow: hidden;
  }

  main { display: flex; flex-direction: column; height: 100vh; }
  .menu-bar { height: 32px; background: #2d2d2d; display: flex; align-items: center; padding: 0 12px; font-size: 13px; border-bottom: 1px solid #111; gap: 20px; }
  .logo { font-weight: bold; color: #fff; letter-spacing: 1px; }
  .menu-items { display: flex; gap: 15px; }
  .menu-dropdown { position: relative; cursor: pointer; }
  .dropdown-content { display: none; position: absolute; top: 100%; left: 0; background: #2d2d2d; border: 1px solid #111; min-width: 120px; z-index: 100; }
  .menu-dropdown:hover .dropdown-content { display: block; }
  .dropdown-content button { width: 100%; background: transparent; border: none; color: #ccc; padding: 8px 12px; text-align: left; cursor: pointer; font-size: 13px; }
  .dropdown-content button:hover { background: #007acc; color: white; }
  .doc-title { margin-left: auto; color: #888; }
  .toolbar { position: absolute; left: 0; top: 32px; bottom: 25px; width: 40px; background: #2d2d2d; border-right: 1px solid #111; display: flex; flex-direction: column; align-items: center; padding-top: 10px; gap: 5px; z-index: 10; }
  .toolbar button { width: 30px; height: 30px; background: transparent; border: none; color: #ccc; border-radius: 4px; cursor: pointer; display: flex; align-items: center; justify-content: center; }
  .toolbar button.active { background: #007acc; color: white; }
  .content-area { flex: 1; display: flex; margin-left: 40px; overflow: hidden; }
  .sidebar-left, .sidebar-right { width: 240px; background: #252526; display: flex; flex-direction: column; border-right: 1px solid #111; }
  .sidebar-right { border-left: 1px solid #111; border-right: none; }
  .panel-header { background: #333; padding: 6px 12px; font-size: 11px; text-transform: uppercase; font-weight: bold; color: #aaa; }
  .workspace-container { flex: 1; overflow: auto; background: #181818; position: relative; padding: 100px; }
  .workspace { display: flex; flex-direction: column; align-items: center; gap: 50px; }
  .spread { display: flex; gap: 2px; background: #000; padding: 2px; box-shadow: 0 20px 50px rgba(0,0,0,0.6); transform: scale(var(--zoom)); transform-origin: top center; transition: transform 0.1s ease-out; }
  .page { background: white; position: relative; color: black; }
  .frame { position: absolute; cursor: default; user-select: none; }
  .text-frame { border: 1px solid transparent; padding: 4px; font-size: 14px; line-height: 1.4; overflow: hidden; }
  .text-frame.selected { border: 1px solid #007acc; box-shadow: 0 0 0 1px #007acc; }
  .properties { padding: 15px; display: flex; flex-direction: column; gap: 15px; }
  .properties label { display: flex; flex-direction: column; gap: 5px; font-size: 12px; }
  .properties input, .properties textarea { background: #3c3c3c; border: 1px solid #555; color: white; padding: 4px 8px; border-radius: 2px; }
  .properties textarea { height: 80px; resize: none; }
  .prop-group { display: flex; gap: 10px; }
  .prop-group label { flex: 1; }
  .page-thumb { padding: 15px; display: flex; flex-direction: column; align-items: center; gap: 8px; }
  .thumb-box { width: 60px; height: 80px; background: #444; border: 1px solid #555; }
  .empty-state { padding: 40px; text-align: center; color: #666; font-style: italic; font-size: 13px; }
  .status-bar { height: 25px; background: #007acc; color: white; display: flex; align-items: center; padding: 0 10px; font-size: 12px; gap: 20px; }
</style>
