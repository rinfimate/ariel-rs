// Full regression pipeline for a given theme.
// Usage: node run_theme_regression.mjs <theme>
// theme: default | dark | forest | neutral
//
// Steps:
//   1. Render ref SVGs from Mermaid JS (render_reference.mjs)
//   2. Convert ref SVGs to PNGs via browser (svg_to_png_browser.mjs)
//   3. Convert rust SVGs to PNGs via browser (svg_to_png_browser.mjs)
//   4. Compare (compare.mjs)
//
// NOTE: rust SVGs must be pre-generated via:
//   cargo run --bin render_corpus --release -- <theme>

import { execSync } from 'child_process';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const theme = process.argv[2] || 'default';

const refDir  = theme === 'default' ? 'ref'  : `ref_${theme}`;
const rustDir = theme === 'default' ? 'rust' : `rust_${theme}`;

function run(cmd) {
  console.log(`\n$ ${cmd}`);
  execSync(cmd, { cwd: __dirname, stdio: 'inherit' });
}

console.log(`\n=== Theme regression: ${theme} ===`);
console.log(`  ref dir : ${refDir}/`);
console.log(`  rust dir: ${rustDir}/`);

run(`node render_reference.mjs ${theme}`);
run(`node svg_to_png_browser.mjs ${refDir}`);
run(`node svg_to_png_browser.mjs ${rustDir}`);
run(`node compare.mjs ${refDir} ${rustDir}`);
