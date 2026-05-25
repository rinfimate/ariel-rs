// Render reference SVGs with htmlLabels: false (no foreignObject).
// Tests: does our production output (no foreignObject) match Mermaid when
// Mermaid is configured to also skip foreignObject?
import puppeteer from 'puppeteer';
import { readFileSync, writeFileSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const CORPUS = join(__dirname, 'grammar_corpus', 'grammar_corpus.json');
const MERMAID = join(__dirname, 'node_modules', 'mermaid', 'dist', 'mermaid.min.js');
const OUT = join(__dirname, 'grammar', 'ref_default_nohtml');
mkdirSync(OUT, { recursive: true });

const corpus = JSON.parse(readFileSync(CORPUS, 'utf8'));

const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox', '--disable-setuid-sandbox'] });
const page = await browser.newPage();
await page.setViewport({ width: 2000, height: 2000 });
await page.setContent('<!DOCTYPE html><html><body></body></html>');
await page.addScriptTag({ path: MERMAID });
await page.evaluate(() => window.mermaid.initialize({
  startOnLoad: false,
  theme: 'default',
  securityLevel: 'loose',
  fontFamily: 'Arial, sans-serif',
  look: 'classic',
  htmlLabels: false,
  flowchart: { htmlLabels: false },
  class: { htmlLabels: false },
  state: { htmlLabels: false },
  sequence: { htmlLabels: false },
  er: { htmlLabels: false },
}));

let ctr = 0, ok = 0, err = 0;
for (const [name, text] of Object.entries(corpus)) {
  const id = `g-${++ctr}`;
  try {
    const svg = await page.evaluate(async (id, text) => {
      const c = document.createElement('div'); c.id = `c-${id}`; document.body.appendChild(c);
      const { svg } = await window.mermaid.render(id, text, c); c.remove(); return svg;
    }, id, text);
    writeFileSync(join(OUT, name + '.svg'), svg);
    ok++;
  } catch (e) {
    console.log(`  ERR ${name}: ${e.message.split('\n')[0]}`);
    err++;
  }
}
await page.close();
await browser.close();
console.log(`${ok} ok, ${err} failed`);
