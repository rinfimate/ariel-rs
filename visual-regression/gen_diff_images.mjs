/**
 * gen_diff_images.mjs — generate side-by-side diff PNGs for failing diagrams.
 *
 * Usage: node gen_diff_images.mjs [ref_dir] [rust_dir] [out_dir]
 * Defaults: ref ref diff
 *
 * For each pair where PDIFF > threshold, generates a side-by-side image:
 *   left  = ref PNG
 *   right = rust PNG
 * Saves to out_dir/{name}.png
 */
import { readFileSync, writeFileSync, mkdirSync, existsSync, readdirSync } from 'fs';
import { join, dirname, basename } from 'path';
import { fileURLToPath } from 'url';
import { PNG } from 'pngjs';

const __dirname = dirname(fileURLToPath(import.meta.url));

const REF_DIR  = join(__dirname, process.argv[2] || 'ref');
const RUST_DIR = join(__dirname, process.argv[3] || 'rust');
const OUT_DIR  = join(__dirname, process.argv[4] || 'diff');
const THRESHOLD_PDIFF = 0.03; // only generate diff if PDIFF > 3%

mkdirSync(OUT_DIR, { recursive: true });

const report = JSON.parse(readFileSync(join(__dirname, 'report.json'), 'utf8'));
const failures = report.results.filter(r => r.status === 'FAIL' || r.status === 'WARN');

let generated = 0;
for (const f of failures) {
  const name = f.name;
  const refPath  = join(REF_DIR,  `${name}.png`);
  const rustPath = join(RUST_DIR, `${name}.png`);
  if (!existsSync(refPath) || !existsSync(rustPath)) continue;
  if (f.pdiff < THRESHOLD_PDIFF) continue;

  const refPng  = PNG.sync.read(readFileSync(refPath));
  const rustPng = PNG.sync.read(readFileSync(rustPath));

  // Normalize both to the same height (use max)
  const maxH = Math.max(refPng.height, rustPng.height);
  const totalW = refPng.width + rustPng.width + 4; // 4px separator

  const out = new PNG({ width: totalW, height: maxH });

  // Fill with light grey (separator + background)
  for (let y = 0; y < maxH; y++) {
    for (let x = 0; x < totalW; x++) {
      const idx = (y * totalW + x) * 4;
      out.data[idx] = out.data[idx+1] = out.data[idx+2] = 200;
      out.data[idx+3] = 255;
    }
  }

  // Copy ref (left)
  for (let y = 0; y < refPng.height; y++) {
    for (let x = 0; x < refPng.width; x++) {
      const si = (y * refPng.width + x) * 4;
      const di = (y * totalW + x) * 4;
      out.data[di]   = refPng.data[si];
      out.data[di+1] = refPng.data[si+1];
      out.data[di+2] = refPng.data[si+2];
      out.data[di+3] = refPng.data[si+3];
    }
  }

  // Copy rust (right, offset by refWidth + 4)
  const rx = refPng.width + 4;
  for (let y = 0; y < rustPng.height; y++) {
    for (let x = 0; x < rustPng.width; x++) {
      const si = (y * rustPng.width + x) * 4;
      const di = (y * totalW + (rx + x)) * 4;
      out.data[di]   = rustPng.data[si];
      out.data[di+1] = rustPng.data[si+1];
      out.data[di+2] = rustPng.data[si+2];
      out.data[di+3] = rustPng.data[si+3];
    }
  }

  const outPath = join(OUT_DIR, `${name}.png`);
  writeFileSync(outPath, PNG.sync.write(out));
  console.log(`  ${name}.png  (PDIFF ${(f.pdiff*100).toFixed(1)}%)`);
  generated++;
}
console.log(`\nGenerated ${generated} diff images in ${OUT_DIR}/`);
