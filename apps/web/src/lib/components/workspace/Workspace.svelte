<script lang="ts">
  import { docStore } from "../../stores/document.svelte";
  import { uiStore } from "../../stores/ui.svelte";
  import { prefsStore } from "../../stores/prefs.svelte";
  import { convertUnit } from "../../utils/geometry";
  import { getSwatchColor } from "../../utils/color";
  import type { Page, Frame, Guide, ImageFrame, Orientation } from "../../types";

  let { 
    onPageMouseDown, 
    onFrameMouseDown, 
    onPortMouseDown, 
    onRulerMouseDown, 
    onGuideMouseDown,
    onResizeMouseDown,
    onContentHandleMouseDown
  }: {
    onPageMouseDown: (e: MouseEvent, page: Page, el: HTMLElement) => void,
    onFrameMouseDown: (e: MouseEvent, frame: Frame) => void,
    onPortMouseDown: (e: MouseEvent, id: string) => void,
    onRulerMouseDown: (e: MouseEvent, orientation: Orientation) => void,
    onGuideMouseDown: (e: MouseEvent, page: Page, guide: Guide) => void,
    onResizeMouseDown: (e: MouseEvent, frame: Frame, handle: string) => void,
    onContentHandleMouseDown: (e: MouseEvent, image: ImageFrame, handle: string) => void
  } = $props();
  function focusOnMount(node: HTMLElement) {
    setTimeout(() => node.focus(), 10);
  }
</script>

