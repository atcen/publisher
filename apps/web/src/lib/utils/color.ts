import type { Document, ColorSwatch } from "../types";

export function getSwatchColor(swatchName: string, doc: Document): string {
  const s = doc.swatches.find(x => x.name === swatchName);
  if (!s) return "transparent";
  if ('Rgb' in s.color) return `rgb(${s.color.Rgb.r * 255},${s.color.Rgb.g * 255},${s.color.Rgb.b * 255})`;
  if ('Cmyk' in s.color) {
    const { c, m, y, k } = s.color.Cmyk;
    return `rgb(${255 * (1 - c) * (1 - k)},${255 * (1 - m) * (1 - k)},${255 * (1 - y) * (1 - k)})`;
  }
  if ('Spot' in s.color) {
    const { alternate_cmyk: [c, m, y, k], tint } = s.color.Spot;
    return `rgb(${255 * (1 - c * tint) * (1 - k * tint)},${255 * (1 - m * tint) * (1 - k * tint)},${255 * (1 - y * tint) * (1 - k * tint)})`;
  }
  return "gray";
}
