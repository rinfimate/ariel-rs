/**
 * run_grammar_regression.mjs — Grammar corpus regression pipeline
 *
 * For each of the 4 themes:
 *   1. Render reference SVGs via Mermaid JS (Puppeteer)
 *   2. Render ariel-rs SVGs via render_corpus --corpus --out
 *   3. Convert both to PNG via Puppeteer (1200px wide, same as main pipeline)
 *   4. Compute MAD/PDIFF scores via resvg pixel comparison
 *   5. Generate visual compare HTML for all 4 themes
 *
 * Usage: node run_grammar_regression.mjs
 */

import puppeteer from 'puppeteer';
import { PNG } from './node_modules/pngjs/lib/png.js';
import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'fs';
import { spawnSync } from 'child_process';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT    = join(__dirname, '..');
const CORPUS  = join(__dirname, 'grammar_corpus', 'grammar_corpus.json');
const MERMAID = join(__dirname, 'node_modules', 'mermaid', 'dist', 'mermaid.min.js');
const OUT     = join(__dirname, 'grammar');
const THEMES  = ['default', 'dark', 'forest', 'neutral'];
const TARGET_W = 1200;

const corpus = JSON.parse(readFileSync(CORPUS, 'utf8'));
const names  = Object.keys(corpus);

const d = (type, theme) => join(OUT, `${type}_${theme}`);
for (const theme of THEMES)
  for (const type of ['ref', 'rust', 'png_ref', 'png_rust'])
    mkdirSync(d(type, theme), { recursive: true });

// ── Helpers ───────────────────────────────────────────────────────────────────

async function svgToPng(page, svgText, pngPath) {
  const vbMatch = svgText.match(/viewBox="([^"]+)"/);
  let h = TARGET_W;
  if (vbMatch) {
    const p = vbMatch[1].trim().split(/\s+/).map(Number);
    if (p[2] > 0 && isFinite(p[3] / p[2])) h = Math.ceil(TARGET_W * p[3] / p[2]);
  }
  h = Math.max(Math.min(h, 20000), 50);
  await page.setViewport({ width: TARGET_W, height: h, deviceScaleFactor: 1 });
  await page.evaluate((svg, w, hh) => {
    document.body.innerHTML = svg;
    const el = document.body.querySelector('svg');
    if (el) { el.style.cssText = `display:block;width:${w}px;height:${hh}px;`; el.setAttribute('width', w); el.setAttribute('height', hh); }
  }, svgText, TARGET_W, h);
  const png = await page.screenshot({ clip: { x: 0, y: 0, width: TARGET_W, height: h }, type: 'png' });
  writeFileSync(pngPath, png);
}

function scorePngs(refPng, rustPng) {
  if (!existsSync(refPng) || !existsSync(rustPng)) return null;
  try {
    const a = PNG.sync.read(readFileSync(refPng));
    const b = PNG.sync.read(readFileSync(rustPng));
    const n = Math.max(a.width * a.height, b.width * b.height);
    const len = Math.min(a.data.length, b.data.length);
    let madSum = 0, diff = 0;
    for (let i = 0; i < len; i += 4) {
      const avg = (Math.abs(a.data[i]-b.data[i]) + Math.abs(a.data[i+1]-b.data[i+1]) + Math.abs(a.data[i+2]-b.data[i+2])) / 3;
      madSum += avg; if (avg > 1) diff++;
    }
    return { mad: madSum/n/255*100, pdiff: diff/n*100 };
  } catch { return null; }
}

// ── Step 1: Mermaid JS reference SVGs ────────────────────────────────────────
console.log('\n── Step 1: Mermaid JS reference SVGs ──');

const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox', '--disable-setuid-sandbox'] });

for (const theme of THEMES) {
  const page = await browser.newPage();
  await page.setViewport({ width: 2000, height: 2000 });
  await page.setContent('<!DOCTYPE html><html><body></body></html>');
  await page.addScriptTag({ path: MERMAID });
  await page.evaluate(t => window.mermaid.initialize({
    startOnLoad: false, theme: t, securityLevel: 'loose', fontFamily: 'Arial, sans-serif',
    look: 'classic',
  }), theme);

  let ctr = 0, ok = 0, err = 0;
  for (const [name, text] of Object.entries(corpus)) {
    const id = `g-${++ctr}`;
    try {
      const svg = await page.evaluate(async (id, text) => {
        const c = document.createElement('div'); c.id = `c-${id}`; document.body.appendChild(c);
        const { svg } = await window.mermaid.render(id, text, c); c.remove(); return svg;
      }, id, text);
      writeFileSync(join(d('ref', theme), name + '.svg'), svg); ok++;
    } catch (e) { console.log(`  ERR [${theme}] ${name}: ${e.message.split('\n')[0]}`); err++; }
  }
  await page.close();
  console.log(`  ${theme}: ${ok} ok, ${err} failed`);
}

