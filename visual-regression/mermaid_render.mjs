/**
 * CLI wrapper to render a single Mermaid diagram to SVG.
 * Usage: node mermaid_render.mjs <diagram_text_file>
 * Outputs the SVG to stdout.
 */
import puppeteer from 'puppeteer';
import { readFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const MERMAID_BUNDLE = join(__dirname, 'node_modules', 'mermaid', 'dist', 'mermaid.min.js');

const inputFile = process.argv[2];
if (!inputFile) {
  process.stderr.write('Usage: node mermaid_render.mjs <input_file>\n');
  process.exit(1);
}

const diagramText = readFileSync(inputFile, 'utf8').trim();

const browser = await puppeteer.launch({
  headless: true,
  args: ['--no-sandbox', '--disable-setuid-sandbox'],
});

const page = await browser.newPage();
await page.setViewport({ width: 2000, height: 2000 });
await page.setContent('<!DOCTYPE html><html><body></body></html>');
await page.addScriptTag({ path: MERMAID_BUNDLE });

await page.evaluate(() => {
  window.mermaid.initialize({
    startOnLoad: false,
    theme: 'default',
    securityLevel: 'loose',
    fontFamily: 'Arial, sans-serif',
  });
});

try {
  const svg = await page.evaluate(async (text) => {
    const container = document.createElement('div');
    container.id = 'mermaid-container';
    document.body.appendChild(container);
    const { svg } = await window.mermaid.render('mermaid-svg-1', text, container);
    container.remove();
    return svg;
  }, diagramText);

  process.stdout.write(svg);
} catch (err) {
  process.stderr.write('Error: ' + err.message + '\n');
  process.exit(1);
} finally {
  await browser.close();
}
