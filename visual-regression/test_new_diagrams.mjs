import puppeteer from 'puppeteer';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
const __dirname = dirname(fileURLToPath(import.meta.url));
const MERMAID = join(__dirname, 'node_modules/mermaid/dist/mermaid.min.js');

const diagrams = {
  cynefin: `cynefin\n  title Cynefin Framework\n  Obvious\n    Best Practice\n  Complicated\n    Expert Analysis\n  Complex\n    Emergence\n  Chaotic\n    Act First`,
  zenuml: `zenuml\n  title Order Flow\n  @Actor Alice\n  @Database DB\n  Alice -> DB: Query\n  DB --> Alice: Result`,
  railroad: `railroad-beta\n  diagram\n    Choice:\n      Terminal: "yes"\n      Terminal: "no"`,
  eventmodeling: `eventmodeling\n  title Simple Order\n  section Web\n    command : PlaceOrder\n    event : OrderPlaced`
};

const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] });
const page = await browser.newPage();
await page.setContent('<!DOCTYPE html><html><body></body></html>');
await page.addScriptTag({ path: MERMAID });
await page.evaluate(() => window.mermaid.initialize({ startOnLoad: false, theme: 'default', securityLevel: 'loose' }));

for (const [name, src] of Object.entries(diagrams)) {
  try {
    const { svg } = await page.evaluate(async (text) => {
      const c = document.createElement('div'); c.id='mc'; document.body.appendChild(c);
      const r = await window.mermaid.render('test-svg', text, c); c.remove(); return r;
    }, src);
    console.log(`${name}: OK (${svg.length} bytes)`);
  } catch(e) {
    console.log(`${name}: ERROR - ${e.message.substring(0,100)}`);
  }
}
await browser.close();
