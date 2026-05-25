/**
 * measure_oracle.mjs — Browser text measurement oracle for fidelity testing.
 *
 * Protocol: line-delimited JSON over stdin/stdout.
 *   stdin  each line: {"text": "...", "fontSize": 16, "bold": false}
 *   stdout each line: {"width": 74.32, "height": 19.2}
 *
 * Uses SVG <text> + getBBox() to match Mermaid's measurement path when
 * configured with htmlLabels:false (our reference configuration). This is
 * also what the browser actually uses to render <text> elements during PNG
 * conversion, so layout decisions made from these widths match the final
 * pixels.
 *
 * Usage: node measure_oracle.mjs   (communicates via stdin/stdout)
 */

import puppeteer from 'puppeteer';
import * as readline from 'readline';

const browser = await puppeteer.launch({ headless: true });
const page = await browser.newPage();

// Blank page with an offscreen SVG to host the <text> elements we measure.
await page.setContent(`<!DOCTYPE html>
<html><body style="margin:0;padding:0;background:#fff;">
  <svg id="svg" xmlns="http://www.w3.org/2000/svg" width="2000" height="2000"
       style="position:absolute;visibility:hidden;">
    <text id="t" x="0" y="100" font-family="Arial, sans-serif"></text>
  </svg>
</body></html>`);

const rl = readline.createInterface({ input: process.stdin, terminal: false });

rl.on('line', async (line) => {
  const trimmed = line.trim();
  if (!trimmed) return;

  let req;
  try {
    req = JSON.parse(trimmed);
  } catch (e) {
    process.stdout.write(JSON.stringify({ error: `bad JSON: ${e.message}` }) + '\n');
    return;
  }

  const { text, fontSize, bold } = req;
  const fontWeight = bold ? 'bold' : 'normal';

  try {
    const result = await page.evaluate(
      ({ text, fontSize, fontWeight }) => {
        const t = document.getElementById('t');
        t.setAttribute('font-size', `${fontSize}px`);
        t.setAttribute('font-weight', fontWeight);
        t.textContent = text;
        const bbox = t.getBBox();
        return { width: bbox.width, height: bbox.height };
      },
      { text, fontSize, fontWeight }
    );
    process.stdout.write(JSON.stringify(result) + '\n');
  } catch (e) {
    process.stdout.write(JSON.stringify({ width: 0, height: fontSize * 1.2, error: e.message }) + '\n');
  }
});

rl.on('close', async () => {
  await browser.close();
  process.exit(0);
});
