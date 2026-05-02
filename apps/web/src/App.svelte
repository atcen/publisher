<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  type Pt = number;
  interface TextFrame { content: string; paragraph_style?: string; next_frame_id?: string; prev_frame_id?: string; align_to_baseline_grid: boolean; }
  interface ImageFrame { asset_path: string; content_x: Pt; content_y: Pt; content_scale_x: number; content_scale_y: number; fitting: 'Fill' | 'Fit' | 'Stretch' | 'Original' | 'Custom'; }
  interface Frame { id: string; layer_id: string; x: Pt; y: Pt; width: Pt; height: Pt; rotation: number; fill_color?: string; stroke_color?: string; stroke_width: Pt; data: { Text?: TextFrame; Image?: ImageFrame; Shape?: { shape_type: string }; Group?: { frames: Frame[] }; }; }
  interface Layer { id: string; name: string; visible: boolean; locked: boolean; color: string; }
  interface Page { width: Pt; height: Pt; margins: { top: number; bottom: number; inside: number; outside: number }; column_count: number; gutter_width: number; guides: Guide[]; frames: Frame[]; applied_parent_id?: string; }
  interface Guide { position: Pt; orientation: 'Horizontal' | 'Vertical'; locked: boolean; color: string | null; }
  interface Spread { pages: Page[]; }
  interface ParentPage { id: string; name: string; spread: Spread; based_on_id?: string; }
  interface DocumentMetadata { name: string; author: string; description: string; created_at: number; modified_at: number; dpi: number; default_unit: "Point"; default_bleed: { top: number; bottom: number; inside: number; outside: number }; color_profile: string; facing_pages: boolean; }
  interface FontVariationAxis { tag: string; name: string; min_value: number; max_value: number; default_value: number; }
  interface FontVariationSetting { tag: string; value: number; }
  interface FontResource { id: string; name: string; family: string; style: string; data: Uint8Array; variation_axes: FontVariationAxis[]; }
  type KerningMode = 'Metric' | 'Optical' | 'None';
  interface ParagraphStyle { name: string; based_on?: string; font_family?: string; font_size?: Pt; alignment?: 'Left' | 'Center' | 'Right' | 'Justify'; variation_settings: FontVariationSetting[]; kerning_mode?: KerningMode; }
  interface CharacterStyle { name: string; based_on?: string; font_family?: string; font_style?: string; font_size?: Pt; variation_settings: FontVariationSetting[]; kerning_mode?: KerningMode; }
  interface Styles { paragraph_styles: ParagraphStyle[]; character_styles: CharacterStyle[]; object_styles: unknown[]; }
  interface BaselineGrid { line_height: Pt; offset: Pt; visible: boolean; color: string; }
  type Color = { Rgb: { r: number; g: number; b: number } } | { Cmyk: { c: number; m: number; y: number; k: number } } | { Spot: { name: string; alternate_cmyk: [number, number, number, number]; tint: number } };
  interface ColorSwatch { name: string; color: Color; }
  interface Document { metadata: DocumentMetadata; fonts: FontResource[]; icc_profiles: unknown[]; swatches: ColorSwatch[]; styles: Styles; spreads: Spread[]; parent_pages: ParentPage[]; layers: Layer[]; baseline_grid: BaselineGrid; }
  interface AppPreferences { theme: string; default_unit: string; autosave_interval: number; recent_files: string[]; }

  let activeTool = $state('select'); let zoom = $state(1); let activePageIndex = $state(0); let selectedFrameIds = $state<string[]>([]);
  let isContentMode = $state(false);
  let currentFilePath = $state<string | null>(null); let hasUnsavedChanges = $state(false); let undoStack = $state<string[]>([]); let redoStack = $state<string[]>([]);
  let showNewDocModal = $state(false); let showStyleEditorModal = $state(false); let showCharStyleEditorModal = $state(false); let showExportModal = $state(false);
  let showSwatchModal = $state(false); let showPrefsModal = $state(false);
  let showFindBar = $state(false); let showReplaceFields = $state(false);
  let findQuery = $state(""); let replaceQuery = $state(""); let useGrep = $state(false);
  let findMatches = $state<{ frameId: string, pageIndex: number }[]>([]); let currentMatchIdx = $state(-1);
  let currentEditingStyle = $state<ParagraphStyle | null>(null); let currentEditingCharStyle = $state<CharacterStyle | null>(null);
  let currentEditingSwatch = $state<ColorSwatch | null>(null);
  let isLinking = $state(false); let linkingSourceFrameId = $state<string | null>(null);
  let showGuides = $state(true); let guidesLocked = $state(false); let isDraggingGuide = $state(false); let currentDraggingGuide = $state<{ page: Page, guide: Guide } | null>(null);
  let exportSettings = $state({ format: 'pdf-x4', pageRange: 'all', customRange: '', includeBleed: true, embedFonts: true, compression: 'high' });
  let snapX = $state<number | null>(null); let snapY = $state<number | null>(null);
  let draggedLayerIndex = $state<number | null>(null); let draggedPageIndex = $state<number | null>(null);
  let prefs = $state<AppPreferences>({ theme: "dark", default_unit: "pt", autosave_interval: 60, recent_files: [] });

  let doc = $state<Document>({
    metadata: { name: "Untitled", author: "", description: "", created_at: 0, modified_at: 0, dpi: 72, default_unit: "Point", default_bleed: { top: 0, bottom: 0, inside: 0, outside: 0 }, color_profile: "sRGB", facing_pages: true },
    fonts: [], icc_profiles: [], swatches: [{ name: "Schwarz", color: { Cmyk: { c: 0, m: 0, y: 0, k: 1 } } }, { name: "Weiß", color: { Cmyk: { c: 0, m: 0, y: 0, k: 0 } } }, { name: "Registration", color: { Cmyk: { c: 1, m: 1, y: 1, k: 1 } } }], 
    styles: { paragraph_styles: [{ name: "Standard", font_family: "Inter", font_size: 12, variation_settings: [], kerning_mode: 'Metric' }], character_styles: [{ name: "[Keines]", variation_settings: [], kerning_mode: 'Metric' }], object_styles: [] },
    spreads: [{ pages: [{ width: 595.27, height: 841.89, margins: { top: 36, bottom: 36, inside: 36, outside: 36 }, column_count: 2, gutter_width: 12, guides: [], frames: [] }] }],
    parent_pages: [],
    layers: [{ id: "layer-1", name: "Ebene 1", visible: true, locked: false, color: "#007acc" }],
    baseline_grid: { line_height: 12, offset: 0, visible: false, color: "#44ffff33" }
  });

  onMount(() => {
    let autoSaveInterval: number;
    const init = async () => {
      try { prefs = await invoke<AppPreferences>("load_preferences"); } catch (e) { console.error("Load prefs failed", e); }
      try {
        const recoveryJson = await invoke<string | null>("check_recovery_file");
        if (recoveryJson && confirm("Wiederherstellung: Ein nicht gespeichertes Dokument wurde gefunden. Wiederherstellen?")) {
          doc = JSON.parse(recoveryJson); hasUnsavedChanges = true;
        } else { await invoke("clear_recovery_file"); }
      } catch (e) { console.error("Recovery check failed", e); }
      autoSaveInterval = setInterval(async () => {
        if (hasUnsavedChanges) {
          try { await invoke("save_recovery_file", { documentJson: JSON.stringify(doc) }); }
          catch (e) { console.error("Auto-save failed", e); }
        }
      }, prefs.autosave_interval * 1000) as unknown as number;
    };
    init();
    return () => { if (autoSaveInterval) clearInterval(autoSaveInterval); };
  });

  async function handleSavePrefs() { try { await invoke("save_preferences", { preferences: prefs }); showPrefsModal = false; } catch (e) { alert("Fehler: " + e); } }

  function pushToUndo() { undoStack.push(JSON.stringify(doc)); if (undoStack.length > 50) undoStack.shift(); redoStack = []; }
  function handleUndo() { if (undoStack.length > 0) { redoStack.push(JSON.stringify(doc)); doc = JSON.parse(undoStack.pop()!); hasUnsavedChanges = true; } }
  function handleRedo() { if (redoStack.length > 0) { undoStack.push(JSON.stringify(doc)); doc = JSON.parse(redoStack.pop()!); hasUnsavedChanges = true; } }
  function markDocumentAsChanged() { hasUnsavedChanges = true; }

  function updateRecentFiles(path: string) {
    prefs.recent_files = [path, ...prefs.recent_files.filter(p => p !== path)].slice(0, 10);
    handleSavePrefs();
  }

  function handleFind() {
    const q = findQuery; if (q.length === 0) { findMatches = []; currentMatchIdx = -1; return; }
    const matches: { frameId: string, pageIndex: number }[] = [];
    let pIdx = 0; 
    try {
      const regex = useGrep ? new RegExp(q, 'gi') : null;
      for (const s of doc.spreads) for (const p of s.pages) { 
        for (const f of p.frames) {
          if (f.data.Text) {
            const content = f.data.Text.content;
            const isMatch = regex ? regex.test(content) : content.toLowerCase().includes(q.toLowerCase());
            if (isMatch) matches.push({ frameId: f.id, pageIndex: pIdx });
          }
        }
        pIdx++; 
      }
    } catch (e) { console.error("Invalid Regex", e); }
    findMatches = matches; if (matches.length > 0) { currentMatchIdx = 0; goToMatch(0); } else currentMatchIdx = -1;
  }
  function goToMatch(idx: number) { const m = findMatches[idx]; if (m) { activePageIndex = m.pageIndex; selectedFrameIds = [m.frameId]; } }
  function handleFindNext() { if (findMatches.length > 0) { currentMatchIdx = (currentMatchIdx + 1) % findMatches.length; goToMatch(currentMatchIdx); } }
  function handleFindPrev() { if (findMatches.length > 0) { currentMatchIdx = (currentMatchIdx - 1 + findMatches.length) % findMatches.length; goToMatch(currentMatchIdx); } }

  function handleReplace() {
    if (currentMatchIdx === -1 || findMatches.length === 0) return;
    const m = findMatches[currentMatchIdx];
    for (const s of doc.spreads) for (const p of s.pages) {
      const f = p.frames.find(x => x.id === m.frameId);
      if (f?.data.Text) {
        pushToUndo();
        if (useGrep) { f.data.Text.content = f.data.Text.content.replace(new RegExp(findQuery, 'gi'), replaceQuery); }
        else { f.data.Text.content = f.data.Text.content.split(findQuery).join(replaceQuery); }
        markDocumentAsChanged(); handleFind(); return;
      }
    }
  }
  function handleReplaceAll() {
    pushToUndo(); let count = 0;
    for (const s of doc.spreads) for (const p of s.pages) for (const f of p.frames) if (f.data.Text) {
      const original = f.data.Text.content;
      if (useGrep) { f.data.Text.content = f.data.Text.content.replace(new RegExp(findQuery, 'gi'), replaceQuery); }
      else { f.data.Text.content = f.data.Text.content.split(findQuery).join(replaceQuery); }
      if (f.data.Text.content !== original) count++;
    }
    if (count > 0) { markDocumentAsChanged(); alert(`${count} Änderungen vorgenommen.`); handleFind(); }
  }

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
  function handleAddStyle() { const s: ParagraphStyle = { name: `Stil ${doc.styles.paragraph_styles.length + 1}`, variation_settings: [], kerning_mode: 'Metric' }; doc.styles.paragraph_styles.push(s); currentEditingStyle = s; showStyleEditorModal = true; markDocumentAsChanged(); }
  function handleAddCharStyle() { const s: CharacterStyle = { name: `Zeichen ${doc.styles.character_styles.length + 1}`, variation_settings: [], kerning_mode: 'Metric' }; doc.styles.character_styles.push(s); currentEditingCharStyle = s; showCharStyleEditorModal = true; markDocumentAsChanged(); }
  function handleAddLayer() { pushToUndo(); doc.layers.unshift({ id: crypto.randomUUID(), name: `Ebene ${doc.layers.length + 1}`, visible: true, locked: false, color: '#007acc' }); markDocumentAsChanged(); }
  
  function handleAddParentPage() {
    pushToUndo(); const id = crypto.randomUUID(); const prefix = String.fromCharCode(65 + doc.parent_pages.length); const name = `${prefix}-Elternseite`;
    const spread: Spread = { pages: [{ width: 595.27, height: 841.89, margins: { top: 36, bottom: 36, inside: 36, outside: 36 }, column_count: 2, gutter_width: 12, guides: [], frames: [] }] };
    if (doc.metadata.facing_pages) spread.pages.push(JSON.parse(JSON.stringify(spread.pages[0])));
    doc.parent_pages.push({ id, name, spread }); markDocumentAsChanged();
  }
  function handleRenameParentPage(id: string, newName: string) {
    if (newName.toLowerCase().includes("master")) { alert("Fehler: Begriff 'Master' unzulässig."); return; }
    const p = doc.parent_pages.find(p => p.id === id); if (p) { pushToUndo(); p.name = newName; markDocumentAsChanged(); }
  }
  function handleDeleteParentPage(id: string) {
    if (confirm("Löschen?")) { pushToUndo(); doc.parent_pages = doc.parent_pages.filter(p => p.id !== id); for (const s of doc.spreads) for (const p of s.pages) if (p.applied_parent_id === id) p.applied_parent_id = undefined; markDocumentAsChanged(); }
  }
  function handleOverrideParentFrame(page: Page, frame: Frame) { pushToUndo(); const nf = JSON.parse(JSON.stringify(frame)); nf.id = crypto.randomUUID(); page.frames.push(nf); selectedFrameIds = [nf.id]; markDocumentAsChanged(); }

  function handleAddSwatch() { const s: ColorSwatch = { name: "Neues Farbfeld", color: { Rgb: { r: 0, g: 0, b: 0 } } }; doc.swatches.push(s); currentEditingSwatch = s; showSwatchModal = true; markDocumentAsChanged(); }
  async function handleConvertSwatch(swatch: ColorSwatch) { if ('Spot' in swatch.color) return; try { swatch.color = await invoke<Color>("convert_color", { color: swatch.color }); markDocumentAsChanged(); } catch (e) { console.error("Conversion failed", e); } }
  function handleToggleSwatchType(swatch: ColorSwatch) {
    if ('Rgb' in swatch.color) swatch.color = { Cmyk: { c: 0, m: 0, y: 0, k: 0 } };
    else if ('Cmyk' in swatch.color) swatch.color = { Spot: { name: swatch.name, alternate_cmyk: [0, 0, 0, 1], tint: 1.0 } };
    else swatch.color = { Rgb: { r: 0, g: 0, b: 0 } };
    markDocumentAsChanged();
  }

  async function handleApplyGridPreset(page: Page, preset: string) { try { pushToUndo(); const updatedPage = await invoke<Page>("apply_grid_preset", { page, preset }); Object.assign(page, updatedPage); markDocumentAsChanged(); } catch (e) { console.error("Preset failed", e); } }

  async function handleOpen() { try { const resp = await invoke<string>("open_document"); const data = JSON.parse(resp); const content = await invoke<string>("read_document", { filePath: data.file_path }); doc = JSON.parse(content); currentFilePath = data.file_path; hasUnsavedChanges = false; updateRecentFiles(data.file_path); await invoke("clear_recovery_file"); } catch (e) { alert("Failed: " + e); } }
  async function handleOpenRecent(path: string) { try { const content = await invoke<string>("read_document", { filePath: path }); doc = JSON.parse(content); currentFilePath = path; hasUnsavedChanges = false; updateRecentFiles(path); await invoke("clear_recovery_file"); } catch (e) { alert("Failed: " + e); } }
  async function handleSave() { try { if (!currentFilePath) { await handleSaveAs(); return; } doc.metadata.modified_at = Math.floor(Date.now() / 1000); const documentJson = JSON.stringify(doc, null, 2); await invoke<string>("save_document_file", { filePath: currentFilePath, documentJson }); hasUnsavedChanges = false; await invoke("clear_recovery_file"); } catch (e) { alert("Failed: " + e); } }
  async function handleSaveAs() { try { doc.metadata.modified_at = Math.floor(Date.now() / 1000); const documentJson = JSON.stringify(doc, null, 2); const path = await invoke<string>("save_as_file", { documentJson }); if (path) { currentFilePath = path; hasUnsavedChanges = false; updateRecentFiles(path); await invoke("clear_recovery_file"); } } catch (e) { alert("Failed: " + e); } }

  function handleCreateDocument() {
    pushToUndo(); const pages: Page[] = Array.from({length: newDocSettings.pages}, () => ({ width: newDocSettings.width, height: newDocSettings.height, margins: {...newDocSettings.margins}, column_count: newDocSettings.columns, gutter_width: newDocSettings.gutter, guides: [], frames: [], stroke_width: 0 }));
    doc = { ...doc, metadata: { ...doc.metadata, name: newDocSettings.name, facing_pages: newDocSettings.facingPages }, spreads: pages.map(p => ({ pages: [p] })) };
    reorganizeSpreads(); showNewDocModal = false; currentFilePath = null; hasUnsavedChanges = false;
  }

  let activePage = $derived.by((): Page | null => { let c = 0; for (const s of doc.spreads) for (const p of s.pages) { if (c === activePageIndex) return p; c++; } return null; });
  let selectedFrames = $derived.by((): Frame[] => { const res: Frame[] = []; for (const s of doc.spreads) for (const p of s.pages) for (const f of p.frames) if (selectedFrameIds.includes(f.id)) res.push(f); return res; });
  let selectedFrame = $derived(selectedFrames[0] || null);
  let titleText = $derived(`${doc.metadata.name}${hasUnsavedChanges ? " •" : ""}`);

  async function handleCopy() { if (selectedFrameIds.length > 0) await navigator.clipboard.writeText(JSON.stringify({ type: 'publisher-frames', frames: selectedFrames })); }
  async function handleCut() { if (selectedFrameIds.length > 0) { await handleCopy(); pushToUndo(); for (const s of doc.spreads) for (const p of s.pages) p.frames = p.frames.filter(f => !selectedFrameIds.includes(f.id)); selectedFrameIds = []; markDocumentAsChanged(); } }
  async function handlePaste(inPlace = false) { try { const data = JSON.parse(await navigator.clipboard.readText()); if (data.type === 'publisher-frames' && activePage) { pushToUndo(); const offset = inPlace ? 0 : 20; const newIds = data.frames.map((f: any) => { const nf = {...f, id: crypto.randomUUID(), x: f.x + offset, y: f.y + offset}; activePage!.frames.push(nf); return nf.id; }); selectedFrameIds = newIds; markDocumentAsChanged(); } } catch {} }
  async function handlePasteTextOnly() { try { const text = await navigator.clipboard.readText(); if (selectedFrame?.data.Text) { pushToUndo(); selectedFrame.data.Text.content += text; markDocumentAsChanged(); } } catch {} }
  function handleDuplicate() { if (selectedFrameIds.length > 0 && activePage) { pushToUndo(); const newIds = selectedFrames.map(f => { const nf = JSON.parse(JSON.stringify(f)); nf.id = crypto.randomUUID(); nf.x += 10; nf.y += 10; activePage!.frames.push(nf); return nf.id; }); selectedFrameIds = newIds; markDocumentAsChanged(); } }
  function handleGroup() { if (selectedFrameIds.length > 1 && activePage) { pushToUndo(); const selected = activePage.frames.filter(f => selectedFrameIds.includes(f.id)); const remaining = activePage.frames.filter(f => !selectedFrameIds.includes(f.id)); const minX = Math.min(...selected.map(f => f.x)); const minY = Math.min(...selected.map(f => f.y)); const maxX = Math.max(...selected.map(f => f.x + f.width)); const maxY = Math.max(...selected.map(f => f.y + f.height)); selected.forEach(f => { f.x -= minX; f.y -= minY; }); const group: Frame = { id: crypto.randomUUID(), layer_id: selected[0].layer_id, x: minX, y: minY, width: maxX-minX, height: maxY-minY, rotation: 0, stroke_width: 0, data: { Group: { frames: selected } } }; activePage.frames = [...remaining, group]; selectedFrameIds = [group.id]; markDocumentAsChanged(); } }
  function handleUngroup() { if (selectedFrameIds.length === 1 && activePage) { const f = activePage.frames.find(f => f.id === selectedFrameIds[0]); if (f?.data.Group) { pushToUndo(); const children = f.data.Group.frames; children.forEach(c => { c.x += f!.x; c.y += f!.y; c.layer_id = f!.layer_id; }); activePage.frames = activePage.frames.filter(x => x.id !== f!.id).concat(children); selectedFrameIds = children.map(c => c.id); markDocumentAsChanged(); } } }
  async function handleExport() { console.log("Export...", exportSettings); alert("Export gestartet"); showExportModal = false; }

  let isDragging = false; let isResizing = false; let isCreating = false; let dragStart = { x: 0, y: 0 }; let initial = { x: 0, y: 0, w: 0, h: 0 }; let resizeHandleIdx = ""; let currentCreating: Frame | null = null;
  function handleFrameMouseDown(e: MouseEvent, frame: Frame) {
    if (isLinking && linkingSourceFrameId && frame.id !== linkingSourceFrameId && frame.data.Text) { pushToUndo(); const source = selectedFrames.find(f => f.id === linkingSourceFrameId); if (source?.data.Text) { source.data.Text.next_frame_id = frame.id; frame.data.Text.prev_frame_id = source.id; markDocumentAsChanged(); } isLinking = false; linkingSourceFrameId = null; return; }
    const l = doc.layers.find(l => l.id === frame.layer_id); if (l?.locked || !l?.visible || activeTool !== 'select') return;
    e.stopPropagation(); if (e.detail === 2) { isContentMode = true; selectedFrameIds = [frame.id]; return; }
    if (e.shiftKey) { if (selectedFrameIds.includes(frame.id)) selectedFrameIds = selectedFrameIds.filter(id => id !== frame.id); else selectedFrameIds.push(frame.id); } else if (!selectedFrameIds.includes(frame.id)) { selectedFrameIds = [frame.id]; isContentMode = false; }
    isDragging = true; dragStart = { x: e.clientX, y: e.clientY };
    if (isContentMode && selectedFrame?.data.Image) { initial = { x: selectedFrame.data.Image.content_x, y: selectedFrame.data.Image.content_y, w: selectedFrame.data.Image.content_scale_x, h: selectedFrame.data.Image.content_scale_y }; }
    else { initial = { x: frame.x, y: frame.y, w: frame.width, h: frame.height }; }
  }
  function handlePageMouseDown(e: MouseEvent, page: Page) {
    if (activeTool === 'select') { selectedFrameIds = []; return; }
    const targetLayer = doc.layers.find(l => !l.locked && l.visible); if (!targetLayer) return;
    e.stopPropagation(); const r = (e.currentTarget as HTMLElement).getBoundingClientRect(); const x = (e.clientX - r.left)/zoom; const y = (e.clientY - r.top)/zoom;
    const nf: Frame = { id: crypto.randomUUID(), layer_id: targetLayer.id, x, y, width: 0, height: 0, rotation: 0, stroke_width: 0, data: activeTool === 'text' ? { Text: { content: "", align_to_baseline_grid: false } } : { Image: { asset_path: "", content_x: 0, content_y: 0, content_scale_x: 1, content_scale_y: 1, fitting: 'Fit' } } };
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
      if (isContentMode && selectedFrame.data.Image) { selectedFrame.data.Image.content_x = initial.x + dx; selectedFrame.data.Image.content_y = initial.y + dy; }
      else {
        let nx = initial.x + dx; let ny = initial.y + dy; snapX = snapY = null;
        if (!e.altKey) {
          const ts = getSnapTargets(activePage); const threshold = 5/zoom;
          const edgesX = [nx, nx+initial.w/2, nx+initial.w]; const edgesY = [ny, ny+initial.h/2, ny+initial.h];
          for (const t of ts) {
            for (const ex of edgesX) if (Math.abs(ex-t) < threshold) { nx -= (ex-t); snapX = t; break; }
            for (const ey of edgesY) if (Math.abs(ey-t) < threshold) { ny -= (ey-t); snapY = t; break; }
          }
        }
        selectedFrame.x = nx; selectedFrame.y = ny;
      }
      markDocumentAsChanged();
    }
    else if (isResizing && selectedFrame) {
      if (isContentMode && selectedFrame.data.Image) {
        const img = selectedFrame.data.Image;
        if (resizeHandleIdx.includes('e')) img.content_scale_x = Math.max(0.1, initial.w + dx/100);
        if (resizeHandleIdx.includes('s')) img.content_scale_y = Math.max(0.1, initial.h + dy/100);
      } else {
        if (resizeHandleIdx.includes('e')) selectedFrame.width = Math.max(10, initial.w + dx);
        if (resizeHandleIdx.includes('s')) selectedFrame.height = Math.max(10, initial.h + dy);
        if (resizeHandleIdx.includes('w')) { const nw = Math.max(10, initial.w - dx); selectedFrame.x = initial.x + (initial.w - nw); selectedFrame.width = nw; }
        if (resizeHandleIdx.includes('n')) { const nh = Math.max(10, initial.h - dy); selectedFrame.y = initial.y + (initial.h - nh); selectedFrame.height = nh; }
      }
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
    else if (cmd && k === ',') { e.preventDefault(); showPrefsModal = true; }
    else if (cmd && (k === 'f' || k === 'h')) { e.preventDefault(); showFindBar = true; if (k === 'h') showReplaceFields = true; }
    else if (cmd && k === 'z') { e.preventDefault(); if (e.shiftKey) handleRedo(); else handleUndo(); }
    else if (cmd && k === 'y') { e.preventDefault(); handleRedo(); }
    else if (cmd && k === 'c') handleCopy(); 
    else if (cmd && k === 'v') { 
      if (e.shiftKey && e.altKey) handlePasteTextOnly(); 
      else if (e.shiftKey) handlePaste(true); 
      else handlePaste(false); 
    } 
    else if (cmd && k === 'x') handleCut(); else if (cmd && k === 'd') { e.preventDefault(); handleDuplicate(); }
    else if (cmd && k === 'g') { e.preventDefault(); if (e.shiftKey) handleUngroup(); else handleGroup(); }
    else if (e.key === 'Escape') { if (showFindBar) { showFindBar = false; showReplaceFields = false; } else if (isContentMode) isContentMode = false; else if (showPrefsModal) showPrefsModal = false; else selectedFrameIds = []; }
    else if ((e.key === 'Delete' || e.key === 'Backspace') && selectedFrameIds.length > 0 && !['INPUT','TEXTAREA'].includes(document.activeElement?.tagName || '')) { pushToUndo(); for (const s of doc.spreads) for (const p of s.pages) p.frames = p.frames.filter(f => !selectedFrameIds.includes(f.id)); selectedFrameIds = []; markDocumentAsChanged(); }
  }

  const DIN_A4 = { w: 595.27, h: 841.89 };
  let newDocSettings = $state({ name: "Dokument 1", width: DIN_A4.w, height: DIN_A4.h, pages: 1, facingPages: true, columns: 2, gutter: 12, margins: { top: 36, bottom: 36, inside: 36, outside: 36 }, bleed: 0 });

  async function handlePlaceImage() { if (selectedFrameIds.length !== 1 || !selectedFrame?.data.Image) return; try { const path = await invoke<string>("open_file"); if (path) { selectedFrame.data.Image.asset_path = path; markDocumentAsChanged(); } } catch (e) { alert("Failed: " + e); } }

  function getSwatchColor(swatchName: string): string {
    const s = doc.swatches.find(x => x.name === swatchName); if (!s) return "transparent";
    if ('Rgb' in s.color) return `rgb(${s.color.Rgb.r*255},${s.color.Rgb.g*255},${s.color.Rgb.b*255})`;
    if ('Cmyk' in s.color) { const {c,m,y,k} = s.color.Cmyk; return `rgb(${255*(1-c)*(1-k)},${255*(1-m)*(1-k)},${255*(1-y)*(1-k)})`; }
    if ('Spot' in s.color) { const {alternate_cmyk: [c,m,y,k], tint} = s.color.Spot; return `rgb(${255*(1-c*tint)*(1-k*tint)},${255*(1-m*tint)*(1-k*tint)},${255*(1-y*tint)*(1-k*tint)})`; }
    return "gray";
  }

  function getVariationValue(style: ParagraphStyle | CharacterStyle, tag: string): number {
    const s = style.variation_settings.find(x => x.tag === tag); if (s) return s.value;
    const font = doc.fonts.find(f => f.family === style.font_family); return font?.variation_axes.find(a => a.tag === tag)?.default_value || 0;
  }
  function setVariationValue(style: ParagraphStyle | CharacterStyle, tag: string, value: number) {
    let s = style.variation_settings.find(x => x.tag === tag); if (!s) { s = { tag, value }; style.variation_settings.push(s); } else s.value = value;
    markDocumentAsChanged();
  }
  function convertUnit(val: number, from: string, to: string): number { const toPt = { pt: 1, mm: 2.83465, cm: 28.3465, in: 72 }; return (val * toPt[from as keyof typeof toPt]) / toPt[to as keyof typeof toPt]; }
  function displayVal(val: number): string { return convertUnit(val, 'pt', prefs.default_unit).toFixed(2); }
