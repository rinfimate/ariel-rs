import puppeteer from 'puppeteer';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
const __dirname = dirname(fileURLToPath(import.meta.url));
const MERMAID = join(__dirname, 'node_modules/mermaid/dist/mermaid.min.js');

// These are the default examples shown in the Mermaid live editor
const diagrams = {
  zenuml: `zenuml
  title Demo
  Alice->John: Hello John, how are you?
  John-->Alice: Great!
  Alice->John: See you later!`,
  
  cynefin: `%%{init: {"quadrantChart": {"chartWidth": 400, "chartHeight": 400}, "themeVariables": {"quadrant1Fill": "#FF0000"} }}%%
cynefin
  title Cynefin Framework Demo
  Obvious
    Best Practice
    Standard Procedure
  Complicated
    Expert Analysis
    Technical Investigation
  Complex
    Emergence
    Experimentation
  Chaotic
    Act First
    Novel Practice
  Disorder
    Unknown`,
    
  railroad: `railroad-beta
  diagram
    Sequence:
      Choice:
        Terminal: "a"
        Terminal: "b"
        Terminal: "c"
      ZeroOrMore:
        Sequence:
          Terminal: ","
          NonTerminal: "name"`,
};

const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] });
const page = await browser.newPage();
await page.setContent('<!DOCTYPE html><html><body></body></html>');
await page.addScriptTag({ path: MERMAID });
await page.evaluate(() => window.mermaid.initialize({ startOnLoad: false, theme: 'default', securityLevel: 'loose' }));

for (const [name, src] of Object.entries(diagrams)) {
  const result = await page.evaluate(async (text) => {
    const c = document.createElement('div'); c.id='mc'; document.body.appendChild(c);
    try { const r = await window.mermaid.render('test-svg-'+Date.now(), text, c); c.remove(); return { ok: true, len: r.svg.length }; }
    catch(e) { c.remove(); return { ok: false, err: e.message.substring(0, 120) }; }
  }, src);
  console.log(`${name}: ${result.ok ? 'OK ('+result.len+' bytes)' : 'FAIL: '+result.err}`);
}
await browser.close();
