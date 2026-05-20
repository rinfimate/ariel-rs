import { readFileSync, writeFileSync, readdirSync } from 'fs';
import { join, dirname, basename } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const REF_DIR  = join(__dirname, 'ref');
const RUST_DIR = join(__dirname, 'rust');
const report   = JSON.parse(readFileSync(join(__dirname, 'report.json'), 'utf8'));

const results = report.results;

const madColor = (r) => {
  if (r.status === 'MISSING') return '#e2e3e5';
  if (r.status === 'FAIL')    return '#f8d7da';
  const mad = r.mad ?? 100;
  if (mad < 0.5)  return '#d4edda';  // green
  if (mad < 1.0)  return '#fff3cd';  // yellow
  if (mad < 1.5)  return '#ffe8b3';  // orange-ish
  return '#d4edda';                   // still PASS
};

// Sort: FAIL first, then WARN, then PASS — so failures are easy to find at the top
const sorted = [...results].sort((a, b) => {
  const order = { FAIL: 0, WARN: 1, MISSING: 2, PASS: 3 };
  return (order[a.status] ?? 4) - (order[b.status] ?? 4);
});

let sections = '';
for (const r of sorted) {
  const name = r.name;
  const refPath  = join(REF_DIR,  name + '.svg');
  const rustPath = join(RUST_DIR, name + '.svg');
  let refSvg  = '';
  let rustSvg = '';
  try { refSvg  = readFileSync(refPath,  'utf8'); } catch {}
  try {
    rustSvg = readFileSync(rustPath, 'utf8');
    // Replace the generic diagram ID with a diagram-specific one so that
    // CSS rules from one embedded SVG don't bleed into another on the same page.
    const uid = `mermaid-svg-rust-${name}`;
    // Replace base SVG ID — also catches treemap's mermaid-svg-0 pattern
    rustSvg = rustSvg.replace(/mermaid-svg-0/g, uid + '-0');
    rustSvg = rustSvg.replace(/mermaid-svg(?!-)/g, uid);
    rustSvg = rustSvg.replace(/mermaid-seq(?!-)/g, uid);
  } catch {}

  const mad = r.mad ?? 100, pdiff = r.pdiff ?? 100;
  const sizeInfo = (r.refW && r.rustW)
    ? ` [ref ${r.refW}×${r.refH} | rust ${r.rustW}×${r.rustH}]`
    : (r.sizeMismatch ? ` | ${r.sizeMismatch}` : '');
  const statusPrefix = (r.status === 'FAIL' || r.status === 'WARN' || r.status === 'MISSING')
    ? `${r.status} — ` : '';
  const badge = r.status === 'MISSING'
    ? 'MISSING — no Rust output'
    : `${statusPrefix}MAD ${mad.toFixed(3)}%  PDIFF ${pdiff.toFixed(3)}%${sizeInfo}`;
  const bg = madColor(r);

  // Wide diagrams (gantt, xychart, sankey, timeline, architecture, block) stack vertically
  const wideTypes = ['gantt', 'xychart', 'sankey', 'timeline', 'architecture', 'block', 'treemap', 'mindmap'];
  const isWide = wideTypes.some(t => name.startsWith(t));
  const rowClass = isWide ? 'diagram-row diagram-row-vertical' : 'diagram-row';

  sections += `
<div class="diagram-section" id="${name}">
  <div class="diagram-header" style="background:${bg}">
    <span class="diagram-name">${name}</span>
    <span class="diagram-badge">${r.status} — ${badge}</span>
    <a class="diagram-anchor" href="#${name}">#</a>
  </div>
  <div class="${rowClass}">
    <div class="diagram-box">
      <div class="box-label">Reference (Mermaid JS)</div>
      <div class="svg-wrap">${refSvg}</div>
    </div>
    <div class="diagram-box">
      <div class="box-label">Rust (ariel-rs)</div>
      <div class="svg-wrap">${rustSvg}</div>
    </div>
  </div>
</div>`;
}

// Build table of contents
let toc = '<ul class="toc">';
for (const r of sorted) {
  const bg = madColor(r);
  const tocName = r.status === 'FAIL' || r.status === 'MISSING'
    ? `<strong>${r.name} ${r.status}</strong>`
    : r.status === 'WARN'
    ? `<strong>${r.name} WARN</strong>`
    : r.name;
  toc += `<li style="background:${bg}"><a href="#${r.name}">${tocName}</a></li>`;
}
toc += '</ul>';

const failCount = results.filter(r => r.status === 'FAIL' || r.status === 'MISSING').length;

const html = `<!DOCTYPE html>
<html><head>
<meta charset="utf-8">
<title>ariel-rs — All Corpus Diagrams</title>
<style>
* { box-sizing: border-box; }
body { font-family: Arial, sans-serif; margin: 0; background: #f0f0f0; color: #222; }
h1 { font-size: 18px; margin: 16px 20px 4px; }
.summary { font-size: 13px; margin: 0 20px 12px; color: #555; }

.toc { display: flex; flex-wrap: wrap; gap: 4px; list-style: none; padding: 8px 20px; margin: 0 0 16px; background: white; border-bottom: 1px solid #ccc; }
.toc li { border-radius: 4px; padding: 4px 8px; font-size: 11px; }
.toc li a { text-decoration: none; color: #333; font-weight: bold; }
.toc li span { color: #666; }

.diagram-section { margin: 0 0 24px; }
.diagram-header { display: flex; align-items: center; gap: 12px; padding: 8px 20px; font-size: 13px; border-top: 2px solid #ccc; }
.diagram-name { font-weight: bold; font-size: 14px; min-width: 220px; }
.diagram-badge { color: #444; }
.diagram-anchor { margin-left: auto; text-decoration: none; color: #999; font-size: 12px; }
.diagram-row { display: flex; gap: 0; }
.diagram-row-vertical { flex-direction: column; }
.diagram-row-vertical .diagram-box { border-right: none; border-bottom: 1px solid #e0e0e0; }
.diagram-row-vertical .diagram-box:last-child { border-bottom: none; }
.diagram-box { flex: 1; background: white; border-right: 1px solid #e0e0e0; padding: 12px 16px; min-width: 0; overflow: hidden; }
.diagram-box:last-child { border-right: none; }
.box-label { font-size: 11px; font-weight: bold; color: #666; margin-bottom: 8px; }
.svg-wrap svg { width: 100%; height: auto; display: block; }
</style>
</head><body>
<h1>ariel-rs — All Corpus Diagrams</h1>
<p class="summary">${results.length} diagrams &nbsp;|&nbsp; FAIL/MISSING shown first &nbsp;|&nbsp; Left = Mermaid JS reference &nbsp;&nbsp; Right = Rust output</p>
${toc}
${sections}
</body></html>`;

writeFileSync(join(__dirname, 'compare_all.html'), html, 'utf8');
console.log(`Written compare_all.html (${results.length} diagrams)`);
