/**
 * gen_compare_all.mjs — per-theme visual comparison HTML files.
 *
 * Usage:
 *   node gen_compare_all.mjs [theme]          — generate one theme file
 *   node gen_compare_all.mjs                  — generate all 4 theme files
 *
 * Output: compare_{theme}.html (one per theme, independent, ~196 iframes each)
 */
import { readFileSync, writeFileSync, existsSync, readdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

const THEMES = ['default', 'dark', 'forest', 'neutral'];
const THEME_LABEL = { default: 'Default', dark: 'Dark', forest: 'Forest', neutral: 'Neutral' };

const themeArg = process.argv[2];
const themesToBuild = (themeArg && THEMES.includes(themeArg)) ? [themeArg] : THEMES;

// ── Helpers ───────────────────────────────────────────────────────────────────

function refDir(theme) {
  return theme === 'default' ? join(__dirname, 'ref') : join(__dirname, `ref_${theme}`);
}
function rustDir(theme) {
  return theme === 'default' ? join(__dirname, 'rust') : join(__dirname, `rust_${theme}`);
}
function reportFile(theme) {
  const p = join(__dirname, `report_${theme}.json`);
  if (existsSync(p)) return p;
  return join(__dirname, 'report.json'); // legacy fallback
}

function loadReport(theme) {
  const p = reportFile(theme);
  if (!existsSync(p)) return null;
  return JSON.parse(readFileSync(p, 'utf8'));
}

function loadSvg(dir, name, uid) {
  const path = join(dir, name + '.svg');
  if (!existsSync(path)) return '';
  let svg = readFileSync(path, 'utf8');
  if (uid) {
    svg = svg.replace(/mermaid-svg-0/g, uid + '-0');
    svg = svg.replace(/mermaid-svg(?!-)/g, uid);
    svg = svg.replace(/mermaid-seq(?!-)/g, uid);
  }
  return svg;
}

function escapeSvgForSrcdoc(s) {
  return s.replace(/&/g, '&amp;');
}

const STATUS_BG = { PASS: '#d4edda', WARN: '#fff3cd', FAIL: '#f8d7da', MISSING: '#e2e3e5' };
const STATUS_ICON = { PASS: '✓', WARN: '~', FAIL: '✗', MISSING: '?' };

const wideTypes = ['gantt', 'xychart', 'sankey', 'timeline', 'architecture', 'block', 'treemap', 'mindmap'];

// ── Build one theme file ──────────────────────────────────────────────────────

function buildThemeFile(theme) {
  const report = loadReport(theme);
  if (!report) { console.warn(`  No report for ${theme}, skipping`); return; }

  const { summary, results } = report;
  const rDir = refDir(theme);
  const uDir = rustDir(theme);

  // Sort: FAIL first, then WARN, PASS last
  const sorted = [...results].sort((a, b) => {
    const ord = { FAIL: 0, MISSING: 1, WARN: 2, PASS: 3 };
    return (ord[a.status] ?? 4) - (ord[b.status] ?? 4);
  });

  // ── TOC ──────────────────────────────────────────────────────────────────────
  let toc = '<ul class="toc">';
  for (const r of sorted) {
    const bg = STATUS_BG[r.status] || '#d4edda';
    const icon = STATUS_ICON[r.status] || '';
    const madStr = r.mad != null ? ` ${r.mad.toFixed(1)}%` : '';
    toc += `<li style="background:${bg}"><a href="#${r.name}">${r.name}</a> <span class="toc-score">${icon}${madStr}</span></li>`;
  }
  toc += '</ul>';

  // ── Diagram sections ─────────────────────────────────────────────────────────
  let sections = '';
  for (const r of sorted) {
    const name = r.name;
    const bg = STATUS_BG[r.status] || '#d4edda';
    const isWide = wideTypes.some(t => name.startsWith(t));
    const rowClass = isWide ? 'diagram-row diagram-row-vertical' : 'diagram-row';
    const mad = r.mad != null ? r.mad.toFixed(3) : '?';
    const pdiff = r.pdiff != null ? r.pdiff.toFixed(3) : '?';
    const sizeNote = r.sizeMismatch ? ` [${r.sizeMismatch}]` : '';

    const refSvg  = loadSvg(rDir, name, null);
    const rustUid = `mermaid-svg-rust-${theme}-${name}`;
    const rustSvg = loadSvg(uDir, name, rustUid);

    const refDoc = refSvg
      ? `<!DOCTYPE html><html><head><style>*{margin:0;padding:0}body{background:white}svg{width:100%;height:auto;display:block}</style></head><body>${escapeSvgForSrcdoc(refSvg)}</body></html>`
      : `<!DOCTYPE html><html><body><em style="color:#999;padding:8px">no ref SVG</em></body></html>`;
    const rustDoc = rustSvg
      ? `<!DOCTYPE html><html><head><style>*{margin:0;padding:0}body{background:white}svg{width:100%;height:auto;display:block}</style></head><body>${escapeSvgForSrcdoc(rustSvg)}</body></html>`
      : `<!DOCTYPE html><html><body><em style="color:#999;padding:8px">no rust SVG</em></body></html>`;

    sections += `
<div class="diagram-section" id="${name}">
  <div class="diagram-header" style="background:${bg}">
    <span class="diagram-name">${name}</span>
    <span class="diagram-badge">${r.status} — MAD ${mad}%  PDIFF ${pdiff}%${sizeNote}</span>
    <a class="diagram-anchor" href="#${name}">#</a>
  </div>
  <div class="${rowClass}">
    <div class="diagram-box">
      <div class="box-label">Reference (Mermaid JS)</div>
      <iframe class="svg-frame" srcdoc="${refDoc.replace(/"/g, '&quot;')}" scrolling="no" onload="autoHeight(this)"></iframe>
    </div>
    <div class="diagram-box">
      <div class="box-label">Rust (ariel-rs)</div>
      <iframe class="svg-frame" srcdoc="${rustDoc.replace(/"/g, '&quot;')}" scrolling="no" onload="autoHeight(this)"></iframe>
    </div>
  </div>
</div>`;
  }

  // ── Navigation ───────────────────────────────────────────────────────────────
  const navLinks = THEMES.map(t => {
    const active = t === theme ? ' class="nav-active"' : '';
    const otherReport = loadReport(t);
    const score = otherReport ? `${otherReport.summary.pass}P ${otherReport.summary.fail}F` : '';
    return `<a href="compare_${t}.html"${active}>${THEME_LABEL[t]} <span class="nav-score">${score}</span></a>`;
  }).join('');

  // ── Full HTML ─────────────────────────────────────────────────────────────────
  const html = `<!DOCTYPE html>
<html><head>
<meta charset="utf-8">
<title>ariel-rs — ${THEME_LABEL[theme]} Theme</title>
<style>
* { box-sizing: border-box; }
body { font-family: Arial, sans-serif; margin: 0; background: #f0f0f0; color: #222; }

/* Navigation */
.nav { display: flex; gap: 8px; padding: 10px 20px; background: #222; align-items: center; position: sticky; top: 0; z-index: 100; }
.nav a { padding: 6px 14px; border-radius: 4px; text-decoration: none; color: #ccc; font-size: 13px; background: #333; }
.nav a:hover { background: #444; color: white; }
.nav a.nav-active { background: #0066cc; color: white; font-weight: bold; }
.nav-score { font-size: 11px; opacity: 0.8; }
.nav-title { color: #888; font-size: 12px; margin-right: 8px; }

/* Summary */
.summary-bar { padding: 8px 20px; background: white; border-bottom: 1px solid #ddd; font-size: 13px; display: flex; gap: 20px; align-items: center; }
.s-pass { color: #155724; font-weight: bold; }
.s-warn { color: #856404; font-weight: bold; }
.s-fail { color: #721c24; font-weight: bold; }

/* TOC */
.toc { display: flex; flex-wrap: wrap; gap: 4px; list-style: none; padding: 8px 20px; margin: 0 0 16px;
  background: white; border-bottom: 1px solid #ccc; max-height: 180px; overflow-y: auto; }
.toc li { border-radius: 4px; padding: 3px 6px; font-size: 11px; }
.toc li a { text-decoration: none; color: #333; font-weight: bold; }
.toc-score { color: #666; font-size: 10px; }

/* Diagrams */
.diagram-section { margin: 0 0 12px; }
.diagram-header { display: flex; align-items: center; gap: 12px; padding: 6px 20px; font-size: 12px; border-top: 2px solid #ccc; }
.diagram-name { font-weight: bold; font-size: 13px; min-width: 200px; }
.diagram-badge { color: #555; }
.diagram-anchor { margin-left: auto; text-decoration: none; color: #999; font-size: 12px; }
.diagram-row { display: flex; }
.diagram-row-vertical { flex-direction: column; }
.diagram-box { flex: 1; background: white; border-right: 1px solid #e0e0e0; padding: 6px 10px; min-width: 0; overflow: hidden; }
.diagram-box:last-child { border-right: none; }
.box-label { font-size: 10px; font-weight: bold; color: #888; margin-bottom: 4px; }
.svg-frame { width: 100%; border: none; display: block; min-height: 60px; }
</style>
</head><body>
<div class="nav">
  <span class="nav-title">ariel-rs themes:</span>
  ${navLinks}
</div>
<div class="summary-bar">
  <strong>${THEME_LABEL[theme]} theme</strong>
  <span class="s-pass">✓ ${summary.pass} pass</span>
  <span class="s-warn">~ ${summary.warn} warn</span>
  <span class="s-fail">✗ ${summary.fail} fail</span>
  <span style="color:#888">${summary.total} diagrams</span>
</div>
${toc}
${sections}
<script>
function autoHeight(iframe) {
  try {
    const d = iframe.contentDocument?.documentElement || iframe.contentDocument?.body;
    if (d) iframe.style.height = Math.max(d.scrollHeight, 60) + 'px';
  } catch(e) {}
}
</script>
</body></html>`;

  const outFile = join(__dirname, `compare_${theme}.html`);
  writeFileSync(outFile, html, 'utf8');
  console.log(`  compare_${theme}.html — ${results.length} diagrams (${summary.pass}P ${summary.warn}W ${summary.fail}F)`);
}

// ── Generate ──────────────────────────────────────────────────────────────────
console.log(`\nGenerating ${themesToBuild.length} theme file(s)...`);
for (const theme of themesToBuild) {
  buildThemeFile(theme);
}
console.log('Done.');
