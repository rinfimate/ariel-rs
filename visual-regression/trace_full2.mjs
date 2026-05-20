import puppeteer from 'puppeteer';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
const __dirname = dirname(fileURLToPath(import.meta.url));
const MERMAID_BUNDLE = join(__dirname, 'node_modules', 'mermaid', 'dist', 'mermaid.min.js');

const diagramText = `erDiagram
    PERSON }|..|{ PERSON : "is married to"
    PERSON ||--o{ ADDRESS : "lives at"
    ADDRESS ||--|{ CITY : "is in"`;

const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] });
const page = await browser.newPage();
await page.setContent('<!DOCTYPE html><html><body></body></html>');
await page.addScriptTag({ path: MERMAID_BUNDLE });
await page.evaluate(() => {
    window.mermaid.initialize({ startOnLoad: false, theme: 'default', securityLevel: 'loose' });
});

const result = await page.evaluate(async (text) => {
    const { svg } = await window.mermaid.render('mermaid-svg-1', text, document.createElement('div'));
    const parser = new DOMParser();
    const doc = parser.parseFromString(svg, 'image/svg+xml');
    // Get edge paths
    const paths = [...doc.querySelectorAll('path.relationshipLine')];
    const edgeData = paths.map(p => p.getAttribute('d').substring(0,50));
    // Get SP1 node
    const sp1 = doc.querySelector('[id*="entity-PERSON-0---entity-PERSON-0---1"]');
    return { edgeData, sp1Transform: sp1?.getAttribute('transform') };
}, diagramText);

console.log('Self-loop paths:');
result.edgeData.slice(0,3).forEach((d,i) => console.log(`path${i+1}: ${d}`));
console.log('sp1:', result.sp1Transform);
await browser.close();
