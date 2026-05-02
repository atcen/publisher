import { invoke } from "@tauri-apps/api/core";
import type { AppPreferences } from "../types";

class PrefsStore {
  prefs = $state<AppPreferences>({
    theme: "dark",
    default_unit: "pt",
    autosave_interval: 60,
    recent_files: []
  });

  async load() {
    try {
      this.prefs = await invoke<AppPreferences>("load_preferences");
    } catch (e) {
      console.error("Load prefs failed", e);
    }
  }

  async save() {
    try {
      await invoke("save_preferences", { preferences: this.prefs });
    } catch (e) {
      console.error("Save prefs failed", e);
    }
  }

  updateRecentFiles(path: string) {
    this.prefs.recent_files = [path, ...this.prefs.recent_files.filter((p: string) => p !== path)].slice(0, 10);
    this.save();
  }
}

export const prefsStore = new PrefsStore();
