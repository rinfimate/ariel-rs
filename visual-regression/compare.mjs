import { readFileSync, writeFileSync, readdirSync, existsSync } from 'fs';
import { join, dirname, basename } from 'path';
import { fileURLToPath } from 'url';
import { PNG } from 'pngjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
// Usage: node compare.mjs [ref_dir] [rust_dir]
// Defaults to ref/ and rust/
const REF_DIR  = join(__dirname, process.argv[2] || 'ref');
const RUST_DIR = join(__dirname, process.argv[3] || 'rust');

// Thresholds (same as smiles-rust)
const PASS_MAD   = 0.02;   // <2%  mean absolute pixel difference
const WARN_MAD   = 0.05;   // <5%
const PASS_PDIFF = 0.05;   // <5%  of pixels differ by >10/255
const WARN_PDIFF = 0.15;   // <15%

function decodePng(path) {
  const buf = readFileSync(path);
  return PNG.sync.read(buf);
}

// Bilinear-sampled pixel read from a PNG at fractional coordinates
function samplePixel(data, w, h, x, y, c) {
  const x0 = Math.min(Math.floor(x), w - 1);
  const y0 = Math.min(Math.floor(y), h - 1);
  const x1 = Math.min(x0 + 1, w - 1);
  const y1 = Math.min(y0 + 1, h - 1);
  const fx = x - x0, fy = y - y0;
  const i00 = (y0 * w + x0) * 4 + c;
  const i10 = (y0 * w + x1) * 4 + c;
  const i01 = (y1 * w + x0) * 4 + c;
  const i11 = (y1 * w + x1) * 4 + c;
  return data[i00] * (1-fx)*(1-fy) + data[i10] * fx*(1-fy)
       + data[i01] * (1-fx)*fy     + data[i11] * fx*fy;
}

// Rescale src PNG data to (tw, th) using bilinear sampling
function rescale(src, sw, sh, tw, th) {
  const out = new Uint8Array(tw * th * 4);
  const xr = sw / tw, yr = sh / th;
  for (let ty = 0; ty < th; ty++) {
    for (let tx = 0; tx < tw; tx++) {
      const sx = tx * xr, sy = ty * yr;
      for (let c = 0; c < 4; c++) {
        out[(ty * tw + tx) * 4 + c] = Math.round(samplePixel(src, sw, sh, sx, sy, c));
      }
    }
  }
  return out;
}

function compare(refPng, rustPng) {
  const { width: rw, height: rh, data: rd } = refPng;
  const { width: uw, height: uh, data: ud } = rustPng;

  const sizeMismatch = (rw !== uw || rh !== uh)
    ? `ref ${rw}x${rh} vs rust ${uw}x${uh}` : null;

  // If sizes differ by more than 50% in either dimension, bail out
  if (Math.abs(rw - uw) / rw > 0.5 || Math.abs(rh - uh) / rh > 0.5) {
    return { mad: 1, pdiff: 1, sizeMismatch };
  }

  // Use the reference dimensions as target; rescale rust if needed
  const tw = rw, th = rh;
  const cmpRd = rd;
  const cmpUd = (rw !== uw || rh !== uh) ? rescale(ud, uw, uh, tw, th) : ud;

  const n = tw * th * 4;   // RGBA channels at target size
  let sumAbsDiff = 0;
  let diffPixels = 0;
  const totalPixels = tw * th;

  for (let i = 0; i < n; i += 4) {
    let maxChanDiff = 0;
    let chanDiffSum = 0;
    for (let c = 0; c < 4; c++) {
      const d = Math.abs(cmpRd[i + c] - cmpUd[i + c]);
      chanDiffSum += d;
      if (d > maxChanDiff) maxChanDiff = d;
    }
    sumAbsDiff += chanDiffSum / 4;
    if (maxChanDiff > 10) diffPixels++;
  }

  const mad   = sumAbsDiff / totalPixels / 255;
  const pdiff = diffPixels / totalPixels;
  return { mad, pdiff, sizeMismatch };
}

function classify(mad, pdiff) {
  if (mad <= PASS_MAD && pdiff <= PASS_PDIFF) return 'PASS';
  if (mad <= WARN_MAD && pdiff <= WARN_PDIFF) return 'WARN';
  return 'FAIL';
}

const refPngs = readdirSync(REF_DIR).filter(f => f.endsWith('.png')).sort();

if (refPngs.length === 0) {
  console.error('No reference PNGs found. Run: node svg_to_png.mjs ref');
  process.exit(1);
}

const results = [];
let pass = 0, warn = 0, fail = 0, missing = 0;

for (const file of refPngs) {
  const name = basename(file, '.png');
  const refPath  = join(REF_DIR, file);
  const rustPath = join(RUST_DIR, file);

  if (!existsSync(rustPath)) {
    console.log(`  -  ${name}: MISSING (no rust output)`);
    missing++;
    results.push({ name, status: 'MISSING' });
    continue;
  }

  const refPng  = decodePng(refPath);
  const rustPng = decodePng(rustPath);
  const { mad, pdiff, sizeMismatch } = compare(refPng, rustPng);
  const status = classify(mad, pdiff);

  const madPct   = (mad   * 100).toFixed(3);
  const pdiffPct = (pdiff * 100).toFixed(3);
  const sizeNote = sizeMismatch ? ` [${sizeMismatch}]` : '';
  const detail   = `MAD ${madPct}%  PDIFF ${pdiffPct}%${sizeNote}`;

  const icon = status === 'PASS' ? '✓' : status === 'WARN' ? '~' : '✗';
  console.log(`  ${icon}  ${name}: ${status}  (${detail})`);

  const refW = refPng.width, refH = refPng.height;
  const rustW = rustPng.width, rustH = rustPng.height;
  results.push({ name, status, mad: parseFloat(madPct), pdiff: parseFloat(pdiffPct), sizeMismatch,
                 refW, refH, rustW, rustH });

  if (status === 'PASS') pass++;
  else if (status === 'WARN') warn++;
  else fail++;
}

console.log(`\nPASS ${pass}  WARN ${warn}  FAIL ${fail}  MISSING ${missing}`);

writeFileSync(join(__dirname, 'report.json'), JSON.stringify({
  summary: { pass, warn, fail, missing, total: refPngs.length },
  thresholds: { PASS_MAD, WARN_MAD, PASS_PDIFF, WARN_PDIFF },
  results,
}, null, 2));

console.log('Report written to report.json');
if (fail > 0) process.exit(1);
