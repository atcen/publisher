import { invoke } from "@tauri-apps/api/core";
import type { Document, Page, Spread, Frame, ParagraphStyle, CharacterStyle, ColorSwatch, Color, AlignMode, DistributeMode, Pt, TextFrameType } from "../types";
import { prefsStore } from "./prefs.svelte";
import { uiStore } from "./ui.svelte";

class DocumentStore {
  doc = $state<Document>({
    metadata: { name: "Untitled", author: "", description: "", created_at: 0, modified_at: 0, dpi: 72, default_unit: "Point", default_bleed: { top: 0, bottom: 0, inside: 0, outside: 0 }, color_profile: "sRGB", facing_pages: true },
    fonts: [], icc_profiles: [], swatches: [{ name: "Schwarz", color: { Cmyk: { c: 0, m: 0, y: 0, k: 1 } } }, { name: "Weiß", color: { Cmyk: { c: 0, m: 0, y: 0, k: 0 } } }, { name: "Registration", color: { Cmyk: { c: 1, m: 1, y: 1, k: 1 } } }], 
    styles: { paragraph_styles: [{ name: "Standard", font_family: "Inter", font_size: 12, variation_settings: [], kerning_mode: 'Metric' }], character_styles: [{ name: "[Keines]", variation_settings: [], kerning_mode: 'Metric' }], object_styles: [] },
    spreads: [{ pages: [{ width: 595.27, height: 841.89, margins: { top: 36, bottom: 36, inside: 36, outside: 36 }, column_count: 2, gutter_width: 12, guides: [], frames: [], applied_parent_id: undefined }] }],
    parent_pages: [],
    layers: [{ id: "layer-1", name: "Ebene 1", visible: true, locked: false, color: "#007acc" }],
    baseline_grid: { line_height: 12, offset: 0, visible: false, color: "#44ffff33" }
  });

  currentFilePath = $state<string | null>(null);
  hasUnsavedChanges = $state(false);
  undoStack = $state<string[]>([]);
  redoStack = $state<string[]>([]);

  pushToUndo() {
    this.undoStack.push(JSON.stringify(this.doc));
    if (this.undoStack.length > 50) this.undoStack.shift();
    this.redoStack = [];
  }

  undo() {
    if (this.undoStack.length > 0) {
      this.redoStack.push(JSON.stringify(this.doc));
      this.doc = JSON.parse(this.undoStack.pop()!);
      this.hasUnsavedChanges = true;
    }
  }

  redo() {
    if (this.redoStack.length > 0) {
      this.undoStack.push(JSON.stringify(this.doc));
      this.doc = JSON.parse(this.redoStack.pop()!);
      this.hasUnsavedChanges = true;
    }
  }

  markModified() {
    this.hasUnsavedChanges = true;
    // Svelte 5 $state is deeply reactive. Root object swap is not needed 
    // and can break object identity for active interactions.
  }

  reorganizeSpreads() {
    const allPages: Page[] = this.doc.spreads.flatMap(s => s.pages);
    const newSpreads: Spread[] = [];
    let current: Page[] = [];
    for (let i = 0; i < allPages.length; i++) {
      if (this.doc.metadata.facing_pages) {
        if (i === 0) newSpreads.push({ pages: [allPages[i]] });
        else {
          current.push(allPages[i]);
          if (current.length === 2) {
            newSpreads.push({ pages: current });
            current = [];
          }
        }
      } else {
        newSpreads.push({ pages: [allPages[i]] });
      }
    }
    if (current.length > 0) newSpreads.push({ pages: current });
    this.doc.spreads = newSpreads;
    this.markModified();
  }

  async open() {
    try {
      const resp = await invoke<string>("open_document");
      const data = JSON.parse(resp);
      const content = await invoke<string>("read_document", { filePath: data.file_path });
      this.doc = JSON.parse(content);
      this.currentFilePath = data.file_path;
      this.hasUnsavedChanges = false;
      prefsStore.updateRecentFiles(data.file_path);
      await invoke("clear_recovery_file");
    } catch (e) {
      alert("Failed to open: " + e);
    }
  }

  async openRecent(path: string) {
    try {
      const content = await invoke<string>("read_document", { filePath: path });
      this.doc = JSON.parse(content);
      this.currentFilePath = path;
      this.hasUnsavedChanges = false;
      prefsStore.updateRecentFiles(path);
      await invoke("clear_recovery_file");
    } catch (e) {
      alert("Failed to open recent: " + e);
    }
  }

  async save() {
    try {
      if (!this.currentFilePath) {
        await this.saveAs();
        return;
      }
      this.doc.metadata.modified_at = Math.floor(Date.now() / 1000);
      const documentJson = JSON.stringify(this.doc, null, 2);
      await invoke<string>("save_document_file", { filePath: this.currentFilePath, documentJson });
      this.hasUnsavedChanges = false;
      await invoke("clear_recovery_file");
    } catch (e) {
      alert("Failed to save: " + e);
    }
  }

