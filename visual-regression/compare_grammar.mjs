// Score grammar regression: compare png_ref_<theme>/ vs png_rust_<theme>/
import { readFileSync, writeFileSync, readdirSync, existsSync } from 'fs';
import { join, dirname, basename } from 'path';
import { fileURLToPath } from 'url';
import { PNG } from 'pngjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const THEMES = ['default', 'dark', 'forest', 'neutral'];

const PASS_MAD = 0.02, WARN_MAD = 0.05;
const PASS_PDIFF = 0.05, WARN_PDIFF = 0.15;

function decodePng(path) { return PNG.sync.read(readFileSync(path)); }

function compare(refPng, rustPng) {
  const { width: rw, height: rh, data: rd } = refPng;
  const { width: uw, height: uh, data: ud } = rustPng;
  const sizeMismatch = (rw !== uw || rh !== uh) ? `ref ${rw}x${rh} vs rust ${uw}x${uh}` : null;
  if (Math.abs(rw - uw) / rw > 0.5 || Math.abs(rh - uh) / rh > 0.5) return { mad: 1, pdiff: 1, sizeMismatch };
  const n = rw * rh * 4;
  let sumAbsDiff = 0, diffPixels = 0;
  for (let i = 0; i < n; i += 4) {
    let max = 0, sum = 0;
    for (let c = 0; c < 4; c++) { const d = Math.abs(rd[i+c] - ud[i+c]); sum += d; if (d > max) max = d; }
    sumAbsDiff += sum / 4;
    if (max > 10) diffPixels++;
  }
  return { mad: sumAbsDiff / (rw*rh) / 255, pdiff: diffPixels / (rw*rh), sizeMismatch };
}

function classify(mad, pdiff) {
  if (mad <= PASS_MAD && pdiff <= PASS_PDIFF) return 'PASS';
  if (mad <= WARN_MAD && pdiff <= WARN_PDIFF) return 'WARN';
  return 'FAIL';
}

const grandSummary = {};

for (const theme of THEMES) {
  const refDir  = join(__dirname, 'grammar', `png_ref_${theme}`);
  const rustDir = join(__dirname, 'grammar', `png_rust_${theme}`);
  if (!existsSync(refDir) || !existsSync(rustDir)) { console.log(`  SKIP ${theme}`); continue; }

  const refFiles = readdirSync(refDir).filter(f => f.endsWith('.png')).sort();
  const results = [];
  let pass = 0, warn = 0, fail = 0, missing = 0;

  for (const file of refFiles) {
    const name = basename(file, '.png');
    const rustPath = join(rustDir, file);
    if (!existsSync(rustPath)) { results.push({ name, status: 'MISSING' }); missing++; continue; }
    const { mad, pdiff, sizeMismatch } = compare(decodePng(join(refDir, file)), decodePng(rustPath));
    const status = classify(mad, pdiff);
    const icon = status === 'PASS' ? '✓' : status === 'WARN' ? '~' : '✗';
    const sizeNote = sizeMismatch ? ` [${sizeMismatch}]` : '';
    if (status !== 'PASS') console.log(`  [${theme}] ${icon}  ${name}: ${status}  MAD ${(mad*100).toFixed(3)}%  PDIFF ${(pdiff*100).toFixed(3)}%${sizeNote}`);
    results.push({ name, status, mad: parseFloat((mad*100).toFixed(3)), pdiff: parseFloat((pdiff*100).toFixed(3)), sizeMismatch });
    if (status === 'PASS') pass++; else if (status === 'WARN') warn++; else fail++;
  }

  console.log(`  [${theme}] PASS ${pass}  WARN ${warn}  FAIL ${fail}  MISSING ${missing}`);
  writeFileSync(join(__dirname, 'grammar', `report_${theme}.json`), JSON.stringify({ results }, null, 2));
  grandSummary[theme] = { pass, warn, fail, missing };
}

console.log('\nSummary:');
for (const [t, s] of Object.entries(grandSummary))
  console.log(`  ${t.padEnd(8)}: PASS ${s.pass}  WARN ${s.warn}  FAIL ${s.fail}  MISSING ${s.missing}`);
