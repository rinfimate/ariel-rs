// Render all corpus diagrams for all 4 themes and rasterize with resvg.
// Output: visual-regression/rsvg_output/<theme>/<name>.svg and <name>.png

import { Resvg } from './node_modules/@resvg/resvg-js/index.js';
import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

const THEMES = [
  { name: 'default', dir: 'rust' },
  { name: 'dark',    dir: 'rust_dark' },
  { name: 'forest',  dir: 'rust_forest' },
  { name: 'neutral', dir: 'rust_neutral' },
];

const corpus = JSON.parse(readFileSync(join(__dirname, 'corpus', 'corpus.json'), 'utf8'));
const names = Object.keys(corpus);

let totalOk = 0, totalFailed = 0;

for (const theme of THEMES) {
  const svgDir = join(__dirname, theme.dir);
  const outDir = join(__dirname, 'rsvg_output', theme.name);
  mkdirSync(outDir, { recursive: true });

  let ok = 0, failed = 0;
  console.log(`\n── ${theme.name} ──`);

  for (const name of names) {
    const svgPath = join(svgDir, name + '.svg');
    if (!existsSync(svgPath)) { console.log(`  SKIP ${name}`); continue; }

    const svgData = readFileSync(svgPath, 'utf8');
    writeFileSync(join(outDir, name + '.svg'), svgData);

    try {
      const resvg = new Resvg(svgData, {
        fitTo: { mode: 'width', value: 1200 },
        font: { loadSystemFonts: true },
      });
      const png = resvg.render().asPng();
      writeFileSync(join(outDir, name + '.png'), png);
      ok++;
    } catch (e) {
      console.log(`  ERR ${name}: ${e.message}`);
      failed++;
    }
  }

  console.log(`  ${ok} ok, ${failed} failed → ${outDir}`);
  totalOk += ok; totalFailed += failed;
}

console.log(`\nTotal: ${totalOk} rendered, ${totalFailed} failed`);
