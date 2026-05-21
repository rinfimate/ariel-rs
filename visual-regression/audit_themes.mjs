/**
 * audit_themes.mjs — cross-theme color variance detector.
 *
 * Compares rust SVG outputs across Default/Dark/Forest/Neutral themes.
 * If an element's fill/stroke value is the SAME in all 4 rust themes,
 * but DIFFERS in the corresponding ref SVGs → the color is hardcoded
 * (should use ThemeVars instead).
 *
 * Also checks for font-family/font-size mismatches between ref and rust.
 *
 * Outputs:
 *   - Console: human-readable report
 *   - audit_report.json: machine-readable fix tasks
 */

import { readFileSync, writeFileSync, readdirSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const THEMES = ['default', 'dark', 'forest', 'neutral'];

// Known ThemeVars defaults (Default theme → what the ThemeVars field is called)
const KNOWN_THEME_COLORS = {
  '#ececff': 'vars.primary_color',
  '#9370db': 'vars.primary_border',
  '#333333': 'vars.primary_text or vars.line_color',
  '#333': 'vars.primary_text or vars.line_color',
  'rgb(147, 112, 219)': 'vars.primary_border',
};

function rustDir(theme) {
  return theme === 'default' ? join(__dirname, 'rust') : join(__dirname, `rust_${theme}`);
}
function refDir(theme) {
  return theme === 'default' ? join(__dirname, 'ref') : join(__dirname, `ref_${theme}`);
}

function stripStyle(svg) {
  return svg.replace(/<style[^>]*>[\s\S]*?<\/style>/gi, '');
}

function extractColorAttrs(svg) {
  const s = stripStyle(svg);
  const colors = new Map(); // attr_key → Set of values

  // fill="..." and stroke="..." attributes
  for (const m of s.matchAll(/\b(fill|stroke)="(#[0-9a-fA-F]{3,8}|rgba?\([^)]+\)|[a-z]+)"/g)) {
    const key = m[1]; // 'fill' or 'stroke'
    const val = m[2].toLowerCase();
    if (['none', 'transparent', 'inherit', 'currentcolor', 'white', 'black'].includes(val)) continue;
    if (val.startsWith('url(')) continue;
    if (!colors.has(key)) colors.set(key, new Set());
    colors.get(key).add(val);
  }

  // fill/stroke in style="fill:X;stroke:Y"
  for (const m of s.matchAll(/\b(fill|stroke):\s*(#[0-9a-fA-F]{3,8}|rgba?\([^)]+\)|[a-z]+)/g)) {
    const key = m[1];
    const val = m[2].toLowerCase();
    if (['none', 'transparent', 'inherit', 'white', 'black'].includes(val)) continue;
    if (!colors.has(key)) colors.set(key, new Set());
    colors.get(key).add(val);
  }

  return colors;
}

function extractFontAttrs(svg) {
  const s = stripStyle(svg);
  const families = new Set();
  const sizes = new Set();
  for (const m of s.matchAll(/font-family="([^"]+)"/g)) {
    families.add(m[1].split(',')[0].trim().toLowerCase());
  }
  for (const m of s.matchAll(/font-size[=:"' ]+([0-9.]+(?:px)?)/g)) {
    sizes.add(m[1]);
  }
  return { families, sizes };
}

// Load SVGs for all themes for a given diagram name
function loadAllThemes(name) {
  const result = {};
  for (const theme of THEMES) {
    const path = join(rustDir(theme), `${name}.svg`);
    result[theme] = existsSync(path) ? readFileSync(path, 'utf8') : null;
  }
  return result;
}

function loadRefThemes(name) {
  const result = {};
  for (const theme of THEMES) {
    const path = join(refDir(theme), `${name}.svg`);
    result[theme] = existsSync(path) ? readFileSync(path, 'utf8') : null;
  }
  return result;
}

// Get diagram names from the default rust dir
const diagrams = readdirSync(rustDir('default'))
  .filter(f => f.endsWith('.svg'))
  .map(f => f.replace('.svg', ''))
  .sort();

const fixTasks = [];
const fontIssues = [];
const colorReport = [];

for (const name of diagrams) {
  const rustSvgs = loadAllThemes(name);
  const refSvgs = loadRefThemes(name);

  // Skip if any theme missing
  if (!THEMES.every(t => rustSvgs[t])) continue;

  // ── Cross-theme color variance check ──────────────────────────────────────
  const rustColors = {};
  const refColors = {};
  for (const theme of THEMES) {
    rustColors[theme] = extractColorAttrs(rustSvgs[theme]);
    if (refSvgs[theme]) {
      refColors[theme] = extractColorAttrs(refSvgs[theme]);
    }
  }

  // For each attribute (fill/stroke), collect all values across themes
  const attrs = new Set(['fill', 'stroke']);
  for (const attr of attrs) {
    // All values seen in rust across all themes
    const rustValueSets = THEMES.map(t => rustColors[t].get(attr) || new Set());
    const allRustValues = new Set([...rustValueSets].flatMap(s => [...s]));

    // Values that appear in ALL 4 rust themes (unchanged = potentially hardcoded)
    const unchangedInRust = [...allRustValues].filter(v =>
      rustValueSets.every(s => s.has(v))
    );

    // Check if these same values vary in the ref SVGs
    for (const val of unchangedInRust) {
      if (!KNOWN_THEME_COLORS[val.toLowerCase()]) continue; // only flag known Default-theme colors

      const refHasDifferentValues = THEMES.some(t => {
        if (!refColors[t]) return false;
        const refSet = refColors[t].get(attr) || new Set();
        return !refSet.has(val); // ref doesn't use this value in this theme
      });

      if (refHasDifferentValues) {
        fixTasks.push({
          diagram: name,
          attribute: attr,
          hardcoded_value: val,
          suggested_field: KNOWN_THEME_COLORS[val.toLowerCase()],
          severity: 'hardcoded_color',
        });
      }
    }
  }

  // ── Font family/size mismatch vs ref ──────────────────────────────────────
  const rustFonts = extractFontAttrs(rustSvgs['default']);
  if (refSvgs['default']) {
    const refFonts = extractFontAttrs(stripStyle(refSvgs['default']));
    const missingFamilies = [...refFonts.families].filter(f =>
      !f.startsWith('var(') && !rustFonts.families.has(f)
    );
    if (missingFamilies.length > 0) {
      fontIssues.push({ diagram: name, missing_families: missingFamilies });
    }
  }
}

// Deduplicate fix tasks by (diagram, attribute, value)
const seen = new Set();
const dedupedTasks = fixTasks.filter(t => {
  const key = `${t.diagram}|${t.attribute}|${t.hardcoded_value}`;
  if (seen.has(key)) return false;
  seen.add(key);
  return true;
});

// Group by diagram
const byDiagram = {};
for (const task of dedupedTasks) {
  if (!byDiagram[task.diagram]) byDiagram[task.diagram] = [];
  byDiagram[task.diagram].push(task);
}

// ── Report ──────────────────────────────────────────────────────────────────
console.log('\n=== Hardcoded Theme Colors (same in all 4 rust themes, differs in ref) ===\n');
let totalIssues = 0;
for (const [diagram, tasks] of Object.entries(byDiagram)) {
  const items = tasks.map(t => `${t.attribute}="${t.hardcoded_value}" → ${t.suggested_field}`).join(', ');
  console.log(`  ${diagram}: ${items}`);
  totalIssues += tasks.length;
}
console.log(`\nTotal hardcoded color instances: ${totalIssues} across ${Object.keys(byDiagram).length} diagrams`);

if (fontIssues.length > 0) {
  console.log('\n=== Missing Font Families ===');
  for (const f of fontIssues) {
    console.log(`  ${f.diagram}: missing ${f.missing_families.join(', ')}`);
  }
}

// Write machine-readable report
const report = {
  generated_at: new Date().toISOString(),
  hardcoded_colors: byDiagram,
  font_issues: fontIssues,
  summary: {
    diagrams_with_hardcoded_colors: Object.keys(byDiagram).length,
    total_hardcoded_instances: totalIssues,
    diagrams_with_font_issues: fontIssues.length,
  },
};
writeFileSync(join(__dirname, 'audit_report.json'), JSON.stringify(report, null, 2));
console.log('\nFull report written to audit_report.json');
