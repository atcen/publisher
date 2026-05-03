<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  
  // Stores
  import { docStore } from "./lib/stores/document.svelte";
  import { uiStore } from "./lib/stores/ui.svelte";
  import { prefsStore } from "./lib/stores/prefs.svelte";
  
  // Types
  import type { Page, Frame, Guide, ImageFrame, ParagraphStyle, CharacterStyle, ColorSwatch } from "./lib/types";
  
  
  // Components
  import MenuBar from "./lib/components/layout/MenuBar.svelte";
  import Toolbar from "./lib/components/layout/Toolbar.svelte";
  import SidebarLeft from "./lib/components/layout/SidebarLeft.svelte";
  import SidebarRight from "./lib/components/layout/SidebarRight.svelte";
  import Workspace from "./lib/components/workspace/Workspace.svelte";
  import StatusBar from "./lib/components/layout/StatusBar.svelte";
  import FindBar from "./lib/components/layout/FindBar.svelte";
  import ModalManager from "./lib/components/ModalManager.svelte";

  // Transient edit state (will move to specialized stores later)
  let currentEditingStyle = $state<ParagraphStyle | null>(null);
  let currentEditingCharStyle = $state<CharacterStyle | null>(null);
  let currentEditingSwatch = $state<ColorSwatch | null>(null);

  // Interaction State
  let isDragging = false;
  let isResizing = false;
  let isCreating = false;
  let isDraggingGuide = false;
  let dragStart = { x: 0, y: 0 };
  let initial = { x: 0, y: 0, w: 0, h: 0 };
  let resizeHandleIdx = "";
  let currentCreating: Frame | null = null;
  let currentDraggingGuide = $state<{ page: Page, guide: Guide } | null>(null);
  let linkingSourceFrameId = $state<string | null>(null);
  let isLinking = $state(false);

  let autoSaveInterval: number;

  $effect(() => {
    if (autoSaveInterval) clearInterval(autoSaveInterval);
    if (prefsStore.prefs.autosave_interval > 0) {
      autoSaveInterval = setInterval(async () => {
        if (docStore.hasUnsavedChanges) {
          try { await invoke("save_recovery_file", { documentJson: JSON.stringify(docStore.doc) }); }
          catch (e) { console.error("Auto-save failed", e); }
        }
      }, prefsStore.prefs.autosave_interval * 1000) as unknown as number;
    }
  });

  onMount(() => {
    const init = async () => {
      await prefsStore.load();
      try {
        const recoveryJson = await invoke<string | null>("check_recovery_file");
        if (recoveryJson && confirm("Wiederherstellung: Ein nicht gespeichertes Dokument wurde gefunden. Wiederherstellen?")) {
          docStore.doc = JSON.parse(recoveryJson);
          docStore.hasUnsavedChanges = true;
        } else { await invoke("clear_recovery_file"); }
      } catch (e) { console.error("Recovery check failed", e); }
    };
    init();

    const handlePlaceImageRequest = () => handlePlaceImage();
    window.addEventListener('place-image', handlePlaceImageRequest);

    return () => { 
      if (autoSaveInterval) clearInterval(autoSaveInterval); 
      window.removeEventListener('place-image', handlePlaceImageRequest);
    };
  });

  // --- INTERACTION HANDLERS ---

  function handleFrameMouseDown(e: MouseEvent, frame: Frame) {
    if (isLinking && linkingSourceFrameId && frame.id !== linkingSourceFrameId && frame.data.Text) {
      docStore.pushToUndo();
      const source = docStore.selectedFrames.find(f => f.id === linkingSourceFrameId);
      if (source?.data.Text) {
        source.data.Text.next_frame_id = frame.id;
        frame.data.Text.prev_frame_id = source.id;
        docStore.markModified();
      }
      isLinking = false;
      linkingSourceFrameId = null;
      return;
    }
    const layer = docStore.doc.layers.find(l => l.id === frame.layer_id);
    if (layer?.locked || !layer?.visible || uiStore.activeTool !== 'select') return;
    
    e.stopPropagation();
    if (e.detail === 2) { uiStore.isContentMode = true; uiStore.selectedFrameIds = [frame.id]; return; }
    
    if (e.shiftKey) {
      if (uiStore.selectedFrameIds.includes(frame.id)) uiStore.selectedFrameIds = uiStore.selectedFrameIds.filter(id => id !== frame.id);
      else uiStore.selectedFrameIds.push(frame.id);
    } else if (!uiStore.selectedFrameIds.includes(frame.id)) {
      uiStore.selectedFrameIds = [frame.id];
      uiStore.isContentMode = false;
    }

    isDragging = true;
    dragStart = { x: e.clientX, y: e.clientY };
    if (uiStore.isContentMode && frame.data.Image) {
      initial = { x: frame.data.Image.content_x, y: frame.data.Image.content_y, w: frame.data.Image.content_scale_x, h: frame.data.Image.content_scale_y };
    } else {
      initial = { x: frame.x, y: frame.y, w: frame.width, h: frame.height };
    }
  }

  function handlePageMouseDown(e: MouseEvent, page: Page) {
    if (uiStore.activeTool === 'select') { uiStore.selectedFrameIds = []; return; }
    const targetLayer = docStore.doc.layers.find(l => !l.locked && l.visible);
    if (!targetLayer) return;
    
    e.stopPropagation();
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const x = (e.clientX - r.left) / uiStore.zoom;
    const y = (e.clientY - r.top) / uiStore.zoom;
    
    const nf: Frame = { 
      id: crypto.randomUUID(), 
      layer_id: targetLayer.id, 
      x, y, width: 0, height: 0, rotation: 0, stroke_width: 0, 
      data: uiStore.activeTool === 'text' 
        ? { Text: { content: "", align_to_baseline_grid: false, frame_type: 'Area' } }
        : { Image: { asset_path: "", content_x: 0, content_y: 0, content_scale_x: 1, content_scale_y: 1, fitting: 'Fit' } }    };
    
    page.frames.push(nf);
    uiStore.selectedFrameIds = [nf.id];
    currentCreating = nf;
    isCreating = true;
    dragStart = { x: e.clientX, y: e.clientY };
  }

  async function handleMouseMove(e: MouseEvent) {
    const dx = (e.clientX - dragStart.x) / uiStore.zoom;
    const dy = (e.clientY - dragStart.y) / uiStore.zoom;
    
    if (isDragging && docStore.selectedFrames[0] && docStore.activePage) {
      const frame = docStore.selectedFrames[0];
      if (uiStore.isContentMode && frame.data.Image) {
        frame.data.Image.content_x = initial.x + dx;
        frame.data.Image.content_y = initial.y + dy;
      } else {
        let nx = initial.x + dx;
        let ny = initial.y + dy;
        uiStore.snapX = uiStore.snapY = null;
        
        if (!e.altKey) {
          const { buildSnapTargets, findSnap } = await import("./lib/utils/geometry");
          const targets = buildSnapTargets(docStore.activePage, uiStore.selectedFrameIds, docStore.doc.baseline_grid);
          const snap = await findSnap(nx, ny, frame.width, frame.height, targets, 5 / uiStore.zoom);
          
          if (snap.x) {
            nx = snap.x.position;
            const target = snap.x.target as any;
            uiStore.snapX = target.Margin?.position ?? target.Column?.position ?? target.Guide?.position ?? target.Object?.position ?? null;
          }
          if (snap.y) {
            ny = snap.y.position;
            const target = snap.y.target as any;
            uiStore.snapY = target.Margin?.position ?? target.Guide?.position ?? target.Object?.position ?? target.Baseline?.position ?? null;
          }
        }
        frame.x = nx; frame.y = ny;
      }
      docStore.markModified();
    }
    else if (isResizing && docStore.selectedFrames[0]) {
      const frame = docStore.selectedFrames[0];
      if (uiStore.isContentMode && frame.data.Image) {
        const img = frame.data.Image;
        if (resizeHandleIdx.includes('e')) img.content_scale_x = Math.max(0.1, initial.w + dx / 100);
        if (resizeHandleIdx.includes('s')) img.content_scale_y = Math.max(0.1, initial.h + dy / 100);
      } else if (frame.data.Text?.frame_type === 'Point') {
        const ratio = Math.max(0.1, (initial.w + dx) / initial.w);
        const style = docStore.doc.styles.paragraph_styles.find(s => s.name === frame.data.Text!.paragraph_style) || docStore.doc.styles.paragraph_styles[0];
        const baseFontSize = style?.font_size ?? 12;
        frame.data.Text!.font_size_override = (frame.data.Text!.font_size_override ?? baseFontSize) * ratio;
        frame.width = initial.w * ratio;
        frame.height = initial.h * ratio;
      } else {
        if (resizeHandleIdx.includes('e')) frame.width = Math.max(10, initial.w + dx);
        if (resizeHandleIdx.includes('s')) frame.height = Math.max(10, initial.h + dy);
        if (resizeHandleIdx.includes('w')) { const nw = Math.max(10, initial.w - dx); frame.x = initial.x + (initial.w - nw); frame.width = nw; }
        if (resizeHandleIdx.includes('n')) { const nh = Math.max(10, initial.h - dy); frame.y = initial.y + (initial.h - nh); frame.height = nh; }
      }
      docStore.markModified();
    }
    else if (isCreating && currentCreating) {
      currentCreating.width = Math.max(0, dx);
      currentCreating.height = Math.max(0, dy);
      docStore.markModified();
    }
    else if (isDraggingGuide && currentDraggingGuide) {
      const r = document.querySelector('.workspace')?.getBoundingClientRect();
      if (r) {
        const x = (e.clientX - r.left) / uiStore.zoom;
        const y = (e.clientY - r.top) / uiStore.zoom;
        if (currentDraggingGuide.guide.orientation === 'Horizontal') currentDraggingGuide.guide.position = y;
        else currentDraggingGuide.guide.position = x;
        docStore.markModified();
      }
    }
  }

  function handleMouseUp() {
    if (isCreating && currentCreating && (currentCreating.width < 5 || currentCreating.height < 5)) {
      for (const s of docStore.doc.spreads) for (const p of s.pages) p.frames = p.frames.filter(f => f.id !== currentCreating!.id);
      uiStore.selectedFrameIds = [];
    }
    isDragging = isResizing = isCreating = isDraggingGuide = false;
    uiStore.snapX = uiStore.snapY = null;
    currentCreating = null;
    currentDraggingGuide = null;
  }

  function handleKeyDown(e: KeyboardEvent) {
    const k = e.key.toLowerCase();
    const cmd = navigator.platform.includes("MAC") ? e.metaKey : e.ctrlKey;
    
    if (cmd && k === 's') { e.preventDefault(); docStore.save(); }
    else if (cmd && k === 'n') { e.preventDefault(); uiStore.showNewDocModal = true; }
    else if (cmd && k === ',') { e.preventDefault(); uiStore.showPrefsModal = true; }
    else if (cmd && (k === 'f' || k === 'h')) { e.preventDefault(); uiStore.showFindBar = true; if (k === 'h') uiStore.showReplaceFields = true; }
    else if (cmd && k === 'z') { e.preventDefault(); if (e.shiftKey) docStore.redo(); else docStore.undo(); }
    else if (cmd && k === 'y') { e.preventDefault(); docStore.redo(); }
    else if (e.key === 'Escape') { 
      if (uiStore.showFindBar) { uiStore.showFindBar = false; uiStore.showReplaceFields = false; }
      else if (uiStore.isContentMode) uiStore.isContentMode = false;
      else if (uiStore.showPrefsModal) uiStore.showPrefsModal = false;
      else uiStore.selectedFrameIds = [];
    }
    else if ((e.key === 'Delete' || e.key === 'Backspace') && uiStore.selectedFrameIds.length > 0 && !['INPUT','TEXTAREA'].includes(document.activeElement?.tagName || '')) {
      docStore.pushToUndo();
      for (const s of docStore.doc.spreads) for (const p of s.pages) p.frames = p.frames.filter(f => !uiStore.selectedFrameIds.includes(f.id));
      uiStore.selectedFrameIds = [];
      docStore.markModified();
    }
  }

  async function handlePlaceImage() {
    if (uiStore.selectedFrameIds.length !== 1 || !docStore.selectedFrames[0]?.data.Image) return;
    try {
      const path = await invoke<string>("open_file");
      if (path) {
        docStore.selectedFrames[0].data.Image.asset_path = path;
        docStore.markModified();
      }
    } catch (e) { console.error(e); }
  }
