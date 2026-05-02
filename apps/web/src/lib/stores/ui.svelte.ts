import type { Pt } from "../types";

class UIStore {
  activeTool = $state('select');
  zoom = $state(1);
  activePageIndex = $state(0);
  selectedFrameIds = $state<string[]>([]);
  isContentMode = $state(false);
  
  showNewDocModal = $state(false);
  showStyleEditorModal = $state(false);
  showCharStyleEditorModal = $state(false);
  showExportModal = $state(false);
  showSwatchModal = $state(false);
  showPrefsModal = $state(false);
  
  showFindBar = $state(false);
  showReplaceFields = $state(false);
  
  snapX = $state<number | null>(null);
  snapY = $state<number | null>(null);
  
  draggedLayerIndex = $state<number | null>(null);
  draggedPageIndex = $state<number | null>(null);

  resetSelection() {
    this.selectedFrameIds = [];
    this.isContentMode = false;
  }
}

export const uiStore = new UIStore();
