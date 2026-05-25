// Usage: node render_reference.mjs [theme]
// theme: default | dark | forest | neutral  (default if omitted)
import puppeteer from 'puppeteer';
import { readFileSync, writeFileSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

const themeName = process.argv[2] || 'default';
const validThemes = ['default', 'dark', 'forest', 'neutral'];
if (!validThemes.includes(themeName)) {
  console.error(`Unknown theme: ${themeName}. Valid: ${validThemes.join(', ')}`);
  process.exit(1);
}

const REF_DIR = themeName === 'default'
  ? join(__dirname, 'ref')
  : join(__dirname, `ref_${themeName}`);

const CORPUS = JSON.parse(readFileSync(join(__dirname, 'corpus', 'corpus.json'), 'utf8'));
const MERMAID_BUNDLE = join(__dirname, 'node_modules', 'mermaid', 'dist', 'mermaid.min.js');

mkdirSync(REF_DIR, { recursive: true });

const browser = await puppeteer.launch({
  headless: true,
  args: ['--no-sandbox', '--disable-setuid-sandbox'],
});

const page = await browser.newPage();
await page.setViewport({ width: 2000, height: 2000 });
await page.setContent('<!DOCTYPE html><html><body></body></html>');
await page.addScriptTag({ path: MERMAID_BUNDLE });

await page.evaluate((theme) => {
  window.mermaid.initialize({
    startOnLoad: false,
    theme,
    securityLevel: 'loose',
    fontFamily: 'Arial, sans-serif',
    // htmlLabels:false → Mermaid emits <text> instead of <foreignObject>, matching
    // ariel-rs's SVG output and allowing pixel-comparable rendering via Puppeteer/resvg.
    htmlLabels: false,
  });
}, themeName);

let passed = 0;
let failed = 0;
let counter = 0;

for (const [name, text] of Object.entries(CORPUS)) {
  counter++;
  const id = `mermaid-svg-${counter}`;
  try {
    const svg = await page.evaluate(async (diagramId, diagramText) => {
      const container = document.createElement('div');
      container.id = `container-${diagramId}`;
      document.body.appendChild(container);
      const { svg } = await window.mermaid.render(diagramId, diagramText, container);
      container.remove();
      return svg;
    }, id, text);

    writeFileSync(join(REF_DIR, `${name}.svg`), svg, 'utf8');
    console.log(`  ✓  ${name}`);
    passed++;
  } catch (err) {
    console.error(`  ✗  ${name}: ${err.message.split('\n')[0]}`);
    failed++;
  }
}

await browser.close();
console.log(`\n${passed} rendered, ${failed} failed  [theme: ${themeName}]`);
console.log(`SVGs saved to: ${REF_DIR}`);
if (failed > 0) process.exit(1);
