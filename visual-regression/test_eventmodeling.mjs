import puppeteer from 'puppeteer';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
const __dirname = dirname(fileURLToPath(import.meta.url));
const MERMAID = join(__dirname, 'node_modules/mermaid/dist/mermaid.min.js');

const src = `eventmodeling
  title Simple Order System
  swimlane Web
    command : Place Order
    event : Order Placed
  swimlane Service
    event : Order Placed
    command : Process Payment
    event : Payment Processed`;

const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] });
const page = await browser.newPage();
await page.setContent('<!DOCTYPE html><html><body></body></html>');
await page.addScriptTag({ path: MERMAID });
await page.evaluate(() => window.mermaid.initialize({ startOnLoad: false, theme: 'default', securityLevel: 'loose' }));
try {
  const result = await page.evaluate(async (text) => {
    const c = document.createElement('div'); c.id='mc'; document.body.appendChild(c);
    try { const r = await window.mermaid.render('test-svg', text, c); c.remove(); return { svg: r.svg }; }
    catch(e) { return { error: e.message }; }
  }, src);
  if (result.svg) console.log('OK:', result.svg.substring(0, 200));
  else console.log('ERROR:', result.error);
} catch(e) { console.log('OUTER ERROR:', e.message); }
await browser.close();
