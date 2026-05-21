// Renders SVG files to PNG using a headless browser so embedded CSS is applied.
// Reuses a single page — injects each SVG via JS, no page reload per file.
// Usage: node svg_to_png_browser.mjs ref
import puppeteer from 'puppeteer';
import { readFileSync, writeFileSync, readdirSync } from 'fs';
import { join, dirname, basename } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const TARGET = process.argv[2] ?? 'ref';
const TARGET_W = 1200;

const dir = join(__dirname, TARGET);
const svgFiles = readdirSync(dir).filter(f => f.endsWith('.svg')).sort();

const browser = await puppeteer.launch({
  headless: true,
  args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-web-security'],
});
const page = await browser.newPage();

// Load a blank white page once — all SVGs injected via evaluate()
await page.setContent('<!DOCTYPE html><html><head><style>*{margin:0;padding:0}body{background:white}</style></head><body></body></html>');

let converted = 0;
let failed = 0;

for (const file of svgFiles) {
  const svgPath = join(dir, file);
  const pngPath = join(dir, file.replace(/\.svg$/, '.png'));
  const svgText = readFileSync(svgPath, 'utf8');

  // Compute render height from viewBox
  const vbMatch = svgText.match(/viewBox="([^"]+)"/);
  let renderH = TARGET_W;
  if (vbMatch) {
    const parts = vbMatch[1].trim().split(/\s+/).map(Number);
    const vbW = parts[2], vbH = parts[3];
    if (vbW > 0 && vbH > 0 && isFinite(vbH / vbW)) {
      renderH = Math.ceil(TARGET_W * vbH / vbW);
    }
  }
  renderH = Math.max(Math.min(renderH, 20000), 50);

  try {
    await page.setViewport({ width: TARGET_W, height: renderH, deviceScaleFactor: 1 });

    // Inject SVG directly — no network wait
    await page.evaluate((svg, w, h) => {
      document.body.innerHTML = svg;
      const el = document.body.querySelector('svg');
      if (el) {
        el.style.cssText = `display:block;width:${w}px;height:${h}px;`;
        el.setAttribute('width', w);
        el.setAttribute('height', h);
      }
    }, svgText, TARGET_W, renderH);

    const png = await page.screenshot({
      clip: { x: 0, y: 0, width: TARGET_W, height: renderH },
      type: 'png',
    });

    writeFileSync(pngPath, png);
    console.log(`  ${basename(file)} => ${basename(pngPath)}`);
    converted++;
  } catch (err) {
    console.error(`  SKIP ${basename(file)}: ${err.message.split('\n')[0]}`);
    failed++;
  }
}

await browser.close();
console.log(`\n${converted} files converted in ${TARGET}/`);
