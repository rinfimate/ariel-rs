// Usage: node svg_to_png.mjs <dir>
// Converts all *.svg files in visual-regression/<dir>/ to *.png.
//
// Uses sharp (librsvg) — full CSS <style> support, exact viewBox dimensions.

import sharp from 'sharp';
import { readFileSync, writeFileSync, readdirSync } from 'fs';
import { join, dirname, basename } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const TARGET = process.argv[2];

if (!TARGET) {
  console.error('Usage: node svg_to_png.mjs <dir>   (e.g. ref or rust)');
  process.exit(1);
}

const dir = join(__dirname, TARGET);
const svgFiles = readdirSync(dir).filter(f => f.endsWith('.svg'));

if (svgFiles.length === 0) {
  console.log(`No SVG files found in ${dir}`);
  process.exit(0);
}

/** Extract intrinsic pixel size from an SVG string.
 *  Priority: viewBox (most reliable) → explicit width/height attrs.
 *  Returns { w, h } in whole pixels. */
function svgSize(svgText) {
  const vb = svgText.match(/viewBox="([^"]+)"/);
  if (vb) {
    const parts = vb[1].trim().split(/[\s,]+/).map(Number);
    if (parts.length === 4) {
      const w = Math.round(Math.abs(parts[2]));
      const h = Math.round(Math.abs(parts[3]));
      if (w > 0 && h > 0) return { w, h };
    }
  }
  // Fallback: explicit width/height (px values only)
  const wm = svgText.match(/\bwidth="(\d+(?:\.\d+)?)"/);
  const hm = svgText.match(/\bheight="(\d+(?:\.\d+)?)"/);
  if (wm && hm) return { w: Math.round(+wm[1]), h: Math.round(+hm[1]) };
  return { w: 800, h: 600 }; // safe fallback
}

let converted = 0;
let failed = 0;

for (const file of svgFiles) {
  const svgPath = join(dir, file);
  const pngPath = join(dir, file.replace(/\.svg$/, '.png'));
  const svgText = readFileSync(svgPath, 'utf8');
  const { w, h } = svgSize(svgText);

  try {
    const pngData = await sharp(Buffer.from(svgText))
      .resize(w, h)
      .png()
      .toBuffer();
    writeFileSync(pngPath, pngData);
    console.log(`  ${basename(file)} (${w}x${h}) => ${basename(pngPath)}`);
    converted++;
  } catch (err) {
    console.error(`  SKIP ${basename(file)}: ${err.message.split('\n')[0]}`);
    failed++;
  }
}

console.log(`\n${converted} files converted in ${TARGET}/`);
