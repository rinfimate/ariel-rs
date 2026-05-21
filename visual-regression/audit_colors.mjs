import { readFileSync, readdirSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

// Strip embedded <style>...</style> blocks so we only see element-level attributes
function stripStyle(svg) {
    return svg.replace(/<style[^>]*>[\s\S]*?<\/style>/gi, '');
}

function extractColors(svg) {
    const s = stripStyle(svg);
    const fills = new Set();
    const strokes = new Set();

    // fill="..." attribute on elements
    for (const m of s.matchAll(/\bfill="(#[0-9a-fA-F]{3,8}|rgba?\([^)]+\)|[a-z]+)"/g)) {
        const v = m[1].toLowerCase();
        if (!['none', 'transparent', 'inherit', 'currentcolor'].includes(v) && !v.startsWith('url'))
            fills.add(v);
    }
    // fill: in inline style=""
    for (const m of s.matchAll(/fill:\s*(#[0-9a-fA-F]{3,8}|rgba?\([^)]+\)|[a-z]+)/g)) {
        const v = m[1].toLowerCase();
        if (!['none', 'transparent', 'inherit'].includes(v))
            fills.add(v);
    }
    // stroke="..."
    for (const m of s.matchAll(/\bstroke="(#[0-9a-fA-F]{3,8}|rgba?\([^)]+\)|[a-z]+)"/g)) {
        const v = m[1].toLowerCase();
        if (!['none', 'transparent', 'inherit', 'currentcolor'].includes(v) && !v.startsWith('url'))
            strokes.add(v);
    }
    // stroke: in inline style=""
    for (const m of s.matchAll(/stroke:\s*(#[0-9a-fA-F]{3,8}|rgba?\([^)]+\)|[a-z]+)/g)) {
        const v = m[1].toLowerCase();
        if (!['none', 'transparent', 'inherit'].includes(v))
            strokes.add(v);
    }

    // font color
    const fontColors = new Set();
    for (const m of s.matchAll(/\bfill="(#[0-9a-fA-F]{3,8}|[a-z]+)"[^>]*class="[^"]*(?:text|label|title|nodeLabel|edgeLabel|actor)/g))
        fontColors.add(m[1].toLowerCase());

    return { fills: [...fills].sort(), strokes: [...strokes].sort() };
}

const refDir = join(__dirname, 'ref');
const rustDir = join(__dirname, 'rust');
const svgs = readdirSync(refDir).filter(f => f.endsWith('.svg')).sort();

const mismatches = [];
for (const f of svgs) {
    const name = f.replace('.svg', '');
    const rustPath = join(rustDir, f);
    if (!existsSync(rustPath)) continue;

    const ref = readFileSync(join(refDir, f), 'utf8');
    const rust = readFileSync(rustPath, 'utf8');
    const rf = extractColors(ref);
    const ruf = extractColors(rust);

    // Colors in REF elements but missing from RUST elements
    const missingFills = rf.fills.filter(c => !ruf.fills.includes(c));
    const missingStrokes = rf.strokes.filter(c => !ruf.strokes.includes(c));
    // Colors in RUST but not REF (new colors we added)
    const extraFills = ruf.fills.filter(c => !rf.fills.includes(c));

    if (missingFills.length || missingStrokes.length) {
        mismatches.push({ name, missingFills, missingStrokes, extraFills });
    }
}

for (const m of mismatches) {
    console.log(`\n--- ${m.name} ---`);
    if (m.missingFills.length)
        console.log(`  fill in REF not RUST: ${m.missingFills.join(', ')}`);
    if (m.missingStrokes.length)
        console.log(`  stroke in REF not RUST: ${m.missingStrokes.join(', ')}`);
    if (m.extraFills.length)
        console.log(`  fill in RUST not REF: ${m.extraFills.join(', ')}`);
}

console.log(`\nTotal diagrams with element-level color mismatches: ${mismatches.length}`);
