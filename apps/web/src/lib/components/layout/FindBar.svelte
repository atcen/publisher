<script lang="ts">
  import { docStore } from "../../stores/document.svelte";
  import { uiStore } from "../../stores/ui.svelte";

  let findQuery = $state("");
  let replaceQuery = $state("");
  let useGrep = $state(false);
  let findMatches = $state<{ frameId: string, pageIndex: number }[]>([]);
  let currentMatchIdx = $state(-1);

  function handleFind() {
    const q = findQuery;
    if (q.length === 0) {
      findMatches = [];
      currentMatchIdx = -1;
      return;
    }
    const matches: { frameId: string, pageIndex: number }[] = [];
    let pIdx = 0;
    try {
      const regex = useGrep ? new RegExp(q, 'gi') : null;
      for (const s of docStore.doc.spreads) {
        for (const p of s.pages) {
          for (const f of p.frames) {
            if (f.data.Text) {
              const content = f.data.Text.content;
              const isMatch = regex ? regex.test(content) : content.toLowerCase().includes(q.toLowerCase());
              if (isMatch) matches.push({ frameId: f.id, pageIndex: pIdx });
            }
          }
          pIdx++;
        }
      }
    } catch (e) {
      console.error("Invalid Regex", e);
    }
    findMatches = matches;
    if (matches.length > 0) {
      currentMatchIdx = 0;
      goToMatch(0);
    } else {
      currentMatchIdx = -1;
    }
  }

  function goToMatch(idx: number) {
    const m = findMatches[idx];
    if (m) {
      uiStore.activePageIndex = m.pageIndex;
      uiStore.selectedFrameIds = [m.frameId];
    }
  }

  function handleFindNext() {
    if (findMatches.length > 0) {
      currentMatchIdx = (currentMatchIdx + 1) % findMatches.length;
      goToMatch(currentMatchIdx);
    }
  }

  function handleFindPrev() {
    if (findMatches.length > 0) {
      currentMatchIdx = (currentMatchIdx - 1 + findMatches.length) % findMatches.length;
      goToMatch(currentMatchIdx);
    }
  }

  function handleReplace() {
    if (currentMatchIdx === -1 || findMatches.length === 0) return;
    const m = findMatches[currentMatchIdx];
    for (const s of docStore.doc.spreads) {
      for (const p of s.pages) {
        const f = p.frames.find(x => x.id === m.frameId);
        if (f?.data.Text) {
          docStore.pushToUndo();
          if (useGrep) {
            f.data.Text.content = f.data.Text.content.replace(new RegExp(findQuery, 'gi'), replaceQuery);
          } else {
            f.data.Text.content = f.data.Text.content.split(findQuery).join(replaceQuery);
          }
          docStore.markModified();
          handleFind(); // Refresh matches
          return;
        }
      }
    }
  }

  function handleReplaceAll() {
    docStore.pushToUndo();
    let count = 0;
    for (const s of docStore.doc.spreads) {
      for (const p of s.pages) {
        for (const f of p.frames) {
          if (f.data.Text) {
            const original = f.data.Text.content;
            if (useGrep) {
              f.data.Text.content = f.data.Text.content.replace(new RegExp(findQuery, 'gi'), replaceQuery);
            } else {
              f.data.Text.content = f.data.Text.content.split(findQuery).join(replaceQuery);
            }
            if (f.data.Text.content !== original) count++;
          }
        }
      }
    }
    if (count > 0) {
      docStore.markModified();
      alert(`${count} Änderungen vorgenommen.`);
      handleFind();
    }
  }
</script>

<div class="find-bar" class:expanded={uiStore.showReplaceFields}>
  <div class="find-row">
    <input 
      type="text" 
      placeholder="Suchen..." 
      bind:value={findQuery} 
      oninput={handleFind} 
      onkeydown={(e) => { if (e.key === 'Enter') { if (e.shiftKey) handleFindPrev(); else handleFindNext(); } }} 
      autofocus 
    />
    <span class="find-count">{findMatches.length > 0 ? currentMatchIdx + 1 : 0} / {findMatches.length}</span>
    <button onclick={handleFindPrev}>↑</button>
    <button onclick={handleFindNext}>↓</button>
    <label class="grep-toggle"><input type="checkbox" bind:checked={useGrep} onchange={handleFind} /> GREP</label>
    <button onclick={() => uiStore.showReplaceFields = !uiStore.showReplaceFields}>
      {uiStore.showReplaceFields ? '▲' : '▼'}
    </button>
    <button onclick={() => { uiStore.showFindBar = false; uiStore.showReplaceFields = false; }}>×</button>
  </div>
  
  {#if uiStore.showReplaceFields}
    <div class="replace-row">
      <input type="text" placeholder="Ersetzen durch..." bind:value={replaceQuery} />
      <button onclick={handleReplace}>Ersetzen</button>
      <button onclick={handleReplaceAll}>Alle ersetzen</button>
    </div>
  {/if}
</div>

<style>
  .find-bar { position: absolute; top: 40px; right: 260px; background: #2d2d2d; border: 1px solid #444; padding: 4px 8px; display: flex; flex-direction: column; gap: 8px; border-radius: 4px; box-shadow: 0 4px 12px rgba(0,0,0,0.4); z-index: 100; min-width: 300px; color: #ccc; }
  .find-bar.expanded { width: 400px; }
  .find-row, .replace-row { display: flex; align-items: center; gap: 8px; width: 100%; }
  .replace-row { margin-top: 8px; border-top: 1px solid #444; padding-top: 8px; }
  .find-bar input[type="text"] { flex: 1; height: 24px; font-size: 12px; background: #3c3c3c; border: 1px solid #555; color: white; padding: 0 8px; border-radius: 2px; }
  .find-count { font-size: 10px; color: #888; min-width: 40px; text-align: center; }
  .find-bar button { background: transparent; border: 1px solid #444; color: #ccc; cursor: pointer; padding: 2px 6px; border-radius: 2px; font-size: 11px; }
  .find-bar button:hover { background: #444; }
  .grep-toggle { font-size: 10px; display: flex; align-items: center; gap: 4px; white-space: nowrap; }
</style>
