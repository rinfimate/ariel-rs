// Convert grammar ref and rust SVGs to PNGs using resvg (consistent pipeline).
import { Resvg } from '@resvg/resvg-js';
import { readFileSync, writeFileSync, readdirSync, mkdirSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const THEMES = ['default', 'dark', 'forest', 'neutral'];

function svgToPng(svgPath) {
  const raw = readFileSync(svgPath, 'utf8')
    .replace(/&nbsp;/g, '&#160;')
    .replace(/&mdash;/g, '&#8212;')
    .replace(/&ndash;/g, '&#8211;')
    .replace(/&hellip;/g, '&#8230;');
  const resvg = new Resvg(Buffer.from(raw, 'utf8'), {
    fitTo: { mode: 'width', value: 1200 },
    font: { loadSystemFonts: true },
  });
  return resvg.render().asPng();
}

let total = 0, failed = 0;

for (const theme of THEMES) {
  for (const type of ['ref', 'rust']) {
    const svgDir = join(__dirname, 'grammar', `${type}_${theme}`);
    const pngDir = join(__dirname, 'grammar', `png_${type}_${theme}`);
    mkdirSync(pngDir, { recursive: true });

    if (!existsSync(svgDir)) { console.log(`  SKIP ${type}_${theme} (no dir)`); continue; }
    const svgs = readdirSync(svgDir).filter(f => f.endsWith('.svg'));

    for (const f of svgs) {
      const pngPath = join(pngDir, f.replace(/\.svg$/, '.png'));
      try {
        const png = svgToPng(join(svgDir, f));
        writeFileSync(pngPath, png);
        total++;
      } catch (e) {
        console.error(`  SKIP ${type}_${theme}/${f}: ${e.message.split('\n')[0]}`);
        failed++;
      }
    }
    console.log(`  ${type}_${theme}: ${svgs.length} svgs → png_${type}_${theme}/`);
  }
}

console.log(`\n${total} PNGs written, ${failed} failed`);
