<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  type Pt = number;
  interface TextFrame { content: string; paragraph_style?: string; next_frame_id?: string; prev_frame_id?: string; align_to_baseline_grid: boolean; }
  interface ImageFrame { asset_path: string; content_x: Pt; content_y: Pt; content_scale_x: number; content_scale_y: number; fitting: 'Fill' | 'Fit' | 'Stretch' | 'Original' | 'Custom'; }
  interface Frame { id: string; layer_id: string; x: Pt; y: Pt; width: Pt; height: Pt; rotation: number; data: { Text?: TextFrame; Image?: ImageFrame; Shape?: { shape_type: string }; Group?: { frames: Frame[] }; }; }
  interface Layer { id: string; name: string; visible: boolean; locked: boolean; color: string; }
  interface Page { width: Pt; height: Pt; margins: { top: number; bottom: number; inside: number; outside: number }; column_count: number; gutter_width: number; guides: Guide[]; frames: Frame[]; }
  interface Guide { position: Pt; orientation: 'Horizontal' | 'Vertical'; locked: boolean; color: string | null; }
  interface Spread { pages: Page[]; }
  interface DocumentMetadata { name: string; author: string; description: string; created_at: number; modified_at: number; dpi: number; default_unit: "Point"; default_bleed: { top: number; bottom: number; inside: number; outside: number }; color_profile: string; facing_pages: boolean; }
  interface ParagraphStyle { name: string; based_on?: string; font_family?: string; font_size?: Pt; alignment?: 'Left' | 'Center' | 'Right' | 'Justify'; }
  interface CharacterStyle { name: string; based_on?: string; font_family?: string; font_style?: string; font_size?: Pt; }
  interface Styles { paragraph_styles: ParagraphStyle[]; character_styles: CharacterStyle[]; object_styles: unknown[]; }
  interface BaselineGrid { line_height: Pt; offset: Pt; visible: boolean; color: string; }
  interface Document { metadata: DocumentMetadata; fonts: unknown[]; icc_profiles: unknown[]; swatches: unknown[]; styles: Styles; spreads: Spread[]; layers: Layer[]; baseline_grid: BaselineGrid; }

  let activeTool = $state('select'); let zoom = $state(1); let activePageIndex = $state(0); let selectedFrameIds = $state<string[]>([]);
  let currentFilePath = $state<string | null>(null); let hasUnsavedChanges = $state(false); let undoStack = $state<string[]>([]); let redoStack = $state<string[]>([]);
  let showNewDocModal = $state(false); let showStyleEditorModal = $state(false); let showCharStyleEditorModal = $state(false); let showExportModal = $state(false);
  let showFindBar = $state(false); let findQuery = $state(""); let findMatches = $state<{ frameId: string, pageIndex: number }[]>([]); let currentMatchIdx = $state(-1);
  let currentEditingStyle = $state<ParagraphStyle | null>(null); let currentEditingCharStyle = $state<CharacterStyle | null>(null);
  let isLinking = $state(false); let linkingSourceFrameId = $state<string | null>(null);
  let showGuides = $state(true); let guidesLocked = $state(false); let isDraggingGuide = $state(false); let currentDraggingGuide = $state<{ page: Page, guide: Guide } | null>(null);
  let exportSettings = $state({ format: 'pdf-x4', pageRange: 'all', customRange: '', includeBleed: true, embedFonts: true, compression: 'high' });
  let snapX = $state<number | null>(null); let snapY = $state<number | null>(null);
  let draggedLayerIndex = $state<number | null>(null); let draggedPageIndex = $state<number | null>(null);

  let doc = $state<Document>({
    metadata: { name: "Untitled", author: "", description: "", created_at: 0, modified_at: 0, dpi: 72, default_unit: "Point", default_bleed: { top: 0, bottom: 0, inside: 0, outside: 0 }, color_profile: "sRGB", facing_pages: true },
    fonts: [], icc_profiles: [], swatches: [], styles: { paragraph_styles: [{ name: "Standard", font_family: "Inter", font_size: 12 }], character_styles: [{ name: "[Keines]" }], object_styles: [] },
    spreads: [{ pages: [{ width: 595.27, height: 841.89, margins: { top: 36, bottom: 36, inside: 36, outside: 36 }, column_count: 2, gutter_width: 12, guides: [], frames: [] }] }],
    layers: [{ id: "layer-1", name: "Ebene 1", visible: true, locked: false, color: "#007acc" }],
    baseline_grid: { line_height: 12, offset: 0, visible: false, color: "#44ffff33" }
  });

  function pushToUndo() { undoStack.push(JSON.stringify(doc)); if (undoStack.length > 50) undoStack.shift(); redoStack = []; }
  function handleUndo() { if (undoStack.length > 0) { redoStack.push(JSON.stringify(doc)); doc = JSON.parse(undoStack.pop()!); hasUnsavedChanges = true; } }
  function handleRedo() { if (redoStack.length > 0) { undoStack.push(JSON.stringify(doc)); doc = JSON.parse(redoStack.pop()!); hasUnsavedChanges = true; } }
  function markDocumentAsChanged() { hasUnsavedChanges = true; }

  function handleFind() {
    const q = findQuery.toLowerCase(); const matches: { frameId: string, pageIndex: number }[] = []; if (q.length === 0) { findMatches = []; currentMatchIdx = -1; return; }
    let pIdx = 0; for (const s of doc.spreads) for (const p of s.pages) { for (const f of p.frames) if (f.data.Text && f.data.Text.content.toLowerCase().includes(q)) matches.push({ frameId: f.id, pageIndex: pIdx }); pIdx++; }
    findMatches = matches; if (matches.length > 0) { currentMatchIdx = 0; goToMatch(0); } else currentMatchIdx = -1;
  }
  function goToMatch(idx: number) { const m = findMatches[idx]; if (m) { activePageIndex = m.pageIndex; selectedFrameIds = [m.frameId]; } }
  function handleFindNext() { if (findMatches.length > 0) { currentMatchIdx = (currentMatchIdx + 1) % findMatches.length; goToMatch(currentMatchIdx); } }
  function handleFindPrev() { if (findMatches.length > 0) { currentMatchIdx = (currentMatchIdx - 1 + findMatches.length) % findMatches.length; goToMatch(currentMatchIdx); } }

  function reorganizeSpreads() {
    const allPages: Page[] = doc.spreads.flatMap(s => s.pages);
    const newSpreads: Spread[] = []; let current: Page[] = [];
    for (let i = 0; i < allPages.length; i++) {
      if (doc.metadata.facing_pages) { if (i === 0) newSpreads.push({ pages: [allPages[i]] }); else { current.push(allPages[i]); if (current.length === 2) { newSpreads.push({ pages: current }); current = []; } } }
      else { newSpreads.push({ pages: [allPages[i]] }); }
    }
    if (current.length > 0) newSpreads.push({ pages: current });
    doc.spreads = newSpreads; markDocumentAsChanged();
  }

  function handleAddPage() { pushToUndo(); doc.spreads.push({ pages: [{ width: 595.27, height: 841.89, margins: { top: 36, bottom: 36, inside: 36, outside: 36 }, column_count: 1, gutter_width: 12, guides: [], frames: [] }] }); reorganizeSpreads(); }
  function handleDeletePage(idx: number) { if (confirm("Löschen?")) { pushToUndo(); const pages = doc.spreads.flatMap(s => s.pages); if (pages.length > 1) { pages.splice(idx, 1); doc.spreads = pages.map(p => ({ pages: [p] })); reorganizeSpreads(); } } }
  function handleAddStyle() { const s: ParagraphStyle = { name: `Stil ${doc.styles.paragraph_styles.length + 1}` }; doc.styles.paragraph_styles.push(s); currentEditingStyle = s; showStyleEditorModal = true; markDocumentAsChanged(); }
  function handleAddCharStyle() { const s: CharacterStyle = { name: `Zeichen ${doc.styles.character_styles.length + 1}` }; doc.styles.character_styles.push(s); currentEditingCharStyle = s; showCharStyleEditorModal = true; markDocumentAsChanged(); }
  function handleAddLayer() { pushToUndo(); doc.layers.unshift({ id: crypto.randomUUID(), name: `Ebene ${doc.layers.length + 1}`, visible: true, locked: false, color: '#007acc' }); markDocumentAsChanged(); }
  
  async function handleOpen() { try { const path = await invoke<string>("open_file"); const content = await invoke<string>("read_document", { filePath: path }); doc = JSON.parse(content); currentFilePath = path; hasUnsavedChanges = false; } catch (e) { alert("Failed to open file: " + e); } }
  async function handleSave() { try { if (!currentFilePath) { await handleSaveAs(); return; } doc.metadata.modified_at = Math.floor(Date.now() / 1000); const documentJson = JSON.stringify(doc, null, 2); await invoke<string>("save_document", { filePath: currentFilePath, documentJson }); hasUnsavedChanges = false; } catch (e) { alert("Failed to save file: " + e); } }
  async function handleSaveAs() { try { doc.metadata.modified_at = Math.floor(Date.now() / 1000); const documentJson = JSON.stringify(doc, null, 2); const path = await invoke<string>("save_as_file", { documentJson }); currentFilePath = path; hasUnsavedChanges = false; } catch (e) { alert("Failed to save file: " + e); } }

  function handleCreateDocument() {
    pushToUndo(); const pages: Page[] = Array.from({length: newDocSettings.pages}, () => ({ width: newDocSettings.width, height: newDocSettings.height, margins: {...newDocSettings.margins}, column_count: newDocSettings.columns, gutter_width: newDocSettings.gutter, guides: [], frames: [] }));
    doc = { ...doc, metadata: { ...doc.metadata, name: newDocSettings.name, facing_pages: newDocSettings.facingPages }, spreads: pages.map(p => ({ pages: [p] })) };
    reorganizeSpreads(); showNewDocModal = false; currentFilePath = null; hasUnsavedChanges = false;
  }

  let activePage = $derived.by((): Page | null => { let c = 0; for (const s of doc.spreads) for (const p of s.pages) { if (c === activePageIndex) return p; c++; } return null; });
  let selectedFrames = $derived.by((): Frame[] => { const res: Frame[] = []; for (const s of doc.spreads) for (const p of s.pages) for (const f of p.frames) if (selectedFrameIds.includes(f.id)) res.push(f); return res; });
  let selectedFrame = $derived(selectedFrames[0] || null);
  let titleText = $derived(`${doc.metadata.name}${hasUnsavedChanges ? " •" : ""}`);

  async function handleCopy() { if (selectedFrameIds.length > 0) await navigator.clipboard.writeText(JSON.stringify({ type: 'publisher-frames', frames: selectedFrames })); }
  async function handleCut() { if (selectedFrameIds.length > 0) { await handleCopy(); pushToUndo(); for (const s of doc.spreads) for (const p of s.pages) p.frames = p.frames.filter(f => !selectedFrameIds.includes(f.id)); selectedFrameIds = []; markDocumentAsChanged(); } }
  async function handlePaste() { try { const data = JSON.parse(await navigator.clipboard.readText()); if (data.type === 'publisher-frames' && activePage) { pushToUndo(); const newIds = data.frames.map((f: any) => { const nf = {...f, id: crypto.randomUUID(), x: f.x + 20, y: f.y + 20}; activePage!.frames.push(nf); return nf.id; }); selectedFrameIds = newIds; markDocumentAsChanged(); } } catch {} }
  function handleDuplicate() { if (selectedFrameIds.length > 0 && activePage) { pushToUndo(); const newIds = selectedFrames.map(f => { const nf = JSON.parse(JSON.stringify(f)); nf.id = crypto.randomUUID(); nf.x += 10; nf.y += 10; activePage!.frames.push(nf); return nf.id; }); selectedFrameIds = newIds; markDocumentAsChanged(); } }
  function handleGroup() { if (selectedFrameIds.length > 1 && activePage) { pushToUndo(); const selected = activePage.frames.filter(f => selectedFrameIds.includes(f.id)); const remaining = activePage.frames.filter(f => !selectedFrameIds.includes(f.id)); const minX = Math.min(...selected.map(f => f.x)); const minY = Math.min(...selected.map(f => f.y)); const maxX = Math.max(...selected.map(f => f.x + f.width)); const maxY = Math.max(...selected.map(f => f.y + f.height)); selected.forEach(f => { f.x -= minX; f.y -= minY; }); const group: Frame = { id: crypto.randomUUID(), layer_id: selected[0].layer_id, x: minX, y: minY, width: maxX-minX, height: maxY-minY, rotation: 0, data: { Group: { frames: selected } } }; activePage.frames = [...remaining, group]; selectedFrameIds = [group.id]; markDocumentAsChanged(); } }
  function handleUngroup() { if (selectedFrameIds.length === 1 && activePage) { const f = activePage.frames.find(f => f.id === selectedFrameIds[0]); if (f?.data.Group) { pushToUndo(); const children = f.data.Group.frames; children.forEach(c => { c.x += f!.x; c.y += f!.y; c.layer_id = f!.layer_id; }); activePage.frames = activePage.frames.filter(x => x.id !== f!.id).concat(children); selectedFrameIds = children.map(c => c.id); markDocumentAsChanged(); } } }
  async function handleExport() { console.log("Export...", exportSettings); alert("Export gestartet"); showExportModal = false; }

  let isDragging = false; let isResizing = false; let isCreating = false; let dragStart = { x: 0, y: 0 }; let initial = { x: 0, y: 0, w: 0, h: 0 }; let resizeHandleIdx = ""; let currentCreating: Frame | null = null;
  function handleFrameMouseDown(e: MouseEvent, frame: Frame) {
    if (isLinking && linkingSourceFrameId && frame.id !== linkingSourceFrameId && frame.data.Text) { pushToUndo(); const source = selectedFrames.find(f => f.id === linkingSourceFrameId); if (source?.data.Text) { source.data.Text.next_frame_id = frame.id; frame.data.Text.prev_frame_id = source.id; markDocumentAsChanged(); } isLinking = false; linkingSourceFrameId = null; return; }
    const l = doc.layers.find(l => l.id === frame.layer_id); if (l?.locked || !l?.visible || activeTool !== 'select') return;
    e.stopPropagation(); if (e.shiftKey) { if (selectedFrameIds.includes(frame.id)) selectedFrameIds = selectedFrameIds.filter(id => id !== frame.id); else selectedFrameIds.push(frame.id); } else if (!selectedFrameIds.includes(frame.id)) selectedFrameIds = [frame.id];
    isDragging = true; dragStart = { x: e.clientX, y: e.clientY }; initial = { x: frame.x, y: frame.y, w: frame.width, h: frame.height };
  }
  function handlePageMouseDown(e: MouseEvent, page: Page) {
    if (activeTool === 'select') { selectedFrameIds = []; return; }
    const targetLayer = doc.layers.find(l => !l.locked && l.visible); if (!targetLayer) return;
    e.stopPropagation(); const r = (e.currentTarget as HTMLElement).getBoundingClientRect(); const x = (e.clientX - r.left)/zoom; const y = (e.clientY - r.top)/zoom;
    const nf: Frame = { id: crypto.randomUUID(), layer_id: targetLayer.id, x, y, width: 0, height: 0, rotation: 0, data: activeTool === 'text' ? { Text: { content: "", align_to_baseline_grid: false } } : { Image: { asset_path: "", content_x: 0, content_y: 0, content_scale_x: 1, content_scale_y: 1, fitting: 'Fit' } } };
    page.frames.push(nf); selectedFrameIds = [nf.id]; currentCreating = nf; isCreating = true; dragStart = { x: e.clientX, y: e.clientY };
  }
  
  function getSnapTargets(page: Page): number[] {
    const ts: number[] = [0, page.width, page.height, page.margins.top, page.height - page.margins.bottom, page.margins.inside, page.width - page.margins.outside];
    page.guides.forEach(g => ts.push(g.position));
    page.frames.filter(f => !selectedFrameIds.includes(f.id)).forEach(f => { ts.push(f.x, f.x+f.width/2, f.x+f.width, f.y, f.y+f.height/2, f.y+f.height); });
    if (doc.baseline_grid.visible) for (let y = doc.baseline_grid.offset; y < page.height; y += doc.baseline_grid.line_height) ts.push(y);
    return [...new Set(ts)];
  }

  function handleMouseMove(e: MouseEvent) {
    const dx = (e.clientX - dragStart.x)/zoom; const dy = (e.clientY - dragStart.y)/zoom;
    if (isDragging && selectedFrame && activePage) {
      let nx = initial.x + dx; let ny = initial.y + dy; snapX = snapY = null;
      if (!e.altKey) {
        const ts = getSnapTargets(activePage); const threshold = 5/zoom;
        const edgesX = [nx, nx+initial.w/2, nx+initial.w]; const edgesY = [ny, ny+initial.h/2, ny+initial.h];
        for (const t of ts) {
          for (const ex of edgesX) if (Math.abs(ex-t) < threshold) { nx -= (ex-t); snapX = t; break; }
          for (const ey of edgesY) if (Math.abs(ey-t) < threshold) { ny -= (ey-t); snapY = t; break; }
        }
      }
      selectedFrame.x = nx; selectedFrame.y = ny; markDocumentAsChanged();
    }
    else if (isResizing && selectedFrame) {
      if (resizeHandleIdx.includes('e')) selectedFrame.width = Math.max(10, initial.w + dx);
      if (resizeHandleIdx.includes('s')) selectedFrame.height = Math.max(10, initial.h + dy);
      if (resizeHandleIdx.includes('w')) { const nw = Math.max(10, initial.w - dx); selectedFrame.x = initial.x + (initial.w - nw); selectedFrame.width = nw; }
      if (resizeHandleIdx.includes('n')) { const nh = Math.max(10, initial.h - dy); selectedFrame.y = initial.y + (initial.h - nh); selectedFrame.height = nh; }
      markDocumentAsChanged();
    } else if (isCreating && currentCreating) { currentCreating.width = Math.max(0, dx); currentCreating.height = Math.max(0, dy); markDocumentAsChanged(); }
    else if (isDraggingGuide && currentDraggingGuide) { const r = document.querySelector('.workspace')?.getBoundingClientRect(); if (r) { const x = (e.clientX - r.left)/zoom; const y = (e.clientY - r.top)/zoom; if (currentDraggingGuide.guide.orientation === 'Horizontal') currentDraggingGuide.guide.position = y; else currentDraggingGuide.guide.position = x; markDocumentAsChanged(); } }
  }
  function handleMouseUp() { if (isCreating && currentCreating && (currentCreating.width < 5 || currentCreating.height < 5)) { for (const s of doc.spreads) for (const p of s.pages) p.frames = p.frames.filter(f => f.id !== currentCreating!.id); selectedFrameIds = []; } isDragging = isResizing = isCreating = isDraggingGuide = false; snapX = snapY = null; currentCreating = null; currentDraggingGuide = null; }
  
  function handleAlign(m: string) { if (selectedFrameIds.length < 2 || !activePage) return; pushToUndo(); const fs = activePage.frames.filter(f => selectedFrameIds.includes(f.id)); if (m === 'left') { const x = Math.min(...fs.map(f => f.x)); fs.forEach(f => f.x = x); } else if (m === 'top') { const y = Math.min(...fs.map(f => f.y)); fs.forEach(f => f.y = y); } markDocumentAsChanged(); }
  function handlePortMouseDown(e: MouseEvent, id: string) { e.stopPropagation(); isLinking = true; linkingSourceFrameId = id; }
  function handleRulerMouseDown(e: MouseEvent, o: 'Horizontal' | 'Vertical') { e.stopPropagation(); pushToUndo(); const g: Guide = { position: 0, orientation: o, locked: false, color: null }; if (activePage) { activePage.guides.push(g); isDraggingGuide = true; currentDraggingGuide = { page: activePage, guide: g }; markDocumentAsChanged(); } }
  function handleGuideMouseDown(e: MouseEvent, page: Page, guide: Guide) { if (!guidesLocked) { e.stopPropagation(); isDraggingGuide = true; currentDraggingGuide = { page, guide }; } }
  function handleLayerDrop(idx: number) { if (draggedLayerIndex !== null && draggedLayerIndex !== idx) { pushToUndo(); const [l] = doc.layers.splice(draggedLayerIndex, 1); doc.layers.splice(idx, 0, l); markDocumentAsChanged(); } draggedLayerIndex = null; }
  function handlePageDrop(idx: number) { if (draggedPageIndex !== null && draggedPageIndex !== idx) { pushToUndo(); const pages = doc.spreads.flatMap(s => s.pages); const [p] = pages.splice(draggedPageIndex, 1); pages.splice(idx, 0, p); doc.spreads = pages.map(p => ({ pages: [p] })); reorganizeSpreads(); } draggedPageIndex = null; }

  function handleKeyDown(e: KeyboardEvent) {
    const k = e.key.toLowerCase(); const cmd = navigator.platform.includes("MAC") ? e.metaKey : e.ctrlKey;
    if (cmd && k === 's') { e.preventDefault(); if (e.shiftKey) handleSaveAs(); else handleSave(); }
    else if (cmd && k === 'n') { e.preventDefault(); showNewDocModal = true; }
    else if (cmd && k === 'f') { e.preventDefault(); showFindBar = true; }
    else if (cmd && k === 'z') { e.preventDefault(); if (e.shiftKey) handleRedo(); else handleUndo(); }
    else if (cmd && k === 'y') { e.preventDefault(); handleRedo(); }
    else if (cmd && k === 'c') handleCopy(); else if (cmd && k === 'x') handleCut(); else if (cmd && k === 'v') handlePaste(); else if (cmd && k === 'd') { e.preventDefault(); handleDuplicate(); }
    else if (cmd && k === 'g') { e.preventDefault(); if (e.shiftKey) handleUngroup(); else handleGroup(); }
    else if (e.key === 'Escape') { if (showFindBar) showFindBar = false; else selectedFrameIds = []; }
    else if ((e.key === 'Delete' || e.key === 'Backspace') && selectedFrameIds.length > 0 && !['INPUT','TEXTAREA'].includes(document.activeElement?.tagName || '')) { pushToUndo(); for (const s of doc.spreads) for (const p of s.pages) p.frames = p.frames.filter(f => !selectedFrameIds.includes(f.id)); selectedFrameIds = []; markDocumentAsChanged(); }
  }

  const DIN_A4 = { w: 595.27, h: 841.89 };
  let newDocSettings = $state({ name: "Dokument 1", width: DIN_A4.w, height: DIN_A4.h, pages: 1, facingPages: true, columns: 2, gutter: 12, margins: { top: 36, bottom: 36, inside: 36, outside: 36 }, bleed: 0 });

  async function handlePlaceImage() { if (selectedFrameIds.length !== 1 || !selectedFrame?.data.Image) return; try { const path = await invoke<string>("open_file"); if (path) { selectedFrame.data.Image.asset_path = path; markDocumentAsChanged(); } } catch (e) { alert("Failed to place image: " + e); } }
