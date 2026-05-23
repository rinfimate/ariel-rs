// Usage: node svg_to_png.mjs <dir>
// Converts all *.svg files in visual-regression/<dir>/ to *.png.
//
// Uses resvg for all directories (fast, consistent rendering).
// Note: SVGs using foreignObject may be skipped if resvg cannot parse them.

import { Resvg } from '@resvg/resvg-js';
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

let converted = 0;
let failed = 0;

for (const file of svgFiles) {
  const svgPath = join(dir, file);
  const pngPath = join(dir, file.replace(/\.svg$/, '.png'));
  // Replace HTML entities not valid in XML (resvg parses SVG as XML)
  const svgRaw = readFileSync(svgPath, 'utf8')
    .replace(/&nbsp;/g, '&#160;')
    .replace(/&mdash;/g, '&#8212;')
    .replace(/&ndash;/g, '&#8211;')
    .replace(/&hellip;/g, '&#8230;')
    .replace(/&amp;nbsp;/g, '&#160;');
  const svg = Buffer.from(svgRaw, 'utf8');

  try {
    const resvg = new Resvg(svg, {
      fitTo: { mode: 'width', value: 1200 },
      font: { loadSystemFonts: true },
    });
    const pngData = resvg.render().asPng();
    writeFileSync(pngPath, pngData);
    console.log(`  ${basename(file)} => ${basename(pngPath)}`);
    converted++;
  } catch (err) {
    console.error(`  SKIP ${basename(file)}: ${err.message.split('\n')[0]}`);
    failed++;
  }
}

console.log(`\n${converted} files converted in ${TARGET}/`);