  async saveAs() {
    try {
      this.doc.metadata.modified_at = Math.floor(Date.now() / 1000);
      const documentJson = JSON.stringify(this.doc, null, 2);
      const path = await invoke<string>("save_as_file", { documentJson });
      if (path) {
        this.currentFilePath = path;
        this.hasUnsavedChanges = false;
        prefsStore.updateRecentFiles(path);
        await invoke("clear_recovery_file");
      }
    } catch (e) {
      alert("Failed to save as: " + e);
    }
  }

  async createNew(settings: any) {
    this.pushToUndo();
    const pages: Page[] = Array.from({ length: settings.pages }, () => ({
      width: settings.width,
      height: settings.height,
      margins: { ...settings.margins },
      column_count: settings.columns,
      gutter_width: settings.gutter,
      guides: [],
      frames: [],
      applied_parent_id: undefined
    }));
    this.doc = {
      ...this.doc,
      metadata: { ...this.doc.metadata, name: settings.name, facing_pages: settings.facingPages },
      spreads: pages.map(p => ({ pages: [p] }))
    };
    this.reorganizeSpreads();
    this.currentFilePath = null;
    this.hasUnsavedChanges = false;
  }

  get activePage(): Page | null {
    let c = 0;
    for (const s of this.doc.spreads) {
      for (const p of s.pages) {
        if (c === uiStore.activePageIndex) return p;
        c++;
      }
    }
    return null;
  }

  get selectedFrames(): Frame[] {
    const res: Frame[] = [];
    for (const s of this.doc.spreads) {
      for (const p of s.pages) {
        for (const f of p.frames) {
          if (uiStore.selectedFrameIds.includes(f.id)) res.push(f);
        }
      }
    }
    return res;
  }

  addParentPage() {
    this.pushToUndo();
    const id = crypto.randomUUID();
    const prefix = String.fromCharCode(65 + this.doc.parent_pages.length);
    const name = `${prefix}-Elternseite`;
    const spread: Spread = { pages: [{ width: 595.27, height: 841.89, margins: { top: 36, bottom: 36, inside: 36, outside: 36 }, column_count: 2, gutter_width: 12, guides: [], frames: [], applied_parent_id: undefined }] };
    if (this.doc.metadata.facing_pages) spread.pages.push(JSON.parse(JSON.stringify(spread.pages[0])));
    this.doc.parent_pages.push({ id, name, spread });
    this.markModified();
  }

  renameParentPage(id: string, newName: string) {
    if (newName.toLowerCase().includes("master")) {
      alert("Fehler: Begriff 'Master' unzulässig.");
      return;
    }
    const p = this.doc.parent_pages.find(p => p.id === id);
    if (p) {
      this.pushToUndo();
      p.name = newName;
      this.markModified();
    }
  }

  deleteParentPage(id: string) {
    if (confirm("Löschen?")) {
      this.pushToUndo();
      this.doc.parent_pages = this.doc.parent_pages.filter(p => p.id !== id);
      for (const s of this.doc.spreads) {
        for (const p of s.pages) {
          if (p.applied_parent_id === id) p.applied_parent_id = undefined;
        }
      }
      this.markModified();
    }
  }

  addPage() {
    this.pushToUndo();
    this.doc.spreads.push({
      pages: [{ width: 595.27, height: 841.89, margins: { top: 36, bottom: 36, inside: 36, outside: 36 }, column_count: 1, gutter_width: 12, guides: [], frames: [], applied_parent_id: undefined }]
    });
    this.reorganizeSpreads();
  }

  deletePage(idx: number) {
    if (confirm("Löschen?")) {
      this.pushToUndo();
      const pages = this.doc.spreads.flatMap(s => s.pages);
      if (pages.length > 1) {
        pages.splice(idx, 1);
        this.doc.spreads = pages.map(p => ({ pages: [p] }));
        this.reorganizeSpreads();
      }
    }
  }

  addLayer() {
    this.pushToUndo();
    this.doc.layers.unshift({
      id: crypto.randomUUID(),
      name: `Ebene ${this.doc.layers.length + 1}`,
      visible: true,
      locked: false,
      color: '#007acc'
    });
    this.markModified();
  }

  moveLayer(from: number, to: number) {
    this.pushToUndo();
    const [l] = this.doc.layers.splice(from, 1);
    this.doc.layers.splice(to, 0, l);
    this.markModified();
  }

  movePage(from: number, to: number) {
    this.pushToUndo();
    const pages = this.doc.spreads.flatMap(s => s.pages);
    const [p] = pages.splice(from, 1);
    pages.splice(to, 0, p);
    this.doc.spreads = pages.map(p => ({ pages: [p] }));
    this.reorganizeSpreads();
  }

  async applyGridPreset(page: Page, preset: string) {
    try {
      this.pushToUndo();
      const updatedPage = await invoke<Page>("apply_grid_preset", { page, preset });
      Object.assign(page, updatedPage);
      this.markModified();
    } catch (e) {
      console.error("Preset failed", e);
    }
  }

