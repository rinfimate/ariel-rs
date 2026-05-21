/**
 * run_regression_all.mjs — unified parallel regression pipeline for all 4 themes.
 *
 * Usage:
 *   node run_regression_all.mjs [--regen-ref] [--audit] [--no-rust]
 *
 * Flags:
 *   --regen-ref   Force regeneration of ref SVGs from Mermaid JS (slow, ~10s per theme)
 *   --audit       Run audit_themes.mjs after comparison to detect hardcoded colors
 *   --no-rust     Skip rust SVG rendering (use existing rust/ outputs)
 *
 * The script caches the corpus.json + mermaid bundle hash. If either changes,
 * ref SVGs are automatically regenerated.
 *
 * Outputs:
 *   - Per-theme report in visual-regression/report_{theme}.json
 *   - compare_all.html updated for the default theme
 *   - audit_report.json if --audit is passed
 */

import { execSync, spawn } from 'child_process';
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { createHash } from 'crypto';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');

const THEMES = ['default', 'dark', 'forest', 'neutral'];
const CACHE_FILE = join(__dirname, '.regression_cache.json');
const MERMAID_BUNDLE = join(__dirname, 'node_modules', 'mermaid', 'dist', 'mermaid.min.js');
const CORPUS_FILE = join(__dirname, 'corpus', 'corpus.json');

const args = process.argv.slice(2);
const forceRegenRef = args.includes('--regen-ref');
const runAudit = args.includes('--audit');
const skipRust = args.includes('--no-rust');

// ── Cache helpers ────────────────────────────────────────────────────────────

function hashFile(path) {
  if (!existsSync(path)) return 'missing';
  return createHash('sha256').update(readFileSync(path)).digest('hex').slice(0, 16);
}

function loadCache() {
  if (!existsSync(CACHE_FILE)) return {};
  try { return JSON.parse(readFileSync(CACHE_FILE, 'utf8')); } catch { return {}; }
}

function saveCache(data) {
  writeFileSync(CACHE_FILE, JSON.stringify(data, null, 2));
}

function needsRefRegen(cache) {
  if (forceRegenRef) return true;
  const corpusHash = hashFile(CORPUS_FILE);
  const mermaidHash = hashFile(MERMAID_BUNDLE);
  return cache.corpusHash !== corpusHash || cache.mermaidHash !== mermaidHash;
}

// ── Command runner ────────────────────────────────────────────────────────────

function run(cmd, opts = {}) {
  const label = opts.label || cmd.split(' ')[0];
  console.log(`  [${label}] ${cmd}`);
  execSync(cmd, { cwd: opts.cwd || __dirname, stdio: 'inherit' });
}

async function runParallel(commands) {
  return Promise.all(commands.map(({ cmd, cwd, label }) =>
    new Promise((resolve, reject) => {
      const child = spawn('node', cmd.split(' ').slice(1), {
        cwd: cwd || __dirname,
        stdio: 'inherit',
        shell: true,
      });
      child.on('close', code => code === 0 ? resolve() : reject(new Error(`${label} failed`)));
    })
  ));
}

// ── Main pipeline ─────────────────────────────────────────────────────────────

const sw = Date.now();
console.log('\n╔══════════════════════════════════════════════════════╗');
console.log('║  ariel-rs Visual Regression — All 4 Themes          ║');
console.log('╚══════════════════════════════════════════════════════╝\n');

const cache = loadCache();
const regenRef = needsRefRegen(cache);

// Step 1: Regenerate ref SVGs if needed
if (regenRef) {
  console.log('── Step 1: Regenerating reference SVGs from Mermaid JS ──');
  // Run all 4 themes in parallel
  await Promise.all(THEMES.map(theme =>
    new Promise((resolve, reject) => {
      const child = spawn('node', ['render_reference.mjs', theme], {
        cwd: __dirname, stdio: 'inherit', shell: true,
      });
      child.on('close', code => code === 0 ? resolve() :
        (console.warn(`  ref ${theme} had failures`), resolve()));
    })
  ));
  // Update cache
  saveCache({
    corpusHash: hashFile(CORPUS_FILE),
    mermaidHash: hashFile(MERMAID_BUNDLE),
    lastRegen: new Date().toISOString(),
  });
} else {
  console.log('── Step 1: Ref SVGs up to date (corpus + mermaid unchanged) ──');
}

