import puppeteer from 'puppeteer';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
const __dirname = dirname(fileURLToPath(import.meta.url));
const MERMAID = join(__dirname, 'node_modules/mermaid/dist/mermaid.min.js');

// Try different eventmodeling syntaxes
const tests = [
`eventmodeling
  Web
    PlaceOrder: cmd
    OrderPlaced: event`,
`eventmodeling
  section Web
    PlaceOrder[cmd]
    OrderPlaced[event]`,
`eventmodeling
  Web[lane]
    PlaceOrder[command]
    OrderPlaced[event]`,
];

const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] });
const page = await browser.newPage();
await page.setContent('<!DOCTYPE html><html><body></body></html>');
await page.addScriptTag({ path: MERMAID });
await page.evaluate(() => window.mermaid.initialize({ startOnLoad: false, theme: 'default', securityLevel: 'loose' }));

for (let i = 0; i < tests.length; i++) {
  const result = await page.evaluate(async (text) => {
    const c = document.createElement('div'); c.id='mc'; document.body.appendChild(c);
    try { const r = await window.mermaid.render('test-svg-'+Date.now(), text, c); c.remove(); return { ok: true, len: r.svg.length }; }
    catch(e) { c.remove(); return { ok: false, err: e.message.substring(0,80) }; }
  }, tests[i]);
  console.log(`Test ${i+1}: ${result.ok ? 'OK ('+result.len+' bytes)' : 'FAIL: '+result.err}`);
}
await browser.close();
