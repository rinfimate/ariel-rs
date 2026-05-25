/**
 * audit_comprehensive.mjs — full 4-theme audit for font-family, font-size,
 * font-weight and color (fill/stroke) differences between ref and ariel-rs.
 *
 * Usage: node audit_comprehensive.mjs
 * Output: console + audit_comprehensive.json
 */

import { readFileSync, writeFileSync, readdirSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const THEMES = ['default', 'dark', 'forest', 'neutral'];

function rustDir(t) { return t === 'default' ? join(__dirname, 'rust') : join(__dirname, `rust_${t}`); }
function refDir(t)  { return t === 'default' ? join(__dirname, 'ref')  : join(__dirname, `ref_${t}`); }

function stripStyle(svg) {
  return svg.replace(/<style[^>]*>[\s\S]*?<\/style>/gi, '');
}

function extractAttrs(svg) {
  const s = stripStyle(svg);

  // font-family: attribute and inline style
  const families = new Set();
  for (const m of s.matchAll(/font-family="([^"]+)"/g)) {
    const first = m[1].split(',')[0].trim().toLowerCase().replace(/^["' ]+|["' ]+$/g, '');
    if (first && first.length < 40 && !first.startsWith('&') && !first.startsWith('var(')) families.add(first);
  }
  for (const m of s.matchAll(/font-family:\s*([^;}"'<]+)/g)) {
    const first = m[1].split(',')[0].trim().toLowerCase().replace(/^["' ]+|["' ]+$/g, '');
    if (first && first.length < 40 && !first.startsWith('&') && !first.startsWith('var(')) families.add(first);
  }

  // font-size: attribute and inline style (normalise: strip px)
  const sizes = new Set();
  for (const m of s.matchAll(/font-size[=:"' ]+([0-9.]+)(px|em|rem|ex|pt)?/g)) {
    sizes.add(m[1] + (m[2] || ''));
  }

  // font-weight: attribute and inline style
  const weights = new Set();
  for (const m of s.matchAll(/font-weight[=:"' ]+(bold|bolder|lighter|normal|[1-9][0-9]{2})/g)) {
    weights.add(m[1]);
  }

  // fill / stroke — element-level attributes only (style block already stripped)
  const fills   = new Set();
  const strokes = new Set();
  const SKIP = new Set(['none','transparent','inherit','currentcolor','white','black','url']);
  for (const m of s.matchAll(/\bfill="(#[0-9a-fA-F]{3,8}|rgba?\([^)]+\)|hsl[^"]*|[a-z]+)"/g)) {
    const v = m[1].toLowerCase();
    if (!SKIP.has(v) && !v.startsWith('url(')) fills.add(v);
  }
  for (const m of s.matchAll(/fill:\s*(#[0-9a-fA-F]{3,8}|rgba?\([^)]+\)|hsl[^;}"]+|[a-z]+)/g)) {
    const v = m[1].toLowerCase().trim();
    if (!SKIP.has(v) && !v.startsWith('url(')) fills.add(v);
  }
  for (const m of s.matchAll(/\bstroke="(#[0-9a-fA-F]{3,8}|rgba?\([^)]+\)|[a-z]+)"/g)) {
    const v = m[1].toLowerCase();
    if (!SKIP.has(v) && !v.startsWith('url(')) strokes.add(v);
  }
  for (const m of s.matchAll(/stroke:\s*(#[0-9a-fA-F]{3,8}|rgba?\([^)]+\)|[a-z]+)/g)) {
    const v = m[1].toLowerCase().trim();
    if (!SKIP.has(v) && !v.startsWith('url(')) strokes.add(v);
  }

  return {
    families: [...families].sort(),
    sizes:    [...sizes].sort((a, b) => parseFloat(a) - parseFloat(b)),
    weights:  [...weights].sort(),
    fills:    [...fills].sort(),
    strokes:  [...strokes].sort(),
  };
}

// Normalise size for comparison: "16px" == "16"
function normSizes(sizes) {
  return sizes.map(s => s.replace(/px$/, '')).sort((a,b) => parseFloat(a)-parseFloat(b));
}

// ── Collect data ──────────────────────────────────────────────────────────────

const names = readdirSync(rustDir('default'))
  .filter(f => f.endsWith('.svg'))
  .map(f => f.replace('.svg', ''))
  .sort();

const report = { generated_at: new Date().toISOString(), themes: {}, summary: {} };
const allIssues = {}; // name → { theme → issues[] }

for (const theme of THEMES) {
  const rDir = refDir(theme);
  const uDir = rustDir(theme);
  const issues = [];

  for (const name of names) {
    const rPath = join(rDir, name + '.svg');
    const uPath = join(uDir, name + '.svg');
    if (!existsSync(rPath) || !existsSync(uPath)) continue;

    const ref  = extractAttrs(readFileSync(rPath, 'utf8'));
    const rust = extractAttrs(readFileSync(uPath, 'utf8'));

    const diag = { name };
    let hasDiff = false;

    // font-family
    const refFam  = ref.families.filter(f => !f.startsWith('var('));
    const rustFam = rust.families.filter(f => !f.startsWith('var('));
    const missFam = refFam.filter(f => !rustFam.includes(f));
    const extraFam = rustFam.filter(f => !refFam.includes(f));
    if (missFam.length || extraFam.length) {
      diag.font_family = { ref: refFam, rust: rustFam, missing: missFam, extra: extraFam };
      hasDiff = true;
    }

    // font-size (normalised)
    const refSz  = normSizes(ref.sizes);
    const rustSz = normSizes(rust.sizes);
    const missSz = refSz.filter(s => !rustSz.includes(s));
    const extraSz = rustSz.filter(s => !refSz.includes(s));
    if (missSz.length || extraSz.length) {
      diag.font_size = { ref: refSz, rust: rustSz, missing: missSz, extra: extraSz };
      hasDiff = true;
    }

    // font-weight
    const refW  = ref.weights;
    const rustW = rust.weights;
    const missW = refW.filter(w => !rustW.includes(w));
    const extraW = rustW.filter(w => !refW.includes(w));
    if (missW.length || extraW.length) {
      diag.font_weight = { ref: refW, rust: rustW, missing: missW, extra: extraW };
      hasDiff = true;
    }

    // fill
    const missFill  = ref.fills.filter(c => !rust.fills.includes(c));
    const extraFill = rust.fills.filter(c => !ref.fills.includes(c));
    if (missFill.length) { diag.fill_missing = missFill; hasDiff = true; }
    if (extraFill.length) { diag.fill_extra = extraFill; hasDiff = true; }

    // stroke
    const missStroke  = ref.strokes.filter(c => !rust.strokes.includes(c));
    const extraStroke = rust.strokes.filter(c => !ref.strokes.includes(c));
    if (missStroke.length) { diag.stroke_missing = missStroke; hasDiff = true; }
    if (extraStroke.length) { diag.stroke_extra = extraStroke; hasDiff = true; }

    if (hasDiff) issues.push(diag);
  }

  report.themes[theme] = issues;
  report.summary[theme] = { total: names.length, with_diffs: issues.length };
}

// ── Cross-theme aggregation: issues present in ALL 4 themes ──────────────────

const universal = {};
for (const name of names) {
  const inAllThemes = THEMES.every(t =>
    report.themes[t].some(d => d.name === name)
  );
  if (inAllThemes) {
    // Collect per-category across themes
    const cats = new Set();
    for (const t of THEMES) {
      const d = report.themes[t].find(x => x.name === name);
      if (d.font_family) cats.add('font_family');
      if (d.font_size)   cats.add('font_size');
      if (d.font_weight) cats.add('font_weight');
      if (d.fill_missing || d.fill_extra) cats.add('fill');
      if (d.stroke_missing || d.stroke_extra) cats.add('stroke');
    }
    universal[name] = [...cats];
  }
}
report.universal_issues = universal;

// ── Console output ────────────────────────────────────────────────────────────

console.log('\n╔══════════════════════════════════════════════════════╗');
console.log('║  ariel-rs Comprehensive Audit (all 4 themes)         ║');
console.log('╚══════════════════════════════════════════════════════╝\n');

// Print per-theme summary table
console.log('── Summary ──────────────────────────────────────────────────────');
console.log('  Theme    │ Diagrams │ With Diffs');
console.log('  ─────────┼──────────┼───────────');
for (const t of THEMES) {
  const s = report.summary[t];
  console.log(`  ${t.padEnd(8)} │   ${String(s.total).padStart(3)}    │   ${s.with_diffs}`);
}

// Font-weight issues (new information, not in existing scripts)
console.log('\n── font-weight mismatches (all themes) ──────────────────────────');
let fwCount = 0;
for (const t of THEMES) {
  for (const d of report.themes[t]) {
    if (d.font_weight) {
      console.log(`  [${t}] ${d.name}: ref=[${d.font_weight.ref.join(',')}] rust=[${d.font_weight.rust.join(',')}]`);
      if (d.font_weight.missing.length) console.log(`          missing: ${d.font_weight.missing.join(', ')}`);
      fwCount++;
    }
  }
}
if (fwCount === 0) console.log('  None — font-weight matches across all diagrams and themes ✓');

// Font-family issues
console.log('\n── font-family mismatches (default theme) ────────────────────────');
let ffCount = 0;
for (const d of report.themes['default']) {
  if (d.font_family) {
    console.log(`  ${d.name}:`);
    if (d.font_family.missing.length) console.log(`    ref has:  ${d.font_family.ref.join(' | ')}`);
    if (d.font_family.missing.length) console.log(`    rust has: ${d.font_family.rust.join(' | ')}`);
    ffCount++;
  }
}
if (ffCount === 0) console.log('  None ✓');

// Font-size issues (summarised by pattern)
const fsSummary = {};
for (const d of report.themes['default']) {
  if (d.font_size) {
    const key = `ref=[${d.font_size.ref.join(',')}] rust=[${d.font_size.rust.join(',')}]`;
    if (!fsSummary[key]) fsSummary[key] = [];
    fsSummary[key].push(d.name);
  }
}
console.log('\n── font-size mismatches (default theme, grouped by pattern) ──────');
if (Object.keys(fsSummary).length === 0) {
  console.log('  None ✓');
} else {
  for (const [pattern, diagrams] of Object.entries(fsSummary)) {
    console.log(`  ${pattern}`);
    console.log(`    affects: ${diagrams.slice(0, 6).join(', ')}${diagrams.length > 6 ? ` +${diagrams.length - 6} more` : ''}`);
  }
}

// Color issues (fill): diagrams where ref colors are missing from rust
const fillIssues = report.themes['default'].filter(d => d.fill_missing?.length);
console.log(`\n── fill colors in ref but not in rust (default) — ${fillIssues.length} diagrams ──`);
for (const d of fillIssues.slice(0, 20)) {
  console.log(`  ${d.name}: ${d.fill_missing.slice(0, 6).join(', ')}${d.fill_missing.length > 6 ? ' …' : ''}`);
}
if (fillIssues.length > 20) console.log(`  … and ${fillIssues.length - 20} more`);

// Stroke issues
const strokeIssues = report.themes['default'].filter(d => d.stroke_missing?.length);
console.log(`\n── stroke colors in ref but not in rust (default) — ${strokeIssues.length} diagrams ──`);
for (const d of strokeIssues) {
  console.log(`  ${d.name}: ${d.stroke_missing.join(', ')}`);
}
if (strokeIssues.length === 0) console.log('  None ✓');

// Universal issues (affect all 4 themes)
console.log('\n── Issues present in ALL 4 themes ───────────────────────────────');
const univEntries = Object.entries(universal);
if (univEntries.length === 0) {
  console.log('  None');
} else {
  for (const [name, cats] of univEntries) {
    console.log(`  ${name}: ${cats.join(', ')}`);
  }
}
console.log(`  Total: ${univEntries.length} diagrams`);

// Write JSON
writeFileSync(join(__dirname, 'audit_comprehensive.json'), JSON.stringify(report, null, 2));
console.log('\nFull report written to audit_comprehensive.json\n');