</script>

<svelte:window on:keydown={handleKeyDown} on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />

<main class:theme-light={prefsStore.prefs.theme === 'light'}>
  <MenuBar />
  <Toolbar />
  <div class="content-area">
    <SidebarLeft />
    <Workspace 
      onPageMouseDown={handlePageMouseDown}
      onFrameMouseDown={handleFrameMouseDown}
      onPortMouseDown={(e: MouseEvent, id: string) => { e.stopPropagation(); isLinking = true; linkingSourceFrameId = id; }}
      onRulerMouseDown={(e: MouseEvent, o: 'Horizontal' | 'Vertical') => { e.stopPropagation(); docStore.pushToUndo(); const g: Guide = { position: 0, orientation: o, locked: false, color: null }; if (docStore.activePage) { docStore.activePage.guides.push(g); isDraggingGuide = true; currentDraggingGuide = { page: docStore.activePage, guide: g }; docStore.markModified(); } }}
      onGuideMouseDown={(e: MouseEvent, page: Page, guide: Guide) => { e.stopPropagation(); isDraggingGuide = true; currentDraggingGuide = { page, guide }; }}
      onResizeMouseDown={(e: MouseEvent, frame: Frame, h: string) => { e.stopPropagation(); isResizing = true; resizeHandleIdx = h; dragStart = {x:e.clientX, y:e.clientY}; initial = {x:frame.x, y:frame.y, w:frame.width, h:frame.height}; }}
      onContentHandleMouseDown={(e: MouseEvent, img: ImageFrame, h: string) => { e.stopPropagation(); isResizing = true; resizeHandleIdx = h; dragStart = {x:e.clientX, y:e.clientY}; initial = {x:img.content_x, y:img.content_y, w:img.content_scale_x, h:img.content_scale_y}; }}
    />
    <SidebarRight 
      onEditSwatch={(s: ColorSwatch) => { currentEditingSwatch = s; uiStore.showSwatchModal = true; }}
      onEditParaStyle={(s: ParagraphStyle) => { currentEditingStyle = s; uiStore.showStyleEditorModal = true; }}
      onEditCharStyle={(s: CharacterStyle) => { currentEditingCharStyle = s; uiStore.showCharStyleEditorModal = true; }}
    />
  </div>
  <StatusBar />
  
  {#if uiStore.showFindBar}
    <FindBar />
  {/if}
</main>

<ModalManager 
  bind:currentEditingStyle
  bind:currentEditingCharStyle
  bind:currentEditingSwatch
  onCloseSwatch={() => currentEditingSwatch = null}
/>
<style>
  :global(body) { margin: 0; padding: 0; background-color: #1e1e1e; color: #ccc; font-family: sans-serif; overflow: hidden; }
  main { display: flex; flex-direction: column; height: 100vh; position: relative; }
  main.theme-light { background-color: #f0f0f0; color: #333; }
  .content-area { flex: 1; display: flex; margin-left: 40px; overflow: hidden; }
</style>
