export type Pt = number;

export interface TextFrame {
  content: string;
  paragraph_style?: string;
  next_frame_id?: string;
  prev_frame_id?: string;
  align_to_baseline_grid: boolean;
}

export interface ImageFrame {
  asset_path: string;
  content_x: Pt;
  content_y: Pt;
  content_scale_x: number;
  content_scale_y: number;
  fitting: 'Fill' | 'Fit' | 'Stretch' | 'Original' | 'Custom';
}

export interface Frame {
  id: string;
  layer_id: string;
  x: Pt;
  y: Pt;
  width: Pt;
  height: Pt;
  rotation: number;
  fill_color?: string;
  stroke_color?: string;
  stroke_width: Pt;
  data: {
    Text?: TextFrame;
    Image?: ImageFrame;
    Shape?: { shape_type: string };
    Group?: { frames: Frame[] };
  };
}

export interface Layer {
  id: string;
  name: string;
  visible: boolean;
  locked: boolean;
  color: string;
}

export interface Page {
  width: Pt;
  height: Pt;
  margins: { top: number; bottom: number; inside: number; outside: number };
  column_count: number;
  gutter_width: number;
  guides: Guide[];
  frames: Frame[];
  applied_parent_id?: string;
}

export interface Guide {
  position: Pt;
  orientation: 'Horizontal' | 'Vertical';
  locked: boolean;
  color: string | null;
}

export interface Spread {
  pages: Page[];
}

export interface ParentPage {
  id: string;
  name: string;
  spread: Spread;
  based_on_id?: string;
}

export interface DocumentMetadata {
  name: string;
  author: string;
  description: string;
  created_at: number;
  modified_at: number;
  dpi: number;
  default_unit: "Point";
  default_bleed: { top: number; bottom: number; inside: number; outside: number };
  color_profile: string;
  facing_pages: boolean;
}

export interface FontVariationAxis {
  tag: string;
  name: string;
  min_value: number;
  max_value: number;
  default_value: number;
}

export interface FontVariationSetting {
  tag: string;
  value: number;
}

export interface FontResource {
  id: string;
  name: string;
  family: string;
  style: string;
  data: Uint8Array;
  variation_axes: FontVariationAxis[];
}

export type KerningMode = 'Metric' | 'Optical' | 'None';

export interface ParagraphStyle {
  name: string;
  based_on?: string;
  font_family?: string;
  font_size?: Pt;
  alignment?: 'Left' | 'Center' | 'Right' | 'Justify';
  variation_settings: FontVariationSetting[];
  kerning_mode?: KerningMode;
}

export interface CharacterStyle {
  name: string;
  based_on?: string;
  font_family?: string;
  font_style?: string;
  font_size?: Pt;
  variation_settings: FontVariationSetting[];
  kerning_mode?: KerningMode;
}

export interface Styles {
  paragraph_styles: ParagraphStyle[];
  character_styles: CharacterStyle[];
  object_styles: unknown[];
}

export interface BaselineGrid {
  line_height: Pt;
  offset: Pt;
  visible: boolean;
  color: string;
}

export type Color = 
  | { Rgb: { r: number; g: number; b: number } } 
  | { Cmyk: { c: number; m: number; y: number; k: number } } 
  | { Spot: { name: string; alternate_cmyk: [number, number, number, number]; tint: number } };

export interface ColorSwatch {
  name: string;
  color: Color;
}

export interface Document {
  metadata: DocumentMetadata;
  fonts: FontResource[];
  icc_profiles: unknown[];
  swatches: ColorSwatch[];
  styles: Styles;
  spreads: Spread[];
  parent_pages: ParentPage[];
  layers: Layer[];
  baseline_grid: BaselineGrid;
}

export type Orientation = 'Horizontal' | 'Vertical';
export type Side = 'Top' | 'Bottom' | 'Left' | 'Right';

export interface SnapPoint {
  position: Pt;
  target: SnapTarget;
}

export type SnapTarget = 
  | { Margin: { position: Pt, side: Side } }
  | { Column: { position: Pt, index: number } }
  | { Guide: { position: Pt, orientation: Orientation, id?: string } }
  | { Object: { position: Pt, orientation: Orientation, object_id: string } }
  | { Baseline: { position: Pt } };

export type AlignMode = 'Left' | 'Center' | 'Right' | 'Top' | 'Middle' | 'Bottom';
export type DistributeMode = 'HorizontalSpacing' | 'VerticalSpacing';

export interface AppPreferences {
  theme: string;
  default_unit: string;
  autosave_interval: number;
  recent_files: string[];
}