  addSwatch() {
    const s: ColorSwatch = { name: "Neues Farbfeld", color: { Rgb: { r: 0, g: 0, b: 0 } } };
    this.doc.swatches.push(s);
    this.markModified();
    return s;
  }

  async convertSwatch(swatch: ColorSwatch) {
    if ('Spot' in swatch.color) return;
    try {
      swatch.color = await invoke<Color>("convert_color", { color: swatch.color });
      this.markModified();
    } catch (e) {
      console.error("Conversion failed", e);
    }
  }

  toggleSwatchType(swatch: ColorSwatch) {
    if ('Rgb' in swatch.color) swatch.color = { Cmyk: { c: 0, m: 0, y: 0, k: 0 } };
    else if ('Cmyk' in swatch.color) swatch.color = { Spot: { name: swatch.name, alternate_cmyk: [0, 0, 0, 1], tint: 1.0 } };
    else swatch.color = { Rgb: { r: 0, g: 0, b: 0 } };
    this.markModified();
  }

  addParagraphStyle() {
    const s: ParagraphStyle = { name: `Stil ${this.doc.styles.paragraph_styles.length + 1}`, variation_settings: [], kerning_mode: 'Metric' };
    this.doc.styles.paragraph_styles.push(s);
    this.markModified();
    return s;
  }

  addCharacterStyle() {
    const s: CharacterStyle = { name: `Zeichen ${this.doc.styles.character_styles.length + 1}`, variation_settings: [], kerning_mode: 'Metric' };
    this.doc.styles.character_styles.push(s);
    this.markModified();
    return s;
  }

  overrideParentFrame(page: Page, frame: Frame) {
    this.pushToUndo();
    const nf = JSON.parse(JSON.stringify(frame));
    nf.id = crypto.randomUUID();
    page.frames.push(nf);
    uiStore.selectedFrameIds = [nf.id];
    this.markModified();
  }

  async alignFrames(mode: AlignMode) {
    if (uiStore.selectedFrameIds.length < 2) return;
    try {
      const changes = await invoke<[string, Pt, Pt][]>("align_frames", { 
        frames: this.selectedFrames, 
        mode 
      });
      if (changes.length > 0) {
        this.pushToUndo();
        for (const [id, x, y] of changes) {
          const frame = this.selectedFrames.find(f => f.id === id);
          if (frame) {
            frame.x = x;
            frame.y = y;
          }
        }
        this.markModified();
      }
    } catch (e) {
      console.error("Alignment failed", e);
    }
  }

  async distributeFrames(mode: DistributeMode) {
    if (uiStore.selectedFrameIds.length < 3) return;
    try {
      const changes = await invoke<[string, Pt, Pt][]>("distribute_frames", { 
        frames: this.selectedFrames, 
        mode 
      });
      if (changes.length > 0) {
        this.pushToUndo();
        for (const [id, x, y] of changes) {
          const frame = this.selectedFrames.find(f => f.id === id);
          if (frame) {
            frame.x = x;
            frame.y = y;
          }
        }
        this.markModified();
      }
    } catch (e) {
      console.error("Distribution failed", e);
    }
  }

  convertTextFrameType(frame: Frame, newType: TextFrameType) {
    if (!frame.data.Text) return;
    this.pushToUndo();
    frame.data.Text.frame_type = newType;
    if (newType === 'Point') {
      if (!frame.data.Text.font_size_override) {
        const style = this.doc.styles.paragraph_styles.find(s => s.name === frame.data.Text!.paragraph_style) || this.doc.styles.paragraph_styles[0];
        frame.data.Text.font_size_override = style?.font_size ?? 12;
      }
      
      const content = frame.data.Text.content;
      const fontSize = frame.data.Text.font_size_override;

      // Professional Measurement Logic
      if (typeof document !== 'undefined') {
        const ruler = document.createElement('div');
        ruler.style.visibility = 'hidden';
        ruler.style.position = 'absolute';
        ruler.style.whiteSpace = 'pre-wrap';
        ruler.style.wordBreak = 'break-word';
        ruler.style.fontFamily = 'sans-serif';
        ruler.style.fontSize = `${fontSize}px`;
        ruler.style.lineHeight = '1.2';
        ruler.style.padding = '0';
        ruler.style.width = 'auto'; // Let it expand
        
        // Ensure trailing newlines are counted in height
        ruler.textContent = content.endsWith('\n') ? content + '\u200b' : (content || ' '); 
        
        document.body.appendChild(ruler);
        
        const rect = ruler.getBoundingClientRect();
        frame.width = Math.max(20, Math.ceil(rect.width) + 2); 
        frame.height = Math.ceil(rect.height);
        
        document.body.removeChild(ruler);
      } else {
        // Fallback for non-browser environments
        const lines = content.split('\n');
        const maxLen = Math.max(...lines.map(l => l.length));
        frame.width = Math.max(20, maxLen * (fontSize * 0.5));
        frame.height = lines.length * (fontSize * 1.2);
      }
    }
    this.markModified();
  }
}

export const docStore = new DocumentStore();
