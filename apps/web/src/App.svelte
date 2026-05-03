<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { docStore } from "./lib/stores/document.svelte";
  import { uiStore } from "./lib/stores/ui.svelte";
  import { prefsStore } from "./lib/stores/prefs.svelte";
  import type { Page, Frame, Guide, ImageFrame, ParagraphStyle, CharacterStyle, ColorSwatch, TextFrameType, Orientation } from "./lib/types";
  
  import MenuBar from "./lib/components/layout/MenuBar.svelte";
  import Toolbar from "./lib/components/layout/Toolbar.svelte";
  import SidebarLeft from "./lib/components/layout/SidebarLeft.svelte";
  import SidebarRight from "./lib/components/layout/SidebarRight.svelte";
  import Workspace from "./lib/components/workspace/Workspace.svelte";
  import StatusBar from "./lib/components/layout/StatusBar.svelte";
  import ModalManager from "./lib/components/ModalManager.svelte";

  // Persistent interaction state
  let interaction = {
    active: false,
    type: null as 'create' | 'drag' | 'resize' | 'guide' | null,
    startClient: { x: 0, y: 0 },
    initialModel: { x: 0, y: 0, w: 0, h: 0 },
    frame: null as Frame | null,
    guide: null as Guide | null,
    page: null as Page | null,
    handle: ""
  };

  // State for resource editing
  let currentEditingSwatch = $state<ColorSwatch | undefined>(undefined);
  let currentEditingParaStyle = $state<ParagraphStyle | undefined>(undefined);
  let currentEditingCharStyle = $state<CharacterStyle | undefined>(undefined);

  function handleGlobalMove(e: MouseEvent) {
    if (!interaction.active) return;
    
    const dx = (e.clientX - interaction.startClient.x) / uiStore.zoom;
    const dy = (e.clientY - interaction.startClient.y) / uiStore.zoom;

    if (interaction.type === 'create' && interaction.frame) {
      interaction.frame.width = Math.max(1, dx);
      interaction.frame.height = Math.max(1, dy);
    } else if (interaction.type === 'resize' && interaction.frame) {
      if (interaction.handle.includes('e')) interaction.frame.width = Math.max(2, interaction.initialModel.w + dx);
      if (interaction.handle.includes('s')) interaction.frame.height = Math.max(2, interaction.initialModel.h + dy);
    } else if (interaction.type === 'drag' && interaction.frame) {
      interaction.frame.x = interaction.initialModel.x + dx;
      interaction.frame.y = interaction.initialModel.y + dy;
    } else if (interaction.type === 'guide' && interaction.guide) {
      const pageEl = document.querySelector('.page');
      if (pageEl) {
        const r = pageEl.getBoundingClientRect();
        const x = (e.clientX - r.left) / uiStore.zoom;
        const y = (e.clientY - r.top) / uiStore.zoom;
        interaction.guide.position = interaction.guide.orientation === 'Horizontal' ? y : x;
      }
    }
    docStore.markModified();
  }

  function handleGlobalUp() {
    if (interaction.type === 'create' && interaction.frame) {
      if (uiStore.activeTool === 'text') {
        const area = interaction.frame.width * interaction.frame.height;
        if (area < 100) { // Tiny area means it was likely a click -> Point Text
          interaction.frame.data.Text!.content = ""; 
          docStore.convertTextFrameType(interaction.frame, 'Point');
          // Provide a comfortable initial width for empty frames, but height is now exact from store
          interaction.frame.width = Math.max(interaction.frame.width, 100); 
        }
        uiStore.isContentMode = true;
      }
      docStore.markModified();
    }
    interaction.active = false;
    interaction.type = null;
    interaction.frame = null;
    interaction.guide = null;
    interaction.page = null;
  }


  onMount(() => {
    prefsStore.load();
    window.addEventListener("mousemove", handleGlobalMove, { capture: true });
    window.addEventListener("mouseup", handleGlobalUp, { capture: true });
    window.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && uiStore.selectedFrameIds.length === 1 && !uiStore.isContentMode) {
        const frame = docStore.selectedFrames[0];
        if (frame?.data.Text) {
          e.preventDefault();
          uiStore.isContentMode = true;
        }
      }
      if (e.key === "Escape") {
        if (uiStore.isContentMode) {
          uiStore.isContentMode = false;
        } else {
          uiStore.resetSelection();
        }
      }
    });
    return () => {
      window.removeEventListener("mousemove", handleGlobalMove, { capture: true });
      window.removeEventListener("mouseup", handleGlobalUp, { capture: true });
    };
  });

  function startCreating(e: MouseEvent, page: Page) {
    if (uiStore.activeTool === 'select') { uiStore.resetSelection(); return; }
    e.stopPropagation();
    
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const x = (e.clientX - r.left) / uiStore.zoom;
    const y = (e.clientY - r.top) / uiStore.zoom;
    
    const nf: Frame = { 
      id: crypto.randomUUID(), layer_id: docStore.doc.layers[0].id, x, y, width: 1, height: 1, rotation: 0, stroke_width: 0, 
      data: uiStore.activeTool === 'text' 
        ? { Text: { content: "", align_to_baseline_grid: false, frame_type: 'Area' } }
        : { Image: { asset_path: "", content_x: 0, content_y: 0, content_scale_x: 1, content_scale_y: 1, fitting: 'Fit' } } 
    };
    page.frames.push(nf);
    uiStore.selectedFrameIds = [nf.id];
    
    interaction.active = true;
    interaction.type = 'create';
    interaction.startClient = { x: e.clientX, y: e.clientY };
    // CRITICAL FIX: Fetch the proxy object from the array instead of using the raw `nf` object
    interaction.frame = page.frames[page.frames.length - 1];
  }