</script>

<svelte:window on:keydown={handleKeyDown} on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />
<main>
  <nav class="menu-bar">
    <div class="logo">PUBLISHER</div>
    <div class="menu-items"><div class="menu-dropdown"><span>Datei</span><div class="dropdown-content"><button onclick={() => showNewDocModal = true}>Neu...</button><button onclick={handleOpen}>Öffnen...</button><button onclick={handleSave}>Speichern</button><button onclick={handleSaveAs}>Speichern unter...</button><button onclick={() => showExportModal = true}>Exportieren...</button></div></div><span>Bearbeiten</span><span>Layout</span><span>Ansicht</span></div>
    <div class="doc-title">{titleText}</div>
  </nav>
  <aside class="toolbar">
    <button class:active={activeTool === 'select'} onclick={() => activeTool = 'select'}>V</button>
    <button class:active={activeTool === 'text'} onclick={() => activeTool = 'text'}>T</button>
    <button class:active={activeTool === 'image'} onclick={() => activeTool = 'image'}>F</button>
  </aside>
  <div class="content-area">
    <aside class="sidebar-left">
      <div class="panel-header">Seiten <button class="header-btn" onclick={handleAddPage}>+</button></div>
      <div class="pages-list">
        {#each doc.spreads.flatMap(s => s.pages) as page, index}
          <div class="page-thumb" class:active={activePageIndex === index} onclick={() => activePageIndex = index} draggable="true" ondragstart={() => draggedPageIndex = index} ondragover={(e) => { e.preventDefault(); return false; }} ondrop={() => handlePageDrop(index)}>
            <div class="thumb-box"></div><span>Seite {index + 1}</span><button class="delete-btn" onclick={() => handleDeletePage(index)}>×</button>
          </div>
        {/each}
      </div>
      <div class="panel-header" style="margin-top: 20px;">Ebenen <button class="header-btn" onclick={handleAddLayer}>+</button></div>
      <div class="layers-list">{#each doc.layers as layer, i}<div class="layer-item" draggable="true" ondragstart={() => draggedLayerIndex = i} ondragover={(e) => { e.preventDefault(); return false; }} ondrop={() => handleLayerDrop(i)}><input type="checkbox" bind:checked={layer.visible} /> <input type="checkbox" bind:checked={layer.locked} /> <div class="layer-color" style="background: {layer.color}"></div> <input class="layer-name-input" type="text" bind:value={layer.name} /></div>{/each}</div>
    </aside>
    <div class="workspace-container" onclick={() => selectedFrameIds = []}>
      <div class="ruler top-ruler" onmousedown={(e) => handleRulerMouseDown(e, 'Vertical')}>{#each Array(20) as _, i}<div class="ruler-tick" style="left: {i * 100 * zoom}px">{i * 100}</div>{/each}</div>
      <div class="ruler left-ruler" onmousedown={(e) => handleRulerMouseDown(e, 'Horizontal')}>{#each Array(20) as _, i}<div class="ruler-tick" style="top: {i * 100 * zoom}px">{i * 100}</div>{/each}</div>
      <div class="workspace" style="--zoom: {zoom}">
        {#each doc.spreads as spread}<div class="spread">
          {#each spread.pages as page}
            <div class="page" style="width: {page.width}px; height: {page.height}px;" onmousedown={(e) => handlePageMouseDown(e, page)}>
              {#if snapX !== null}<div class="snap-guide vertical" style="left: {snapX}px"></div>{/if}
              {#if snapY !== null}<div class="snap-guide horizontal" style="top: {snapY}px"></div>{/if}
              <div class="margin-box" style="top: {page.margins.top}px; bottom: {page.margins.bottom}px; left: {page.margins.inside}px; right: {page.margins.outside}px;">
                {#if page.column_count > 1}<div class="column-gutters">{#each Array(page.column_count - 1) as _, i}{@const colW = (page.width - page.margins.inside - page.margins.outside - (page.column_count - 1) * page.gutter_width) / page.column_count}<div class="gutter-guide" style="left: {(i + 1) * colW + i * page.gutter_width}px; width: {page.gutter_width}px;"></div>{/each}</div>{/if}
              </div>
              {#if doc.baseline_grid.visible}<div class="baseline-grid">{#each Array(Math.floor(page.height / doc.baseline_grid.line_height)) as _, i}<div class="baseline-line" style="top: {doc.baseline_grid.offset + i * doc.baseline_grid.line_height}px"></div>{/each}</div>{/if}
              {#if showGuides}{#each page.guides as guide}<div class="guide" class:horizontal={guide.orientation === 'Horizontal'} class:vertical={guide.orientation === 'Vertical'} style="{guide.orientation === 'Horizontal' ? 'top' : 'left'}: {guide.position}px;" onmousedown={(e) => handleGuideMouseDown(e, page, guide)}></div>{/each}{/if}
              {#each [...doc.layers].reverse() as layer}{#if layer.visible}{#each page.frames as frame}{#if frame.layer_id === layer.id}
                <div class="frame" class:text-frame={frame.data.Text} class:image-frame={frame.data.Image} class:selected={selectedFrameIds.includes(frame.id)} onmousedown={(e) => handleFrameMouseDown(e, frame)} style="left: {frame.x}px; top: {frame.y}px; width: {frame.width}px; height: {frame.height}px; transform: rotate({frame.rotation}deg); --layer-color: {layer.color}">
                  {#if frame.data.Text}{frame.data.Text.content}{:else if frame.data.Image}<div class="image-placeholder">{#if frame.data.Image.asset_path}Bild{:else}Kein Bild{/if}</div>
                  {:else if frame.data.Group}<div class="group-content">{#each frame.data.Group.frames as c}<div class="frame-preview" style="left: {c.x}px; top: {c.y}px; width: {c.width}px; height: {c.height}px;"></div>{/each}</div>{/if}
                  {#if selectedFrameIds.length === 1 && selectedFrameIds[0] === frame.id}
                    {#each ['n','s','e','w','nw','ne','sw','se'] as h}<div class="resize-handle {h}" onmousedown={(e) => { e.stopPropagation(); isResizing = true; resizeHandleIdx = h; dragStart = {x:e.clientX, y:e.clientY}; initial = {x:frame.x, y:frame.y, w:frame.width, h:frame.height}; }}></div>{/each}
                    {#if frame.data.Text}<div class="port in-port"></div><div class="port out-port" onmousedown={(e) => handlePortMouseDown(e, frame.id)}></div>{/if}
                  {/if}
                </div>
              {/if}{/each}{/if}{/each}
            </div>
          {/each}
        </div>{/each}
      </div>
    </div>
    <aside class="sidebar-right">
      <div class="panel-header">Formate <button class="header-btn" onclick={handleAddStyle}>A+</button> <button class="header-btn" onclick={handleAddCharStyle}>Z+</button></div>
      <div class="styles-list">{#each doc.styles.paragraph_styles as s}<div class="style-item" onclick={() => { currentEditingStyle = s; showStyleEditorModal = true; }}>{s.name} (A)</div>{/each}</div>
      <div class="styles-list">{#each doc.styles.character_styles as s}<div class="style-item" onclick={() => { currentEditingCharStyle = s; showCharStyleEditorModal = true; }}>{s.name} (Z)</div>{/each}</div>
      <div class="panel-header" style="margin-top: 20px;">Eigenschaften</div>
      {#if selectedFrame}<div class="properties">
        {#if selectedFrame.data.Text}<label>Absatzformat <select bind:value={selectedFrame.data.Text.paragraph_style}>{#each doc.styles.paragraph_styles as s}<option value={s.name}>{s.name}</option>{/each}</select></label><label><input type="checkbox" bind:checked={selectedFrame.data.Text.align_to_baseline_grid} /> Am Grundraster ausrichten</label><textarea bind:value={selectedFrame.data.Text.content}></textarea>{/if}
        {#if selectedFrame.data.Image}<div class="place-image-action"><button onclick={handlePlaceImage}>Bild platzieren...</button></div>{/if}
        <div class="prop-group"><label>X <input type="number" bind:value={selectedFrame.x} /></label><label>Y <input type="number" bind:value={selectedFrame.y} /></label></div>
        <div class="prop-group"><label>W <input type="number" bind:value={selectedFrame.width} /></label><label>H <input type="number" bind:value={selectedFrame.height} /></label></div>
      </div>{:else if selectedFrameIds.length > 1}<div class="empty-state">{selectedFrameIds.length} Objekte</div><div class="properties"><button onclick={() => handleAlign('left')}>L-Align</button></div>{/if}
      <div class="panel-header" style="margin-top: 20px;">Grundlinienraster</div>
      <div class="properties"><label><input type="checkbox" bind:checked={doc.baseline_grid.visible} /> Anzeigen</label><div class="prop-group"><label>Abst. <input type="number" bind:value={doc.baseline_grid.line_height} /></label><label>Vers. <input type="number" bind:value={doc.baseline_grid.offset} /></label></div></div>
      <div class="panel-header" style="margin-top: 20px;">Ansicht</div><div class="properties"><label>Zoom <input type="range" min="0.1" max="2" step="0.1" bind:value={zoom} /></label><label><input type="checkbox" bind:checked={showGuides} /> Hilfslinien</label></div>
    </aside>
  </div>
  <footer class="status-bar"><span>Bereit</span><span>{(zoom * 100).toFixed(0)}%</span></footer>
  {#if showFindBar}<div class="find-bar"><input type="text" placeholder="Suchen..." bind:value={findQuery} oninput={handleFind} onkeydown={(e) => { if (e.key === 'Enter') { if (e.shiftKey) handleFindPrev(); else handleFindNext(); } }} autofocus /><span class="find-count">{findMatches.length > 0 ? currentMatchIdx + 1 : 0} / {findMatches.length}</span><button onclick={handleFindPrev}>↑</button><button onclick={handleFindNext}>↓</button><button onclick={() => showFindBar = false}>×</button></div>{/if}
</main>

{#if showNewDocModal}<div class="modal-backdrop"><div class="modal"><div class="modal-header">Neu</div><div class="modal-body"><label>Name <input type="text" bind:value={newDocSettings.name} /></label><label>Seiten <input type="number" bind:value={newDocSettings.pages} /></label><label><input type="checkbox" bind:checked={newDocSettings.facingPages} /> Doppelseiten</label></div><div class="modal-footer"><button onclick={() => showNewDocModal = false}>Abbrechen</button><button class="primary" onclick={handleCreateDocument}>Erstellen</button></div></div></div>{/if}
{#if showStyleEditorModal && currentEditingStyle}<div class="modal-backdrop"><div class="modal"><div class="modal-header">Stil: {currentEditingStyle.name}</div><div class="modal-body"><label>Name <input type="text" bind:value={currentEditingStyle.name} /></label><label>Schrift <input type="text" bind:value={currentEditingStyle.font_family} /></label></div><div class="modal-footer"><button class="primary" onclick={() => showStyleEditorModal = false}>Fertig</button></div></div></div>{/if}
{#if showCharStyleEditorModal && currentEditingCharStyle}<div class="modal-backdrop"><div class="modal"><div class="modal-header">Zeichenformat: {currentEditingCharStyle.name}</div><div class="modal-body"><label>Name <input type="text" bind:value={currentEditingCharStyle.name} /></label><label>Schrift <input type="text" bind:value={currentEditingCharStyle.font_family} /></label></div><div class="modal-footer"><button class="primary" onclick={() => showCharStyleEditorModal = false}>Fertig</button></div></div></div>{/if}
{#if showExportModal}<div class="modal-backdrop"><div class="modal"><div class="modal-header">Export</div><div class="modal-body"><label>Format <select bind:value={exportSettings.format}><option value="pdf-x4">PDF/X-4</option><option value="epub-fixed">ePub Fixed</option></select></label></div><div class="modal-footer"><button onclick={() => showExportModal = false}>Abbrechen</button><button class="primary" onclick={handleExport}>Exportieren</button></div></div></div>{/if}

<style>
  :global(body) { margin: 0; padding: 0; background-color: #1e1e1e; color: #ccc; font-family: sans-serif; overflow: hidden; }
  main { display: flex; flex-direction: column; height: 100vh; }
  .menu-bar { height: 32px; background: #2d2d2d; display: flex; align-items: center; padding: 0 12px; font-size: 13px; border-bottom: 1px solid #111; gap: 20px; }
  .menu-dropdown { position: relative; cursor: pointer; }
  .dropdown-content { display: none; position: absolute; top: 100%; left: 0; background: #2d2d2d; border: 1px solid #111; min-width: 150px; z-index: 100; }
  .menu-dropdown:hover .dropdown-content { display: block; }
  .dropdown-content button { width: 100%; background: transparent; border: none; color: #ccc; padding: 8px; text-align: left; cursor: pointer; }
  .toolbar { position: absolute; left: 0; top: 32px; bottom: 25px; width: 40px; background: #2d2d2d; border-right: 1px solid #111; display: flex; flex-direction: column; align-items: center; padding-top: 10px; gap: 5px; z-index: 10; }
  .toolbar button { width: 30px; height: 30px; background: transparent; border: none; color: #ccc; border-radius: 4px; cursor: pointer; }
  .toolbar button.active { background: #007acc; color: white; }
  .content-area { flex: 1; display: flex; margin-left: 40px; overflow: hidden; }
  .sidebar-left, .sidebar-right { width: 240px; background: #252526; display: flex; flex-direction: column; border-right: 1px solid #111; }
  .sidebar-right { border-left: 1px solid #111; border-right: none; overflow-y: auto; }
  .panel-header { background: #333; padding: 6px 12px; font-size: 11px; text-transform: uppercase; font-weight: bold; color: #aaa; display: flex; align-items: center; justify-content: space-between; }
  .header-btn { background: transparent; border: none; color: #aaa; cursor: pointer; font-size: 14px; }
  .workspace-container { flex: 1; overflow: auto; background: #181818; position: relative; padding: 60px; }
  .workspace { display: flex; flex-direction: column; align-items: center; gap: 50px; }
  .spread { display: flex; gap: 2px; background: #000; padding: 2px; box-shadow: 0 10px 30px rgba(0,0,0,0.5); transform: scale(var(--zoom)); transform-origin: top center; }
  .page { background: white; position: relative; color: black; }
  .margin-box { position: absolute; border: 1px solid #ff00ff22; pointer-events: none; }
  .column-gutters { position: absolute; top:0; left:0; right:0; bottom:0; display:flex; }
  .gutter-guide { position: absolute; top:0; bottom:0; background: #00ffff08; border-left: 1px solid #00ffff11; border-right: 1px solid #00ffff11; }
  .baseline-grid { position: absolute; top:0; left:0; right:0; bottom:0; pointer-events: none; }
  .baseline-line { position: absolute; left: 0; right: 0; height: 1px; border-top: 1px solid #44ffff11; }
  .guide { position: absolute; z-index: 10; cursor: grab; }
  .guide.horizontal { left: 0; right: 0; height: 1px; border-top: 1px solid #00ffff44; }
  .guide.vertical { top: 0; bottom: 0; width: 1px; border-left: 1px solid #00ffff44; }
  .ruler { position: absolute; background: #2d2d2d; border: 1px solid #111; z-index: 20; color: #888; font-size: 9px; cursor: crosshair; }
  .top-ruler { top:0; left:40px; right:0; height:20px; }
  .left-ruler { left:0; top:32px; bottom:0; width:20px; }
  .snap-guide { position: absolute; border: 1px dashed #ff00ff; z-index: 100; pointer-events: none; }
  .snap-guide.horizontal { left: 0; right: 0; }
  .snap-guide.vertical { top: 0; bottom: 0; }
  .frame { position: absolute; border: 1px solid transparent; }
  .frame.selected { border: 1px solid var(--layer-color, #007acc) !important; box-shadow: 0 0 0 1px var(--layer-color, #007acc); }
  .image-frame { background: #253340; display: flex; align-items: center; justify-content: center; }
  .frame-preview { position: absolute; border: 1px solid #007acc33; pointer-events: none; }
  .resize-handle { position: absolute; width: 8px; height: 8px; background: white; border: 1px solid var(--layer-color, #007acc); }
  .resize-handle.n { top: -4px; left: calc(50% - 4px); cursor: n-resize; }
  .resize-handle.s { bottom: -4px; left: calc(50% - 4px); cursor: s-resize; }
  .resize-handle.e { right: -4px; top: calc(50% - 4px); cursor: e-resize; }
  .resize-handle.w { left: -4px; top: calc(50% - 4px); cursor: w-resize; }
  .resize-handle.nw { top: -4px; left: -4px; cursor: nw-resize; }
  .resize-handle.ne { top: -4px; right: -4px; cursor: ne-resize; }
  .resize-handle.sw { bottom: -4px; left: -4px; cursor: sw-resize; }
  .resize-handle.se { bottom: -4px; right: -4px; cursor: se-resize; }
  .port { position: absolute; width: 10px; height: 10px; background: white; border: 1px solid var(--layer-color, #007acc); }
  .in-port { top: 10px; left: -5px; }
  .out-port { bottom: 10px; right: -5px; cursor: crosshair; }
  .properties { padding: 8px 12px; display: flex; flex-direction: column; gap: 8px; font-size: 12px; }
  .prop-group { display: flex; gap: 8px; }
  .prop-group label { flex: 1; }
  .style-item { padding: 4px 12px; border-bottom: 1px solid #333; cursor: pointer; font-size: 12px; }
  .page-thumb { padding: 10px; display: flex; flex-direction: column; align-items: center; gap: 5px; position: relative; cursor: pointer; }
  .page-thumb.active { background: #007acc22; border-left: 2px solid #007acc; }
  .thumb-box { width: 50px; height: 70px; background: #444; border: 1px solid #555; }
  .delete-btn { position: absolute; top: 2px; right: 2px; background: #c43; color: white; border: none; border-radius: 50%; width: 14px; height: 14px; font-size: 10px; cursor: pointer; display: none; }
  .page-thumb:hover .delete-btn { display: flex; align-items: center; justify-content: center; }
  .modal-backdrop { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 1000; }
  .modal { background: #2d2d2d; width: 400px; border-radius: 4px; box-shadow: 0 10px 30px rgba(0,0,0,0.5); border: 1px solid #444; }
  .modal-header { padding: 12px; background: #333; font-weight: bold; border-bottom: 1px solid #444; }
  .modal-body { padding: 15px; display: flex; flex-direction: column; gap: 12px; }
  .modal-footer { padding: 12px; border-top: 1px solid #444; display: flex; justify-content: flex-end; gap: 10px; }
  .status-bar { height: 25px; background: #007acc; color: white; display: flex; align-items: center; padding: 0 10px; font-size: 11px; gap: 20px; }
  input, select, textarea { background: #3c3c3c; border: 1px solid #555; color: white; padding: 4px 8px; border-radius: 2px; }
  button.primary { background: #007acc; color: white; border: none; padding: 6px 12px; border-radius: 2px; cursor: pointer; }
  .layer-item { display: flex; align-items: center; padding: 4px 12px; gap: 8px; border-bottom: 1px solid #333; cursor: grab; font-size: 12px; }
  .layer-color { width: 12px; height: 12px; border-radius: 2px; }
  .layer-name-input { background: transparent; border: none; color: #eee; flex: 1; padding: 2px; }
  .find-bar { position: absolute; top: 40px; right: 260px; background: #2d2d2d; border: 1px solid #444; padding: 4px 8px; display: flex; align-items: center; gap: 8px; border-radius: 4px; box-shadow: 0 4px 12px rgba(0,0,0,0.4); z-index: 100; }
  .find-bar input { width: 150px; height: 20px; font-size: 12px; }
  .find-count { font-size: 10px; color: #888; min-width: 30px; text-align: center; }
  .find-bar button { background: transparent; border: none; color: #ccc; cursor: pointer; padding: 2px; }
</style>
