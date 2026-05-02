<script lang="ts">
  import { prefsStore } from "../../stores/prefs.svelte";
  import { uiStore } from "../../stores/ui.svelte";

  function convertUnit(val: number, from: string, to: string): number {
    const toPt = { pt: 1, mm: 2.83465, cm: 28.3465, in: 72 };
    return (val * toPt[from as keyof typeof toPt]) / toPt[to as keyof typeof toPt];
  }
  
  function displayVal(val: number): string {
    return convertUnit(val, 'pt', prefsStore.prefs.default_unit).toFixed(2);
  }
</script>

<footer class="status-bar">
  <span>Bereit</span>
  <span>{displayVal(0)} {prefsStore.prefs.default_unit}</span>
  <span>{(uiStore.zoom * 100).toFixed(0)}%</span>
</footer>

<style>
  .status-bar { height: 25px; background: #007acc; color: white; display: flex; align-items: center; padding: 0 10px; font-size: 11px; gap: 20px; }
</style>