// ── Step 2: ariel-rs SVGs ────────────────────────────────────────────────────
console.log('\n── Step 2: ariel-rs SVGs ──');

for (const theme of THEMES) {
  spawnSync('cargo', ['run', '--bin', 'render_corpus', '--release', '--', theme, '--corpus', CORPUS, '--out', d('rust', theme)],
    { cwd: ROOT, stdio: 'inherit' });
}

// ── Step 3: SVGs → PNGs via Puppeteer (1200px) ───────────────────────────────
console.log('\n── Step 3: SVG → PNG (Puppeteer) ──');

const pngPage = await browser.newPage();
await pngPage.setContent('<!DOCTYPE html><html><head><style>*{margin:0;padding:0}body{background:white}</style></head><body></body></html>');

for (const theme of THEMES) {
  let done = 0;
  for (const name of names) {
    const refSvg  = join(d('ref',  theme), name + '.svg');
    const rustSvg = join(d('rust', theme), name + '.svg');
    if (existsSync(refSvg))  await svgToPng(pngPage, readFileSync(refSvg,  'utf8'), join(d('png_ref',  theme), name + '.png'));
    if (existsSync(rustSvg)) await svgToPng(pngPage, readFileSync(rustSvg, 'utf8'), join(d('png_rust', theme), name + '.png'));
    done++;
  }
  console.log(`  ${theme}: ${done} diagrams`);
}
await pngPage.close();
await browser.close();

// ── Step 4: MAD / PDIFF scores ───────────────────────────────────────────────
console.log('\n── Step 4: Scoring ──');

const reports = {};
for (const theme of THEMES) {
  reports[theme] = [];
  for (const name of names) {
    const s = scorePngs(join(d('png_ref', theme), name+'.png'), join(d('png_rust', theme), name+'.png'));
    const status = !s ? 'MISSING' : s.mad < 2 ? 'PASS' : s.mad < 5 ? 'WARN' : 'FAIL';
    reports[theme].push({ name, status, mad: s?.mad, pdiff: s?.pdiff });
  }
  const p=reports[theme].filter(r=>r.status==='PASS').length;
  const w=reports[theme].filter(r=>r.status==='WARN').length;
  const f=reports[theme].filter(r=>r.status==='FAIL').length;
  const m=reports[theme].filter(r=>r.status==='MISSING').length;
  console.log(`  ${theme}: ${p}P ${w}W ${f}F ${m}M`);
  writeFileSync(join(OUT, `report_${theme}.json`), JSON.stringify({ results: reports[theme] }, null, 2));
}

// ── Step 5: Compare HTML ──────────────────────────────────────────────────────
console.log('\n── Step 5: Compare HTML ──');

