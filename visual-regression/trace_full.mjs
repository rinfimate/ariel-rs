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
    const container = document.createElement('div');
    container.id = 'mc';
    document.body.appendChild(container);
    const { svg } = await window.mermaid.render('mermaid-svg-1', text, container);
    // Extract node transforms
    const parser = new DOMParser();
    const doc = parser.parseFromString(svg, 'image/svg+xml');
    const nodes = [...doc.querySelectorAll('[id*="entity"]')];
    const transforms = nodes.map(n => ({ id: n.id, transform: n.getAttribute('transform') }));
    return { transforms };
}, diagramText);

console.log('Node positions:');
result.transforms.forEach(t => console.log(t.id, t.transform));
await browser.close();