{#snippet ParentContent(page: Page, parentId: string, pageIdxInSpread: number)}
  {#each docStore.doc.parent_pages.filter(p => p.id === parentId) as parent}
    {#if parent.based_on_id}
      {@render ParentContent(page, parent.based_on_id, pageIdxInSpread)}
    {/if}
    {#if parent.spread.pages[pageIdxInSpread]}
      <div class="parent-content">
        {#each parent.spread.pages[pageIdxInSpread].frames as frame}
          <div 
            class="frame parent-frame" 
            onclick={(e) => { e.stopPropagation(); docStore.overrideParentFrame(page, frame); }} 
            style="left: {frame.x}px; top: {frame.y}px; width: {frame.width}px; height: {frame.height}px; background: {frame.fill_color ? getSwatchColor(frame.fill_color, docStore.doc) : 'transparent'}; border-width: {frame.stroke_width}px; border-color: {frame.stroke_color ? getSwatchColor(frame.stroke_color, docStore.doc) : 'transparent'};"
          >
             {#if frame.data.Text}{frame.data.Text.content}{/if}
             <div class="override-hint">Überschreiben</div>
          </div>
        {/each}
      </div>
    {/if}
  {/each}
{/snippet}

<div class="workspace-container" onmousedown={() => uiStore.resetSelection()}>
  <div class="ruler top-ruler" onmousedown={(e) => onRulerMouseDown(e, 'Vertical')}>
    {#each Array(20) as _, i}
      <div class="ruler-tick" style="left: {i * 100 * uiStore.zoom}px">
        {convertUnit(i * 100, 'pt', prefsStore.prefs.default_unit).toFixed(0)}
      </div>
    {/each}
  </div>
  <div class="ruler left-ruler" onmousedown={(e) => onRulerMouseDown(e, 'Horizontal')}>
    {#each Array(20) as _, i}
      <div class="ruler-tick" style="top: {i * 100 * uiStore.zoom}px">
        {convertUnit(i * 100, 'pt', prefsStore.prefs.default_unit).toFixed(0)}
      </div>
    {/each}
  </div>
  
  <div class="workspace" style="--zoom: {uiStore.zoom}">
    {#each docStore.doc.spreads as spread}
      <div class="spread">
        {#each spread.pages as page, pageIdxInSpread}
          <div 
            class="page" 
            style="width: {page.width}px; height: {page.height}px;" 
            onmousedown={(e) => { 
              const el = e.currentTarget as HTMLElement;
              e.stopPropagation(); 
              onPageMouseDown(e, page, el); 
            }}
          >
            {#if uiStore.snapX !== null}<div class="snap-guide vertical" style="left: {uiStore.snapX}px"></div>{/if}
            {#if uiStore.snapY !== null}<div class="snap-guide horizontal" style="top: {uiStore.snapY}px"></div>{/if}
            
            <div class="margin-box" style="top: {page.margins.top}px; bottom: {page.margins.bottom}px; left: {page.margins.inside}px; right: {page.margins.outside}px;">
              {#if page.column_count > 1}
                <div class="column-gutters">
                  {#each Array(page.column_count - 1) as _, i}
                    {@const colW = (page.width - page.margins.inside - page.margins.outside - (page.column_count - 1) * page.gutter_width) / page.column_count}
                    <div class="gutter-guide" style="left: {(i + 1) * colW + i * page.gutter_width}px; width: {page.gutter_width}px;"></div>
                  {/each}
                </div>
              {/if}
            </div>

            {#if docStore.doc.baseline_grid.visible}
              <div class="baseline-grid">
                {#each Array(Math.floor(page.height / docStore.doc.baseline_grid.line_height)) as _, i}
                  <div class="baseline-line" style="top: {docStore.doc.baseline_grid.offset + i * docStore.doc.baseline_grid.line_height}px"></div>
                {/each}
              </div>
            {/if}

            {#each page.guides as guide}
              <div 
                class="guide" 
                class:horizontal={guide.orientation === 'Horizontal'} 
                class:vertical={guide.orientation === 'Vertical'} 
                style="{guide.orientation === 'Horizontal' ? 'top' : 'left'}: {guide.position}px;" 
                onmousedown={(e) => onGuideMouseDown(e, page, guide)}
              ></div>
            {/each}
            
            {#if page.applied_parent_id}
              {@render ParentContent(page, page.applied_parent_id, pageIdxInSpread)}
            {/if}

            {#each page.frames as frame (frame.id)}
              {@const layer = docStore.doc.layers.find(l => l.id === frame.layer_id)}
              {#if !layer || layer.visible}
                <div 
                  class="frame frame-container" 
                  class:selected={uiStore.selectedFrameIds.includes(frame.id)} 
                  class:content-mode={uiStore.isContentMode && uiStore.selectedFrameIds.includes(frame.id)} 
                  onmousedown={(e) => onFrameMouseDown(e, frame)} 
                  ondblclick={() => { if (frame.data.Text) uiStore.isContentMode = true; }}
                  style="left: {frame.x}px; top: {frame.y}px; width: {frame.width}px; height: {frame.height}px; transform: rotate({frame.rotation}deg); --layer-color: {layer?.color ?? '#007acc'};"
                >
                  <div 
                    class="frame-content" 
                    style="background: {frame.fill_color ? getSwatchColor(frame.fill_color, docStore.doc) : 'transparent'}; border: {frame.stroke_width}px solid {frame.stroke_color ? getSwatchColor(frame.stroke_color, docStore.doc) : 'transparent'}; {frame.data.Text ? `font-size: ${frame.data.Text.font_size_override ?? (docStore.doc.styles.paragraph_styles.find(s => s.name === frame.data.Text?.paragraph_style) || docStore.doc.styles.paragraph_styles[0]).font_size ?? 12}px;` : ''}"
                  >
                    {#if frame.data.Text}
                      {#if uiStore.isContentMode && uiStore.selectedFrameIds.includes(frame.id)}
                        <textarea
                          class="inline-editor"
                          bind:value={frame.data.Text.content}
                          oninput={() => { if (frame.data.Text?.frame_type === 'Point') docStore.convertTextFrameType(frame, 'Point'); docStore.markModified(); }}
                          onmousedown={(e) => e.stopPropagation()}
                          onkeydown={(e) => { if (e.key === 'Escape') { e.stopPropagation(); uiStore.isContentMode = false; } }}
                          use:focusOnMount
                        ></textarea>
                      {:else}
                        <div class="text-content-wrapper">
                          {frame.data.Text.content}
                        </div>
                      {/if}
                      
                      <!-- Overflow Indicator -->
                      {#if frame.data.Text.frame_type === 'Area' && !uiStore.isContentMode}
                        <div class="overflow-indicator" title="Textüberlauf!"></div>
                      {/if}
                    {:else if frame.data.Image}
                      <div class="image-content" style="transform: translate({frame.data.Image.content_x}px, {frame.data.Image.content_y}px) scale({frame.data.Image.content_scale_x}, {frame.data.Image.content_scale_y});">
                        <div class="image-placeholder">{#if frame.data.Image.asset_path}Bild{:else}Kein Bild{/if}</div>
                      </div>
                    {:else if frame.data.Group}
                      <div class="group-content">
                        {#each frame.data.Group.frames as c}
                          <div class="frame-preview" style="left: {c.x}px; top: {c.y}px; width: {c.width}px; height: {c.height}px;"></div>
                        {/each}
                      </div>
                    {/if}
                  </div>
                </div>
              {/if}
            {/each}

            <!-- Selection UI Layer (Decoupled from content loops) -->
            {#each page.frames as frame (frame.id)}
              {#if uiStore.selectedFrameIds.length === 1 && uiStore.selectedFrameIds[0] === frame.id}
                <div 
                  class="selection-overlay" 
                  class:content-mode={uiStore.isContentMode}
                  style="left: {frame.x}px; top: {frame.y}px; width: {frame.width}px; height: {frame.height}px; transform: rotate({frame.rotation}deg); --layer-color: {docStore.doc.layers.find(l => l.id === frame.layer_id)?.color ?? '#007acc'};"
                >
                  {#if !uiStore.isContentMode}
                    {#each ['n','s','e','w','nw','ne','sw','se'] as h}
                      <div class="resize-handle {h}" onmousedown={(e) => onResizeMouseDown(e, frame, h)}></div>
                    {/each}
                  {:else if frame.data.Image}
                    <div class="content-handles" style="transform: translate({frame.data.Image.content_x}px, {frame.data.Image.content_y}px);">
                      {#each ['nw','ne','sw','se'] as h}
                        <div class="content-handle {h}" onmousedown={(e) => onContentHandleMouseDown(e, frame.data.Image!, h)}></div>
                      {/each}
                    </div>
                  {/if}
                  {#if frame.data.Text}
                    <div class="port in-port"></div>
                    <div class="port out-port" onmousedown={(e) => onPortMouseDown(e, frame.id)}></div>
                  {/if}
                </div>
              {/if}
            {/each}
          </div>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .workspace-container { 
    flex: 1; overflow: auto; background: #181818; position: relative; padding: 60px; 
    user-select: none; -webkit-user-select: none;
  }
  .workspace { display: flex; flex-direction: column; align-items: flex-start; gap: 50px; }
  .spread { display: flex; gap: 2px; background: #000; padding: 2px; box-shadow: 0 10px 30px rgba(0,0,0,0.5); transform: scale(var(--zoom)); transform-origin: top center; }
  .page { background: white; position: relative; color: black; }
  .margin-box { position: absolute; border: 1px solid #ff00ff22; pointer-events: none; }
  .column-gutters { position: absolute; top:0; left:0; right:0; bottom:0; display:flex; }
  .gutter-guide { position: absolute; top:0; bottom:0; background: #00ffff08; border-left: 1px solid #00ffff11; border-right: 1px solid #00ffff11; }
  .baseline-grid { position: absolute; top:0; left:0; right:0; bottom:0; pointer-events: none; }
  .baseline-line { position: absolute; left: 0; right: 0; height: 1px; border-top: 1px solid #44ffff11; }
  .guide { position: absolute; z-index: 10; cursor: grab; background: #00ffff44; }
  .guide.horizontal { left: 0; right: 0; height: 1px; }
  .guide.vertical { top: 0; bottom: 0; width: 1px; }
  .ruler { position: absolute; background: #2d2d2d; border: 1px solid #111; z-index: 20; color: #888; font-size: 9px; cursor: crosshair; }
  .top-ruler { top:0; left:0; right:0; height:20px; }
  .left-ruler { left:0; top:0; bottom:0; width:20px; }
  .ruler-tick { position: absolute; padding: 2px; }
  .snap-guide { position: absolute; border: 1px dashed #ff00ff; z-index: 100; pointer-events: none; }
  .snap-guide.horizontal { left: 0; right: 0; }
  .snap-guide.vertical { top: 0; bottom: 0; }
  .frame-container { 
    position: absolute; 
    box-sizing: border-box; 
    z-index: 5;
    pointer-events: auto; /* Catch clicks for selection */
  }
  .frame-content {
    position: absolute;
    top: 0; left: 0; width: 100%; height: 100%;
    overflow: hidden; 
    background: rgba(128, 128, 128, 0.05); 
    color: #1a1a1a;
    pointer-events: none; /* Let clicks go to the container handler */
    z-index: 1;
    outline: 1px solid rgba(128, 128, 128, 0.2); /* Subtle default outline */
    outline-offset: -1px;
  }
  .frame-container.selected .frame-content { 
    z-index: 10;
  }
  .frame-container.content-mode .frame-content { 
    z-index: 11;
  }
  .text-content-wrapper, .inline-editor { 
    width: 100%; height: 100%; 
    white-space: pre-wrap; word-break: break-word; 
    text-align: left;
    font-family: sans-serif; 
    line-height: 1.2;
    padding: 0; margin: 0;
    box-sizing: border-box;
    display: block;
    overflow: hidden;
    /* Professional Typographic Hardening */
    text-rendering: optimizeLegibility !important;
    -webkit-font-smoothing: antialiased !important;
    -moz-osx-font-smoothing: grayscale !important;
    font-variant-ligatures: common-ligatures !important;
    letter-spacing: normal !important;
    font-feature-settings: "kern" 1, "liga" 1 !important;
  }
  .text-content-wrapper { 
    pointer-events: none; 
    color: #1a1a1a;
  }
  .inline-editor { 
    background: transparent; border: none !important; outline: none !important; 
    box-shadow: none !important; -webkit-appearance: none; appearance: none;
    color: #1a1a1a !important; 
    font-size: inherit; 
    resize: none !important;
    user-select: text !important; /* Allow text selection while editing */
    -webkit-user-select: text !important;
    pointer-events: auto; /* Always interactive */
  }
  .inline-editor:focus {
    outline: none !important;
    border: none !important;
    box-shadow: none !important;
  }
  .selection-overlay {
    position: absolute;
    pointer-events: none;
    z-index: 20;
    outline: 1px solid var(--layer-color, #007acc);
    outline-offset: -1px;
  }
  .selection-overlay.content-mode {
    outline: 1px solid orange;
  }
  .resize-handle { 
    position: absolute; 
    width: 8px; height: 8px; 
    background: white; 
    border: 1px solid var(--layer-color, #007acc);
    pointer-events: auto; /* Handles are interactive */
  }
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
</style>