// Step 2: Render rust SVGs for all themes
if (!skipRust) {
  console.log('\n── Step 2: Rendering ariel-rs SVGs for all themes ──');
  run(`cargo run --bin render_corpus --release -- default`, { cwd: ROOT, label: 'rust/default' });
  await Promise.all(['dark', 'forest', 'neutral'].map(theme =>
    new Promise((resolve, reject) => {
      const child = spawn('cargo', ['run', '--bin', 'render_corpus', '--release', '--', theme], {
        cwd: ROOT, stdio: 'inherit', shell: true,
      });
      child.on('close', code => code === 0 ? resolve() : reject(new Error(`rust ${theme} failed`)));
    })
  ));
}

// Step 3: Rasterize all SVGs to PNGs via browser (parallel)
console.log('\n── Step 3: Rasterizing SVGs to PNGs via browser ──');
const dirs = THEMES.flatMap(t => [
  t === 'default' ? 'ref' : `ref_${t}`,
  t === 'default' ? 'rust' : `rust_${t}`,
]);
await Promise.all(dirs.map(dir =>
  new Promise((resolve, reject) => {
    const child = spawn('node', ['svg_to_png_browser.mjs', dir], {
      cwd: __dirname, stdio: 'pipe', shell: true,
    });
    let out = '';
    child.stdout.on('data', d => out += d);
    child.stderr.on('data', d => out += d);
    child.on('close', () => {
      const last = out.trim().split('\n').pop();
      console.log(`  ${dir}: ${last}`);
      resolve();
    });
  })
));

// Step 4: Compare all themes and collect results
console.log('\n── Step 4: Pixel comparison ──');
const results = {};
for (const theme of THEMES) {
  const refD = theme === 'default' ? 'ref' : `ref_${theme}`;
  const rustD = theme === 'default' ? 'rust' : `rust_${theme}`;
  try {
    execSync(`node compare.mjs ${refD} ${rustD}`, { cwd: __dirname, stdio: 'pipe' });
  } catch (e) {
    // compare exits 1 on failures — read report anyway
  }
  try {
    const report = JSON.parse(readFileSync(join(__dirname, 'report.json'), 'utf8'));
    results[theme] = report.summary;
    // Save theme-specific report
    writeFileSync(
      join(__dirname, `report_${theme}.json`),
      JSON.stringify(report, null, 2)
    );
  } catch { results[theme] = { error: true }; }
}

// Step 5: Print summary table
console.log('\n── Results ──────────────────────────────────────────────');
console.log('  Theme    │ Pass │ Warn │ Fail');
console.log('  ─────────┼──────┼──────┼──────');
for (const theme of THEMES) {
  const r = results[theme];
  if (!r || r.error) { console.log(`  ${theme.padEnd(8)} │  err │  err │  err`); continue; }
  const p = String(r.pass).padStart(4), w = String(r.warn).padStart(4), f = String(r.fail).padStart(4);
  const flag = r.fail > 0 ? ' ✗' : r.warn > 10 ? ' ~' : ' ✓';
  console.log(`  ${theme.padEnd(8)} │${p} │${w} │${f}${flag}`);
}

// Step 6: Optional audit
if (runAudit) {
  console.log('\n── Step 6: Audit cross-theme color variance ──');
  run('node audit_themes.mjs', { label: 'audit' });
}

// Step 7: Regenerate compare_all.html for default theme
console.log('\n── Step 7: Regenerating compare_all.html ──');
run('node gen_compare_all.mjs', { label: 'html' });

const elapsed = ((Date.now() - sw) / 1000).toFixed(1);
console.log(`\n✓ Done in ${elapsed}s\n`);
