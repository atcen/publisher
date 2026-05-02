<script lang="ts">
  import { docStore } from "../../stores/document.svelte";
  import { prefsStore } from "../../stores/prefs.svelte";
  import { uiStore } from "../../stores/ui.svelte";

  let titleText = $derived(`${docStore.doc.metadata.name}${docStore.hasUnsavedChanges ? " •" : ""}`);
</script>

<nav class="menu-bar">
  <div class="logo">PUBLISHER</div>
  <div class="menu-items">
    <div class="menu-dropdown">
      <span>Datei</span>
      <div class="dropdown-content">
        <button onclick={() => uiStore.showNewDocModal = true}>Neu...</button>
        <button onclick={() => docStore.open()}>Öffnen...</button>
        <div class="menu-submenu">
          <span>Zuletzt geöffnet</span>
          <div class="submenu-content">
            {#each prefsStore.prefs.recent_files as path}
              <button onclick={() => docStore.openRecent(path)}>
                {path.split(/[/\\]/).pop()}
              </button>
            {/each}
            {#if prefsStore.prefs.recent_files.length > 0}
              <div class="menu-separator"></div>
              <button onclick={() => { prefsStore.prefs.recent_files = []; prefsStore.save(); }}>
                Liste löschen
              </button>
            {/if}
          </div>
        </div>
        <button onclick={() => docStore.save()}>Speichern</button>
        <button onclick={() => docStore.saveAs()}>Speichern unter...</button>
        <button onclick={() => uiStore.showPrefsModal = true}>Einstellungen...</button>
        <button onclick={() => uiStore.showExportModal = true}>Exportieren...</button>
      </div>
    </div>
    <span>Bearbeiten</span>
    <span>Layout</span>
    <span>Ansicht</span>
  </div>
  <div class="doc-title">{titleText}</div>
</nav>

<style>
  .menu-bar { height: 32px; background: #2d2d2d; display: flex; align-items: center; padding: 0 12px; font-size: 13px; border-bottom: 1px solid #111; gap: 20px; color: #ccc; }
  .logo { font-weight: bold; color: #007acc; }
  .menu-items { display: flex; gap: 15px; }
  .menu-dropdown { position: relative; cursor: pointer; }
  .dropdown-content { display: none; position: absolute; top: 100%; left: 0; background: #2d2d2d; border: 1px solid #111; min-width: 150px; z-index: 100; }
  .menu-dropdown:hover .dropdown-content { display: block; }
  .dropdown-content button { width: 100%; background: transparent; border: none; color: #ccc; padding: 8px; text-align: left; cursor: pointer; font-size: 12px; }
  .dropdown-content button:hover { background: #3d3d3d; }
  .menu-submenu { position: relative; padding: 8px; font-size: 12px; }
  .submenu-content { display: none; position: absolute; top: 0; left: 100%; background: #2d2d2d; border: 1px solid #111; min-width: 150px; }
  .menu-submenu:hover .submenu-content { display: block; }
  .menu-separator { height: 1px; background: #444; margin: 4px 0; }
  .doc-title { margin-left: auto; font-style: italic; color: #888; }
</style>