</script>

<main 
  class:theme-light={prefsStore.prefs.theme === 'light'} 
  style="cursor: {uiStore.activeTool === 'text' ? 'text' : 'default'}"
>
  <MenuBar />
  <Toolbar />
  <div class="content-area">
    <SidebarLeft />
    <Workspace 
      onPageMouseDown={startCreating}
      onFrameMouseDown={(e: MouseEvent, f: Frame) => { 
        e.stopPropagation(); 
        if (!uiStore.selectedFrameIds.includes(f.id)) uiStore.isContentMode = false;
        uiStore.selectedFrameIds = [f.id]; 
        interaction.active = true; 
        interaction.type = 'drag'; 
        interaction.startClient = { x: e.clientX, y: e.clientY }; 
        interaction.initialModel = {x:f.x, y:f.y, w:f.width, h:f.height}; 
        interaction.frame = f; 
      }}
      onResizeMouseDown={(e: MouseEvent, f: Frame, h: string) => { e.stopPropagation(); interaction.active = true; interaction.type = 'resize'; interaction.handle = h; interaction.startClient = { x: e.clientX, y: e.clientY }; interaction.initialModel = {x:f.x, y:f.y, w:f.width, h:f.height}; interaction.frame = f; }}
      onRulerMouseDown={(e: MouseEvent, o: Orientation) => {
        e.stopPropagation();
        if (!docStore.activePage) return;
        docStore.pushToUndo();
        const g: Guide = { position: 0, orientation: o, locked: false, color: null };
        docStore.activePage.guides.push(g);
        interaction.active = true;
        interaction.type = 'guide';
        interaction.guide = g;
        interaction.page = docStore.activePage;
        interaction.startClient = { x: e.clientX, y: e.clientY };
      }}
      onGuideMouseDown={(e: MouseEvent, p: Page, g: Guide) => {
        e.stopPropagation();
        interaction.active = true;
        interaction.type = 'guide';
        interaction.guide = g;
        interaction.page = p;
        interaction.startClient = { x: e.clientX, y: e.clientY };
      }}
      onPortMouseDown={(e: MouseEvent, id: string) => { e.stopPropagation(); console.log('Linking from', id); }}
      onContentHandleMouseDown={(e: MouseEvent, img: ImageFrame, h: string) => { e.stopPropagation(); console.log('Content handle', h); }}
    />
    <SidebarRight 
      onEditSwatch={(s: ColorSwatch) => { currentEditingSwatch = s; uiStore.showSwatchModal = true; }}
      onEditParaStyle={(s: ParagraphStyle) => { currentEditingParaStyle = s; uiStore.showStyleEditorModal = true; }}
      onEditCharStyle={(s: CharacterStyle) => { currentEditingCharStyle = s; uiStore.showStyleEditorModal = true; }}
    />
  </div>
  <StatusBar />
</main>
<ModalManager 
  bind:currentEditingSwatch 
  bind:currentEditingStyle={currentEditingParaStyle} 
  bind:currentEditingCharStyle 
  onCloseSwatch={() => { currentEditingSwatch = undefined; }}
/>

<style>
  :global(body) { margin: 0; padding: 0; background-color: #1e1e1e; color: #ccc; font-family: sans-serif; overflow: hidden; }
  main { display: flex; flex-direction: column; height: 100vh; position: relative; width: 100vw; }
  .content-area { flex: 1; display: flex; margin-left: 40px; overflow: hidden; }
</style>
