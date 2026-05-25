// Produce a visual diff PNG showing which pixels differ between ref and ours.
// Usage: node pixel_diff.mjs <ref_dir> <our_dir> <name>
// Outputs: <name>_diff.png in current dir
import { readFileSync, writeFileSync } from 'fs';
import { PNG } from 'pngjs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const [refDir, ourDir, name] = process.argv.slice(2);

if (!name) {
  console.error('Usage: node pixel_diff.mjs <ref_dir> <our_dir> <name>');
  process.exit(1);
}

const ref = PNG.sync.read(readFileSync(join(__dirname, refDir, name + '.png')));
const our = PNG.sync.read(readFileSync(join(__dirname, ourDir, name + '.png')));

const w = Math.min(ref.width, our.width);
const h = Math.min(ref.height, our.height);
const out = new PNG({ width: w, height: h });

let diffPixels = 0;
let maxDiff = 0;
for (let y = 0; y < h; y++) {
  for (let x = 0; x < w; x++) {
    const refIdx = (y * ref.width + x) * 4;
    const ourIdx = (y * our.width + x) * 4;
    const outIdx = (y * w + x) * 4;
    let chanMax = 0;
    for (let c = 0; c < 3; c++) {
      const d = Math.abs(ref.data[refIdx + c] - our.data[ourIdx + c]);
      if (d > chanMax) chanMax = d;
    }
    if (chanMax > maxDiff) maxDiff = chanMax;
    if (chanMax > 10) {
      // Red for diffs
      out.data[outIdx] = 255;
      out.data[outIdx + 1] = 0;
      out.data[outIdx + 2] = 0;
      out.data[outIdx + 3] = 255;
      diffPixels++;
    } else if (chanMax > 0) {
      // Yellow for sub-threshold diffs
      out.data[outIdx] = 255;
      out.data[outIdx + 1] = 200;
      out.data[outIdx + 2] = 0;
      out.data[outIdx + 3] = 128;
    } else {
      // White for matching - faded to show structure
      const avg = (ref.data[refIdx] + ref.data[refIdx + 1] + ref.data[refIdx + 2]) / 3;
      const fade = Math.round(220 + avg * 0.14);
      out.data[outIdx] = fade;
      out.data[outIdx + 1] = fade;
      out.data[outIdx + 2] = fade;
      out.data[outIdx + 3] = 255;
    }
  }
}

const outPath = join(__dirname, name + '_diff.png');
writeFileSync(outPath, PNG.sync.write(out));
console.log(`Diff: ${outPath}`);
console.log(`  ${diffPixels} pixels differ (>10/255), ${(diffPixels / (w * h) * 100).toFixed(2)}%`);
console.log(`  max channel diff: ${maxDiff}/255`);
console.log(`  ref ${ref.width}x${ref.height}, ours ${our.width}x${our.height}`);
