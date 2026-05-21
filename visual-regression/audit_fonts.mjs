import { readFileSync, readdirSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

function extractFonts(svg) {
    const families = new Set();
    const sizes = new Set();
    const fills = new Set();
    const strokes = new Set();

    for (const m of svg.matchAll(/font-family="([^"]+)"/g)) {
        const first = m[1].split(',')[0].trim().toLowerCase().replace(/^["' ]+|["' ]+$/g, '');
        families.add(first);
    }
    for (const m of svg.matchAll(/font-family:\s*([^;}"']+)/g)) {
        const first = m[1].split(',')[0].trim().toLowerCase().replace(/^["' ]+|["' ]+$/g, '');
        if (first && !first.startsWith('var(') && first.length < 40) families.add(first);
    }
    for (const m of svg.matchAll(/font-size[=:"' ]+([0-9.]+(?:px|em|rem|ex|pt)?)/g))
        sizes.add(m[1]);

    // fill / stroke — only hex colors and named colors (skip gradients/url refs)
    for (const m of svg.matchAll(/\bfill="(#[0-9a-fA-F]{3,8}|rgba?\([^)]+\)|[a-z]+)"/g)) {
        if (!['none', 'transparent', 'inherit', 'currentcolor', 'url'].includes(m[1].toLowerCase()))
            fills.add(m[1].toLowerCase());
    }
    for (const m of svg.matchAll(/\bstroke="(#[0-9a-fA-F]{3,8}|rgba?\([^)]+\)|[a-z]+)"/g)) {
        if (!['none', 'transparent', 'inherit', 'currentcolor', 'url'].includes(m[1].toLowerCase()))
            strokes.add(m[1].toLowerCase());
    }
    // also pick up fill/stroke from style attrs
    for (const m of svg.matchAll(/fill:\s*(#[0-9a-fA-F]{3,8}|rgba?\([^)]+\)|[a-z]+)/g)) {
        if (!['none', 'transparent', 'inherit'].includes(m[1].toLowerCase()))
            fills.add(m[1].toLowerCase());
    }

    return {
        families: [...families].sort(),
        sizes: [...sizes].sort((a, b) => parseFloat(a) - parseFloat(b)),
        fills: [...fills].sort(),
        strokes: [...strokes].sort(),
    };
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
    const rf = extractFonts(ref);
    const ruf = extractFonts(rust);

    // Only flag fills/strokes that are in REF but missing from RUST (not vice versa — we may add extras)
    const missingFills = rf.fills.filter(c => !ruf.fills.includes(c));
    const missingStrokes = rf.strokes.filter(c => !ruf.strokes.includes(c));

    const famMismatch = JSON.stringify(rf.families) !== JSON.stringify(ruf.families);
    const sizMismatch = JSON.stringify(rf.sizes) !== JSON.stringify(ruf.sizes);
    const colorMismatch = missingFills.length > 0 || missingStrokes.length > 0;

    if (famMismatch || sizMismatch || colorMismatch) {
        mismatches.push({ name, rf, ruf, famMismatch, sizMismatch, missingFills, missingStrokes });
    }
}

for (const m of mismatches) {
    const issues = [];
    if (m.famMismatch) issues.push('FONT-FAMILY');
    if (m.sizMismatch) issues.push('FONT-SIZE');
    if (m.missingFills.length) issues.push('FILL');
    if (m.missingStrokes.length) issues.push('STROKE');
    console.log(`\n--- ${m.name} [${issues.join(', ')}] ---`);
    if (m.famMismatch) {
        console.log(`  font-family REF : ${m.rf.families.join(' | ')}`);
        console.log(`  font-family RUST: ${m.ruf.families.join(' | ')}`);
    }
    if (m.sizMismatch) {
        console.log(`  font-size REF : ${m.rf.sizes.join(', ')}`);
        console.log(`  font-size RUST: ${m.ruf.sizes.join(', ')}`);
    }
    if (m.missingFills.length)
        console.log(`  fills in REF but not RUST: ${m.missingFills.join(', ')}`);
    if (m.missingStrokes.length)
        console.log(`  strokes in REF but not RUST: ${m.missingStrokes.join(', ')}`);
}

console.log(`\nTotal diagrams with mismatches: ${mismatches.length}`);
