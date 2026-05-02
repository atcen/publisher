import { invoke } from "@tauri-apps/api/core";
import type { Page, Frame, Pt, Guide, SnapTarget, SnapPoint } from "../types";

export async function findSnap(
  x: number, 
  y: number, 
  width: number, 
  height: number, 
  targets: SnapTarget[], 
  threshold: number
): Promise<{ x: SnapPoint | null, y: SnapPoint | null }> {
  return await invoke("find_snap", { x, y, width, height, targets, threshold });
}

export function buildSnapTargets(page: Page, selectedFrameIds: string[], baselineGrid: { visible: boolean, offset: Pt, line_height: Pt }): SnapTarget[] {
  const targets: SnapTarget[] = [];
  
  // Margins
  targets.push({ Margin: { position: page.margins.top, side: 'Top' } });
  targets.push({ Margin: { position: page.height - page.margins.bottom, side: 'Bottom' } });
  targets.push({ Margin: { position: page.margins.inside, side: 'Left' } });
  targets.push({ Margin: { position: page.width - page.margins.outside, side: 'Right' } });

  // Columns
  // (Simplified for now, could be expanded)
  
  // Guides
  page.guides.forEach(g => {
    targets.push({ Guide: { position: g.position, orientation: g.orientation } });
  });
  
  // Other Objects
  page.frames
    .filter(f => !selectedFrameIds.includes(f.id))
    .forEach(f => {
      targets.push({ Object: { position: f.x, orientation: 'Vertical', frame_id: f.id } });
      targets.push({ Object: { position: f.x + f.width / 2, orientation: 'Vertical', frame_id: f.id } });
      targets.push({ Object: { position: f.x + f.width, orientation: 'Vertical', frame_id: f.id } });
      targets.push({ Object: { position: f.y, orientation: 'Horizontal', frame_id: f.id } });
      targets.push({ Object: { position: f.y + f.height / 2, orientation: 'Horizontal', frame_id: f.id } });
      targets.push({ Object: { position: f.y + f.height, orientation: 'Horizontal', frame_id: f.id } });
    });
    
  if (baselineGrid.visible) {
    for (let y = baselineGrid.offset; y < page.height; y += baselineGrid.line_height) {
      targets.push({ Baseline: { position: y } });
    }
  }
  
  return targets;
}

export function convertUnit(val: number, from: string, to: string): number {
  const toPt = { pt: 1, mm: 2.83465, cm: 28.3465, in: 72 };
  return (val * toPt[from as keyof typeof toPt]) / toPt[to as keyof typeof toPt];
}
