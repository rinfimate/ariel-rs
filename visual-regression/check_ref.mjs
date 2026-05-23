import { readFileSync } from 'fs';
const svg = readFileSync('visual-regression/grammar/ref_default/state.svg', 'utf8');

// Extract divider rects
const divRe = /class="divider"[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"/g;
let m;
const divRects = [];
while ((m = divRe.exec(svg)) !== null) {
  divRects.push({x: parseFloat(m[1]), y: parseFloat(m[2]), w: parseFloat(m[3]), h: parseFloat(m[4])});
}
console.log('DIV rects:', JSON.stringify(divRects));

// Find start circles
const circRe = /class="state-start"[^>]*cx="([^"]+)"[^>]*cy="([^"]+)"[^>]*r="([^"]+)"/g;
while ((m = circRe.exec(svg)) !== null) {
  console.log('start-circle cx=' + m[1] + ' cy=' + m[2] + ' r=' + m[3]);
}

// Find all rects (to find state boxes inside dividers)
const rectRe = /<rect[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"/g;
while ((m = rectRe.exec(svg)) !== null) {
  const r = {x: parseFloat(m[1]), y: parseFloat(m[2]), w: parseFloat(m[3]), h: parseFloat(m[4])};
  if (Math.abs(r.h - 40) < 1) console.log('h=40 rect:', JSON.stringify(r));
}