</script>

{#snippet ParentContent(parentId: string, pageIdx: number)}
  {#each doc.parent_pages.filter(p => p.id === parentId) as parent}
    {#if parent.based_on_id}{@render ParentContent(parent.based_on_id, pageIdx)}{/if}
    {#if parent.spread.pages[pageIdx]}
      <div class="parent-content">
        {#each parent.spread.pages[pageIdx].frames as frame}
          <div class="frame parent-frame" onclick={(e) => { e.stopPropagation(); if (activePage) handleOverrideParentFrame(activePage, frame); }} style="left: {frame.x}px; top: {frame.y}px; width: {frame.width}px; height: {frame.height}px; background: {frame.fill_color ? getSwatchColor(frame.fill_color) : 'transparent'}; border-width: {frame.stroke_width}px; border-color: {frame.stroke_color ? getSwatchColor(frame.stroke_color) : 'transparent'};">
             {#if frame.data.Text}{frame.data.Text.content}{/if}<div class="override-hint">Überschreiben</div>
          </div>
        {/each}
      </div>
    {/if}
  {/each}
{/snippet}

<svelte:window on:keydown={handleKeyDown} on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />
<main class:theme-light={prefs.theme === 'light'}>
  <nav class="menu-bar">
    <div class="logo">PUBLISHER</div>
    <div class="menu-items">
      <div class="menu-dropdown"><span>Datei</span><div class="dropdown-content"><button onclick={() => showNewDocModal = true}>Neu...</button><button onclick={handleOpen}>Öffnen...</button><div class="menu-submenu"><span>Zuletzt geöffnet</span><div class="submenu-content">{#each prefs.recent_files as path}<button onclick={() => handleOpenRecent(path)}>{path.split('/').pop()}</button>{/each}</div></div><button onclick={handleSave}>Speichern</button><button onclick={handleSaveAs}>Speichern unter...</button><button onclick={() => showPrefsModal = true}>Einstellungen...</button><button onclick={() => showExportModal = true}>Exportieren...</button></div></div>
      <span>Bearbeiten</span><span>Layout</span><span>Ansicht</span>
    </div>
    <div class="doc-title">{titleText}</div>
  </nav>
  <aside class="toolbar">
    <button class:active={activeTool === 'select'} onclick={() => activeTool = 'select'}>V</button>
    <button class:active={activeTool === 'text'} onclick={() => activeTool = 'text'}>T</button>
    <button class:active={activeTool === 'image'} onclick={() => activeTool = 'image'}>F</button>
  </aside>
  <div class="content-area">
    <aside class="sidebar-left">
      <div class="panel-header">Elternseiten <button class="header-btn" onclick={handleAddParentPage}>+</button></div>
      <div class="pages-list">{#each doc.parent_pages as p}<div class="page-thumb"><div class="thumb-box parent"></div><div class="parent-meta"><input class="parent-name-input" type="text" value={p.name} onchange={(e) => handleRenameParentPage(p.id, (e.target as HTMLInputElement).value)} /><select class="parent-based-on" bind:value={p.based_on_id} onchange={markDocumentAsChanged}><option value={undefined}>[Keine]</option>{#each doc.parent_pages.filter(x => x.id !== p.id) as op}<option value={op.id}>Basis: {op.name}</option>{/each}</select></div><button class="delete-btn" onclick={() => handleDeleteParentPage(p.id)}>×</button></div>{/each}</div>
      <div class="panel-header" style="margin-top: 20px;">Seiten <button class="header-btn" onclick={handleAddPage}>+</button></div>
      <div class="pages-list">{#each doc.spreads.flatMap(s => s.pages) as page, index}<div class="page-thumb" class:active={activePageIndex === index} onclick={() => activePageIndex = index} draggable="true" ondragstart={() => draggedPageIndex = index} ondragover={(e) => { e.preventDefault(); return false; }} ondrop={() => handlePageDrop(index)}><div class="thumb-box"></div><span>Seite {index + 1}</span><button class="delete-btn" onclick={() => handleDeletePage(index)}>×</button></div>{/each}</div>
      <div class="panel-header" style="margin-top: 20px;">Ebenen <button class="header-btn" onclick={handleAddLayer}>+</button></div>
      <div class="layers-list">{#each doc.layers as layer, i}<div class="layer-item" draggable="true" ondragstart={() => draggedLayerIndex = i} ondragover={(e) => { e.preventDefault(); return false; }} ondrop={() => handleLayerDrop(i)}><input type="checkbox" bind:checked={layer.visible} /> <input type="checkbox" bind:checked={layer.locked} /> <div class="layer-color" style="background: {layer.color}"></div> <input class="layer-name-input" type="text" bind:value={layer.name} /></div>{/each}</div>
    </aside>
    <div class="workspace-container" onclick={() => selectedFrameIds = []}>
      <div class="ruler top-ruler" onmousedown={(e) => handleRulerMouseDown(e, 'Vertical')}>{#each Array(20) as _, i}<div class="ruler-tick" style="left: {i * 100 * zoom}px">{convertUnit(i * 100, 'pt', prefs.default_unit).toFixed(0)}</div>{/each}</div>
      <div class="ruler left-ruler" onmousedown={(e) => handleRulerMouseDown(e, 'Horizontal')}>{#each Array(20) as _, i}<div class="ruler-tick" style="top: {i * 100 * zoom}px">{convertUnit(i * 100, 'pt', prefs.default_unit).toFixed(0)}</div>{/each}</div>
      <div class="workspace" style="--zoom: {zoom}">
        {#each doc.spreads as spread}<div class="spread">
          {#each spread.pages as page, pageIdxInSpread}
            <div class="page" style="width: {page.width}px; height: {page.height}px;" onmousedown={(e) => handlePageMouseDown(e, page)}>
              {#if snapX !== null}<div class="snap-guide vertical" style="left: {snapX}px"></div>{/if}
              {#if snapY !== null}<div class="snap-guide horizontal" style="top: {snapY}px"></div>{/if}
              <div class="margin-box" style="top: {page.margins.top}px; bottom: {page.margins.bottom}px; left: {page.margins.inside}px; right: {page.margins.outside}px;">
                {#if page.column_count > 1}<div class="column-gutters">{#each Array(page.column_count - 1) as _, i}{@const colW = (page.width - page.margins.inside - page.margins.outside - (page.column_count - 1) * page.gutter_width) / page.column_count}<div class="gutter-guide" style="left: {(i + 1) * colW + i * page.gutter_width}px; width: {page.gutter_width}px;"></div>{/each}</div>{/if}
              </div>
              {#if doc.baseline_grid.visible}<div class="baseline-grid">{#each Array(Math.floor(page.height / doc.baseline_grid.line_height)) as _, i}<div class="baseline-line" style="top: {doc.baseline_grid.offset + i * doc.baseline_grid.line_height}px"></div>{/each}</div>{/if}
              {#if showGuides}{#each page.guides as guide}<div class="guide" class:horizontal={guide.orientation === 'Horizontal'} class:vertical={guide.orientation === 'Vertical'} style="{guide.orientation === 'Horizontal' ? 'top' : 'left'}: {guide.position}px;" onmousedown={(e) => handleGuideMouseDown(e, page, guide)}></div>{/each}{/if}
              {#if page.applied_parent_id}{@render ParentContent(page.applied_parent_id, pageIdxInSpread)}{/if}
              {#each [...doc.layers].reverse() as layer}{#if layer.visible}{#each page.frames as frame}{#if frame.layer_id === layer.id}
                <div class="frame" class:selected={selectedFrameIds.includes(frame.id)} class:content-mode={isContentMode && selectedFrameIds.includes(frame.id)} onmousedown={(e) => handleFrameMouseDown(e, frame)} style="left: {frame.x}px; top: {frame.y}px; width: {frame.width}px; height: {frame.height}px; transform: rotate({frame.rotation}deg); --layer-color: {layer.color}; background: {frame.fill_color ? getSwatchColor(frame.fill_color) : 'transparent'}; border: {frame.stroke_width}px solid {frame.stroke_color ? getSwatchColor(frame.stroke_color) : 'transparent'};">
                  {#if frame.data.Text}{frame.data.Text.content}{:else if frame.data.Image}<div class="image-content" style="transform: translate({frame.data.Image.content_x}px, {frame.data.Image.content_y}px) scale({frame.data.Image.content_scale_x}, {frame.data.Image.content_scale_y});"><div class="image-placeholder">{#if frame.data.Image.asset_path}Bild{:else}Kein Bild{/if}</div></div>
                  {:else if frame.data.Group}<div class="group-content">{#each frame.data.Group.frames as c}<div class="frame-preview" style="left: {c.x}px; top: {c.y}px; width: {c.width}px; height: {c.height}px;"></div>{/each}</div>{/if}
                  {#if selectedFrameIds.length === 1 && selectedFrameIds[0] === frame.id}
                    {#if !isContentMode}{#each ['n','s','e','w','nw','ne','sw','se'] as h}<div class="resize-handle {h}" onmousedown={(e) => { e.stopPropagation(); isResizing = true; resizeHandleIdx = h; dragStart = {x:e.clientX, y:e.clientY}; initial = {x:frame.x, y:frame.y, w:frame.width, h:frame.height}; }}></div>{/each}
                    {:else if frame.data.Image}<div class="content-handles" style="transform: translate({frame.data.Image.content_x}px, {frame.data.Image.content_y}px);">{#each ['nw','ne','sw','se'] as h}<div class="content-handle {h}" onmousedown={(e) => { e.stopPropagation(); isResizing = true; resizeHandleIdx = h; dragStart = {x:e.clientX, y:e.clientY}; initial = {x:frame.data.Image!.content_x, y:frame.data.Image!.content_y, w:frame.data.Image!.content_scale_x, h:frame.data.Image!.content_scale_y}; }}></div>{/each}</div>{/if}
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
      <div class="panel-header">Farbfelder <button class="header-btn" onclick={handleAddSwatch}>+</button></div>
      <div class="swatches-grid">{#each doc.swatches as swatch}<div class="swatch-item" class:is-spot={'Spot' in swatch.color} onclick={() => { if (selectedFrame) { selectedFrame.fill_color = swatch.name; markDocumentAsChanged(); } }} oncontextmenu={(e) => { e.preventDefault(); currentEditingSwatch = swatch; showSwatchModal = true; }}><div class="swatch-color" style="background: {getSwatchColor(swatch.name)}"></div><span class="swatch-name">{swatch.name}</span></div>{/each}</div>
      <div class="panel-header" style="margin-top: 20px;">Formate <button class="header-btn" onclick={handleAddStyle}>A+</button> <button class="header-btn" onclick={handleAddCharStyle}>Z+</button></div>
      <div class="styles-list">{#each doc.styles.paragraph_styles as s}<div class="style-item" onclick={() => { currentEditingStyle = s; showStyleEditorModal = true; }}>{s.name} (A)</div>{/each}</div>
      <div class="styles-list">{#each doc.styles.character_styles as s}<div class="style-item" onclick={() => { currentEditingCharStyle = s; showCharStyleEditorModal = true; }}>{s.name} (Z)</div>{/each}</div>
      <div class="panel-header" style="margin-top: 20px;">Eigenschaften</div>
      {#if selectedFrame}<div class="properties">
        {#if selectedFrame.data.Text}<label>Absatzformat <select bind:value={selectedFrame.data.Text.paragraph_style}>{#each doc.styles.paragraph_styles as s}<option value={s.name}>{s.name}</option>{/each}</select></label><label><input type="checkbox" bind:checked={selectedFrame.data.Text.align_to_baseline_grid} /> Am Grundraster ausrichten</label><textarea bind:value={selectedFrame.data.Text.content}></textarea>{/if}
        {#if selectedFrame.data.Image}<div class="place-image-action"><button onclick={handlePlaceImage}>Bild platzieren...</button></div>{/if}
        <div class="prop-group"><label>X ({prefs.default_unit}) <input type="number" value={convertUnit(selectedFrame.x, 'pt', prefs.default_unit)} oninput={(e) => { selectedFrame!.x = convertUnit(parseFloat((e.target as HTMLInputElement).value), prefs.default_unit, 'pt'); markDocumentAsChanged(); }} /></label><label>Y <input type="number" value={convertUnit(selectedFrame.y, 'pt', prefs.default_unit)} oninput={(e) => { selectedFrame!.y = convertUnit(parseFloat((e.target as HTMLInputElement).value), prefs.default_unit, 'pt'); markDocumentAsChanged(); }} /></label></div>
        <div class="prop-group"><label>W <input type="number" value={convertUnit(selectedFrame.width, 'pt', prefs.default_unit)} oninput={(e) => { selectedFrame!.width = convertUnit(parseFloat((e.target as HTMLInputElement).value), prefs.default_unit, 'pt'); markDocumentAsChanged(); }} /></label><label>H <input type="number" value={convertUnit(selectedFrame.height, 'pt', prefs.default_unit)} oninput={(e) => { selectedFrame!.height = convertUnit(parseFloat((e.target as HTMLInputElement).value), prefs.default_unit, 'pt'); markDocumentAsChanged(); }} /></label></div>
        <div class="prop-group"><label>Kontur <input type="number" bind:value={selectedFrame.stroke_width} /></label></div>
      </div>{:else if selectedFrameIds.length > 1}<div class="empty-state">{selectedFrameIds.length} Objekte</div><div class="properties"><button onclick={() => handleAlign('left')}>L-Align</button></div>{/if}
      <div class="panel-header" style="margin-top: 20px;">Layout</div>
      {#if activePage}<div class="properties"><label>Elternseite <select bind:value={activePage.applied_parent_id} onchange={markDocumentAsChanged}><option value={undefined}>[Keine]</option>{#each doc.parent_pages as p}<option value={p.id}>{p.name}</option>{/each}</select></label><label>Raster-Preset <select onchange={(e) => handleApplyGridPreset(activePage!, (e.target as HTMLSelectElement).value)}><option value="">[Wählen]</option><option value="TwelveColumn">12 Spalten</option><option value="EightColumn">8 Spalten</option><option value="GoldenRatio">Goldener Schnitt</option><option value="Fibonacci">Fibonacci</option><option value="Manuscript">Manuskript</option></select></label><div class="prop-group"><label>Spalten <input type="number" min="1" bind:value={activePage.column_count} /></label><label>Gutter <input type="number" bind:value={activePage.gutter_width} /></label></div></div>{/if}
      <div class="panel-header" style="margin-top: 20px;">Grundlinienraster</div>
      <div class="properties"><label><input type="checkbox" bind:checked={doc.baseline_grid.visible} /> Anzeigen</label><div class="prop-group"><label>Abst. <input type="number" bind:value={doc.baseline_grid.line_height} /></label><label>Vers. <input type="number" bind:value={doc.baseline_grid.offset} /></label></div></div>
      <div class="panel-header" style="margin-top: 20px;">Ansicht</div><div class="properties"><label>Zoom <input type="range" min="0.1" max="2" step="0.1" bind:value={zoom} /></label><label><input type="checkbox" bind:checked={showGuides} /> Hilfslinien</label></div>
    </aside>
  </div>
  <footer class="status-bar"><span>Bereit</span><span>{displayVal(0)} {prefs.default_unit}</span><span>{(zoom * 100).toFixed(0)}%</span></footer>
  {#if showFindBar}<div class="find-bar" class:expanded={showReplaceFields}><div class="find-row"><input type="text" placeholder="Suchen..." bind:value={findQuery} oninput={handleFind} onkeydown={(e) => { if (e.key === 'Enter') { if (e.shiftKey) handleFindPrev(); else handleFindNext(); } }} autofocus /><span class="find-count">{findMatches.length > 0 ? currentMatchIdx + 1 : 0} / {findMatches.length}</span><button onclick={handleFindPrev}>↑</button><button onclick={handleFindNext}>↓</button><label class="grep-toggle"><input type="checkbox" bind:checked={useGrep} onchange={handleFind} /> GREP</label><button onclick={() => showReplaceFields = !showReplaceFields}>{showReplaceFields ? '▲' : '▼'}</button><button onclick={() => { showFindBar = false; showReplaceFields = false; }}>×</button></div>{#if showReplaceFields}<div class="replace-row"><input type="text" placeholder="Ersetzen durch..." bind:value={replaceQuery} /><button onclick={handleReplace}>Ersetzen</button><button onclick={handleReplaceAll}>Alle ersetzen</button></div>{/if}</div>{/if}
</main>

{#if showNewDocModal}<div class="modal-backdrop"><div class="modal"><div class="modal-header">Neu</div><div class="modal-body"><label>Name <input type="text" bind:value={newDocSettings.name} /></label><label>Seiten <input type="number" bind:value={newDocSettings.pages} /></label><label><input type="checkbox" bind:checked={newDocSettings.facingPages} /> Doppelseiten</label></div><div class="modal-footer"><button onclick={() => showNewDocModal = false}>Abbrechen</button><button class="primary" onclick={handleCreateDocument}>Erstellen</button></div></div></div>{/if}
{#if showPrefsModal}<div class="modal-backdrop"><div class="modal"><div class="modal-header">Einstellungen</div><div class="modal-body"><label>Erscheinungsbild <select bind:value={prefs.theme}><option value="dark">Dunkel</option><option value="light">Hell</option></select></label><label>Standardeinheit <select bind:value={prefs.default_unit}><option value="pt">Punkt (pt)</option><option value="mm">Millimeter (mm)</option><option value="cm">Zentimeter (cm)</option><option value="in">Zoll (in)</option></select></label><label>Auto-Save Intervall (Sekunden) <input type="number" bind:value={prefs.autosave_interval} /></label></div><div class="modal-footer"><button onclick={() => showPrefsModal = false}>Abbrechen</button><button class="primary" onclick={handleSavePrefs}>Speichern</button></div></div></div>{/if}
{#if showStyleEditorModal && currentEditingStyle}<div class="modal-backdrop"><div class="modal"><div class="modal-header">Stil: {currentEditingStyle.name}</div><div class="modal-body"><label>Name <input type="text" bind:value={currentEditingStyle.name} /></label><label>Schrift <input type="text" bind:value={currentEditingStyle.font_family} /></label><label>Unterschneidung (Kerning) <select bind:value={currentEditingStyle.kerning_mode} onchange={markDocumentAsChanged}><option value="Metric">Metrisch</option><option value="Optical">Optisch</option><option value="None">Keine</option></select></label>{#if doc.fonts.find(f => f.family === currentEditingStyle!.font_family)}<div class="variations-panel"><div class="panel-header">Variationsachsen</div>{#each (doc.fonts.find(f => f.family === currentEditingStyle!.font_family)?.variation_axes || []) as axis}<label>{axis.name} ({axis.tag}) <input type="range" min={axis.min_value} max={axis.max_value} step="1" value={getVariationValue(currentEditingStyle!, axis.tag)} oninput={(e) => setVariationValue(currentEditingStyle!, axis.tag, parseFloat((e.target as HTMLInputElement).value))} /> {getVariationValue(currentEditingStyle!, axis.tag)}</label>{/each}</div>{/if}</div><div class="modal-footer"><button class="primary" onclick={() => showStyleEditorModal = false}>Fertig</button></div></div></div>{/if}
{#if showCharStyleEditorModal && currentEditingCharStyle}<div class="modal-backdrop"><div class="modal"><div class="modal-header">Zeichenformat: {currentEditingCharStyle.name}</div><div class="modal-body"><label>Name <input type="text" bind:value={currentEditingCharStyle.name} /></label><label>Schrift <input type="text" bind:value={currentEditingCharStyle.font_family} /></label><label>Unterschneidung (Kerning) <select bind:value={currentEditingCharStyle.kerning_mode} onchange={markDocumentAsChanged}><option value="Metric">Metrisch</option><option value="Optical">Optisch</option><option value="None">Keine</option></select></label>{#if doc.fonts.find(f => f.family === currentEditingCharStyle!.font_family)}<div class="variations-panel"><div class="panel-header">Variationsachsen</div>{#each (doc.fonts.find(f => f.family === currentEditingCharStyle!.font_family)?.variation_axes || []) as axis}<label>{axis.name} ({axis.tag}) <input type="range" min={axis.min_value} max={axis.max_value} step="1" value={getVariationValue(currentEditingCharStyle!, axis.tag)} oninput={(e) => setVariationValue(currentEditingCharStyle!, axis.tag, parseFloat((e.target as HTMLInputElement).value))} /> {getVariationValue(currentEditingCharStyle!, axis.tag)}</label>{/each}</div>{/if}</div><div class="modal-footer"><button class="primary" onclick={() => showCharStyleEditorModal = false}>Fertig</button></div></div></div>{/if}
{#if showExportModal}<div class="modal-backdrop"><div class="modal"><div class="modal-header">Export</div><div class="modal-body"><label>Format <select bind:value={exportSettings.format}><option value="pdf-x4">PDF/X-4</option><option value="epub-fixed">ePub Fixed</option></select></label></div><div class="modal-footer"><button onclick={() => showExportModal = false}>Abbrechen</button><button class="primary" onclick={handleExport}>Exportieren</button></div></div></div>{/if}
{#if showSwatchModal && currentEditingSwatch}<div class="modal-backdrop"><div class="modal"><div class="modal-header">Farbfeld bearbeiten</div><div class="modal-body"><label>Name <input type="text" bind:value={currentEditingSwatch.name} /></label><button onclick={() => handleToggleSwatchType(currentEditingSwatch!)}>Farbtyp ändern</button>{#if 'Rgb' in currentEditingSwatch.color}<div class="color-inputs"><label>R <input type="number" min="0" max="1" step="0.01" bind:value={currentEditingSwatch.color.Rgb.r} /></label><label>G <input type="number" min="0" max="1" step="0.01" bind:value={currentEditingSwatch.color.Rgb.g} /></label><label>B <input type="number" min="0" max="1" step="0.01" bind:value={currentEditingSwatch.color.Rgb.b} /></label></div><button onclick={() => handleConvertSwatch(currentEditingSwatch!)}>Zu CMYK konvertieren</button>{:else if 'Cmyk' in currentEditingSwatch.color}<div class="color-inputs"><label>C <input type="number" min="0" max="1" step="0.01" bind:value={currentEditingSwatch.color.Cmyk.c} /></label><label>M <input type="number" min="0" max="1" step="0.01" bind:value={currentEditingSwatch.color.Cmyk.m} /></label><label>Y <input type="number" min="0" max="1" step="0.01" bind:value={currentEditingSwatch.color.Cmyk.y} /></label><label>K <input type="number" min="0" max="1" step="0.01" bind:value={currentEditingSwatch.color.Cmyk.k} /></label></div><button onclick={() => handleConvertSwatch(currentEditingSwatch!)}>Zu RGB konvertieren</button>{:else if 'Spot' in currentEditingSwatch.color}<div class="color-inputs"><label>C <input type="number" min="0" max="1" step="0.01" bind:value={currentEditingSwatch.color.Spot.alternate_cmyk[0]} /></label><label>M <input type="number" min="0" max="1" step="0.01" bind:value={currentEditingSwatch.color.Spot.alternate_cmyk[1]} /></label><label>Y <input type="number" min="0" max="1" step="0.01" bind:value={currentEditingSwatch.color.Spot.alternate_cmyk[2]} /></label><label>K <input type="number" min="0" max="1" step="0.01" bind:value={currentEditingSwatch.color.Spot.alternate_cmyk[3]} /></label></div><label>Tonwert (Tint) <input type="range" min="0" max="1" step="0.01" bind:value={currentEditingSwatch.color.Spot.tint} /> {(currentEditingSwatch.color.Spot.tint * 100).toFixed(0)}%</label>{/if}</div><div class="modal-footer"><button class="primary" onclick={() => showSwatchModal = false}>Fertig</button></div></div></div>{/if}

<style>
  :global(body) { margin: 0; padding: 0; background-color: #1e1e1e; color: #ccc; font-family: sans-serif; overflow: hidden; }
  main { display: flex; flex-direction: column; height: 100vh; }
  main.theme-light { background-color: #f0f0f0; color: #333; }
  main.theme-light .menu-bar { background: #e0e0e0; border-bottom-color: #ccc; color: #333; }
  main.theme-light .toolbar, main.theme-light .sidebar-left, main.theme-light .sidebar-right { background: #f5f5f5; border-color: #ccc; color: #333; }
  main.theme-light .panel-header { background: #ddd; color: #666; }
  main.theme-light input, main.theme-light select, main.theme-light textarea { background: white; color: #333; border-color: #ccc; }
  .menu-bar { height: 32px; background: #2d2d2d; display: flex; align-items: center; padding: 0 12px; font-size: 13px; border-bottom: 1px solid #111; gap: 20px; }
  .menu-dropdown { position: relative; cursor: pointer; }
  .dropdown-content { display: none; position: absolute; top: 100%; left: 0; background: #2d2d2d; border: 1px solid #111; min-width: 150px; z-index: 100; }
  .menu-dropdown:hover .dropdown-content { display: block; }
  .dropdown-content button { width: 100%; background: transparent; border: none; color: #ccc; padding: 8px; text-align: left; cursor: pointer; }
  .menu-submenu { position: relative; padding: 8px; }
  .submenu-content { display: none; position: absolute; top: 0; left: 100%; background: #2d2d2d; border: 1px solid #111; min-width: 150px; }
  .menu-submenu:hover .submenu-content { display: block; }
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
  .frame.content-mode { border-color: orange !important; box-shadow: 0 0 0 1px orange; }
  .parent-frame { opacity: 0.6; cursor: copy; border: 1px dashed #007acc88 !important; }
  .parent-frame:hover { opacity: 1; border-color: #007acc !important; }
  .override-hint { position: absolute; top: -15px; left: 0; font-size: 9px; color: #007acc; background: white; padding: 1px 4px; display: none; white-space: nowrap; }
  .parent-frame:hover .override-hint { display: block; }
  .image-frame { background: #253340; display: flex; align-items: center; justify-content: center; overflow: hidden; }
  .image-content { position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; }
  .content-handles { position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; }
  .content-handle { position: absolute; width: 8px; height: 8px; background: orange; border: 1px solid white; pointer-events: auto; }
  .content-handle.nw { top: -4px; left: -4px; cursor: nw-resize; }
  .content-handle.ne { top: -4px; right: -4px; cursor: ne-resize; }
  .content-handle.sw { bottom: -4px; left: -4px; cursor: sw-resize; }
  .content-handle.se { bottom: -4px; right: -4px; cursor: se-resize; }
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
  .page-thumb { padding: 10px; display: flex; flex-direction: column; align-items: center; gap: 5px; position: relative; cursor: pointer; border-bottom: 1px solid #333; }
  .page-thumb.active { background: #007acc22; border-left: 2px solid #007acc; }
  .thumb-box { width: 50px; height: 70px; background: #444; border: 1px solid #555; }
  .thumb-box.parent { border-color: #007acc; background: #2d2d2d; }
  .parent-meta { flex: 1; display: flex; flex-direction: column; gap: 4px; overflow: hidden; }
  .parent-based-on { background: transparent; border: 1px solid #444; color: #888; font-size: 9px; padding: 2px; }
  .delete-btn { position: absolute; top: 2px; right: 2px; background: #c43; color: white; border: none; border-radius: 50%; width: 14px; height: 14px; font-size: 10px; cursor: pointer; display: none; }
  .page-thumb:hover .delete-btn { display: flex; align-items: center; justify-content: center; }
  .swatches-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 4px; padding: 8px; }
  .swatch-item { display: flex; align-items: center; gap: 6px; padding: 4px; background: #333; border-radius: 2px; cursor: pointer; font-size: 11px; position: relative; }
  .swatch-item:hover { background: #444; }
  .swatch-item.is-spot::after { content: "•"; position: absolute; top: 2px; right: 4px; color: orange; font-size: 14px; }
  .swatch-color { width: 16px; height: 16px; border: 1px solid #111; }
  .swatch-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .color-inputs { display: grid; grid-template-columns: repeat(2, 1fr); gap: 8px; }
  .variations-panel { padding: 8px; border: 1px solid #444; border-radius: 2px; margin-top: 10px; }
  .variations-panel label { display: flex; flex-direction: column; gap: 4px; font-size: 11px; margin-bottom: 8px; }
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
  .layer-name-input, .parent-name-input { background: transparent; border: none; color: #eee; flex: 1; padding: 2px; font-size: 11px; }
  .parent-name-input { width: 100px; border-bottom: 1px solid transparent; }
  .parent-name-input:focus { border-bottom-color: #007acc; outline: none; }
  .find-bar { position: absolute; top: 40px; right: 260px; background: #2d2d2d; border: 1px solid #444; padding: 4px 8px; display: flex; flex-direction: column; gap: 8px; border-radius: 4px; box-shadow: 0 4px 12px rgba(0,0,0,0.4); z-index: 100; min-width: 300px; }
  .find-bar.expanded { width: 400px; }
  .find-row, .replace-row { display: flex; align-items: center; gap: 8px; width: 100%; }
  .replace-row { margin-top: 8px; border-top: 1px solid #444; padding-top: 8px; }
  .find-bar input[type="text"] { flex: 1; height: 24px; font-size: 12px; }
  .find-count { font-size: 10px; color: #888; min-width: 40px; text-align: center; }
  .find-bar button { background: transparent; border: 1px solid #444; color: #ccc; cursor: pointer; padding: 2px 6px; border-radius: 2px; font-size: 11px; }
  .find-bar button:hover { background: #444; }
  .grep-toggle { font-size: 10px; display: flex; align-items: center; gap: 4px; white-space: nowrap; }
</style>