const BG = { PASS:'#d4edda', WARN:'#fff3cd', FAIL:'#f8d7da', MISSING:'#e2e3e5' };
// For srcdoc attribute: escape & and " in the SVG content
const escapeSrcdoc = s => s.replace(/&/g,'&amp;').replace(/"/g,'&quot;');

for (const theme of THEMES) {
  const results = reports[theme];
  const p=results.filter(r=>r.status==='PASS').length;
  const w=results.filter(r=>r.status==='WARN').length;
  const f=results.filter(r=>r.status==='FAIL').length;

  const COLOR = r => BG[r.status]==='#d4edda'?'#28a745':BG[r.status]==='#fff3cd'?'#ffc107':'#dc3545';

  // Build data for JS
  const diagrams = results.map(r => {
    const rFile = join(d('ref',  theme), r.name+'.svg');
    const uFile = join(d('rust', theme), r.name+'.svg');
    return {
      name: r.name,
      status: r.status,
      mad: r.mad?.toFixed(3) ?? 'N/A',
      pdiff: r.pdiff?.toFixed(3) ?? 'N/A',
      refSrc:  existsSync(rFile) ? `ref_${theme}/${r.name}.svg`  : '',
      rustSrc: existsSync(uFile) ? `rust_${theme}/${r.name}.svg` : '',
      color: COLOR(r),
    };
  });

  const dataJson = JSON.stringify(diagrams);
  const titleTheme = theme.charAt(0).toUpperCase()+theme.slice(1);

  const html = `<!DOCTYPE html><html><head><meta charset="utf-8">
<title>Grammar Regression — ${titleTheme}</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:sans-serif;display:flex;height:100vh;overflow:hidden;background:#111}
/* Sidebar */
.sidebar{width:200px;min-width:200px;background:#1a1a1a;display:flex;flex-direction:column;height:100vh;overflow:hidden}
.sidebar-head{padding:10px 12px;background:#111;border-bottom:1px solid #333;flex-shrink:0}
.sidebar-head h2{font-size:13px;color:#fff;font-weight:bold}
.sidebar-head p{font-size:11px;color:#666;margin-top:2px}
.sidebar-list{overflow-y:auto;flex:1}
.nav-item{display:flex;align-items:center;padding:6px 10px;font-size:12px;color:#aaa;cursor:pointer;border-bottom:1px solid #222;border-left:3px solid transparent;gap:6px}
.nav-item:hover{background:#252525;color:#fff}
.nav-item.active{background:#0d2137;color:#fff;border-left-color:#4ea8f7}
.nav-item .nav-name{flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.nav-item .nav-pct{font-size:10px;color:#555;flex-shrink:0}
.nav-item .nav-dot{width:8px;height:8px;border-radius:50%;flex-shrink:0}
/* Main */
.main{flex:1;display:flex;flex-direction:column;overflow:hidden;background:#f5f5f5}
.main-head{padding:10px 16px;background:#fff;border-bottom:1px solid #ddd;display:flex;align-items:center;gap:12px;flex-shrink:0}
.main-head h1{font-size:15px;font-weight:bold;color:#222}
.badge{font-size:11px;padding:2px 8px;border-radius:10px;color:#fff}
.main-head .links{margin-left:auto;display:flex;gap:8px}
.main-head .links a{font-size:11px;padding:4px 10px;border-radius:4px;text-decoration:none;background:#e8f0fe;color:#1a73e8;border:1px solid #c5d8fb}
.main-head .links a:hover{background:#d2e3fc}
.panels{display:flex;flex:1;overflow:hidden}
.panel{flex:1;display:flex;flex-direction:column;overflow:hidden;border-right:1px solid #ddd}
.panel:last-child{border-right:none}
.panel-label{padding:5px 12px;font-size:11px;font-weight:bold;color:#666;background:#f8f8f8;border-bottom:1px solid #e0e0e0;flex-shrink:0}
.panel iframe{flex:1;border:none;width:100%;height:100%;background:#fff}
.placeholder{flex:1;display:flex;align-items:center;justify-content:center;color:#999;font-size:13px}
</style>
</head><body>
<div class="sidebar">
  <div class="sidebar-head">
    <h2>Grammar — ${titleTheme}</h2>
    <p>${p}P &nbsp;${w}W &nbsp;${f}F &nbsp;/ ${results.length}</p>
  </div>
  <div class="sidebar-list" id="navList"></div>
</div>
<div class="main">
  <div class="main-head">
    <h1 id="diagTitle">Select a diagram</h1>
    <span class="badge" id="diagBadge" style="display:none"></span>
    <div class="links" id="diagLinks" style="display:none">
      <a id="linkRef" target="_blank">Open Ref ↗</a>
      <a id="linkRust" target="_blank">Open ariel-rs ↗</a>
    </div>
  </div>
  <div class="panels">
    <div class="panel">
      <div class="panel-label">Reference (Mermaid JS)</div>
      <iframe id="frameRef" src="about:blank"></iframe>
    </div>
    <div class="panel">
      <div class="panel-label">Rust (ariel-rs)</div>
      <iframe id="frameRust" src="about:blank"></iframe>
    </div>
  </div>
</div>
<script>
const DATA = ${dataJson};
const BADGE_BG = {'PASS':'#28a745','WARN':'#e6a817','FAIL':'#dc3545','MISSING':'#6c757d'};
let current = null;

function show(name) {
  const d = DATA.find(x=>x.name===name);
  if (!d) return;
  current = name;
  // Nav highlight
  document.querySelectorAll('.nav-item').forEach(el => el.classList.toggle('active', el.dataset.name===name));
  // Header
  document.getElementById('diagTitle').textContent = name;
  const badge = document.getElementById('diagBadge');
  badge.style.display='';
  badge.style.background = BADGE_BG[d.status]||'#888';
  badge.textContent = d.status+' — MAD '+d.mad+'%  PDIFF '+d.pdiff+'%';
  // Links
  const links = document.getElementById('diagLinks');
  links.style.display='';
  document.getElementById('linkRef').href = d.refSrc || '#';
  document.getElementById('linkRust').href = d.rustSrc || '#';
  // Frames
  document.getElementById('frameRef').src = d.refSrc || 'about:blank';
  document.getElementById('frameRust').src = d.rustSrc || 'about:blank';
  // URL hash
  location.hash = name;
}

// Build nav
const nav = document.getElementById('navList');
for (const d of DATA) {
  const el = document.createElement('div');
  el.className = 'nav-item';
  el.dataset.name = d.name;
  el.innerHTML = '<span class="nav-dot" style="background:'+d.color+'"></span><span class="nav-name">'+d.name+'</span><span class="nav-pct">'+d.mad+'%</span>';
  el.onclick = () => show(d.name);
  nav.appendChild(el);
}

// Restore from hash or show first
const hash = location.hash.slice(1);
if (hash && DATA.find(x=>x.name===hash)) show(hash);
else if (DATA.length) show(DATA[0].name);
</script>
</body></html>`;

  writeFileSync(join(OUT, `compare_grammar_${theme}.html`), html);
  console.log(`  compare_grammar_${theme}.html — ${p}P ${w}W ${f}F`);
}

console.log(`\n✓ Done. Output: ${OUT}/`);
