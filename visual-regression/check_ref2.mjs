import { readFileSync } from 'fs';
const svg = readFileSync('visual-regression/grammar/ref_default/state.svg', 'utf8');

// Find all circles
const circRe = /<circle[^>]*>/g;
let m;
while ((m = circRe.exec(svg)) !== null) {
  const tag = m[0];
  const cx = tag.match(/cx="([^"]+)"/)?.[1];
  const cy = tag.match(/cy="([^"]+)"/)?.[1];
  const r = tag.match(/\br="([^"]+)"/)?.[1];
  if (r && parseFloat(r) >= 5) {
    console.log('circle cx=' + cx + ' cy=' + cy + ' r=' + r, tag.slice(0,80));
  }
}
