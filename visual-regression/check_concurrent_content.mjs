import { readFileSync } from 'fs';
const svg = readFileSync('visual-regression/grammar/rust_default/state.svg', 'utf8');

const rects = [];
let m;
const rectRe = /<rect[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"/g;
while ((m = rectRe.exec(svg)) !== null) {
  rects.push({x: parseFloat(m[1]), y: parseFloat(m[2]), w: parseFloat(m[3]), h: parseFloat(m[4])});
}
const divRects = rects.filter(r => Math.abs(r.h - 254) < 2);
console.log('Divider rects (h~254):', JSON.stringify(divRects));

const circles = [];
const circRe = /<circle[^>]*cx="([^"]+)"[^>]*cy="([^"]+)"[^>]*r="([^"]+)"/g;
while ((m = circRe.exec(svg)) !== null) {
  circles.push({cx: parseFloat(m[1]), cy: parseFloat(m[2]), r: parseFloat(m[3])});
}
const bigCircles = circles.filter(c => c.r >= 5);
console.log('Circles r>=5:', JSON.stringify(bigCircles));

const stateBoxes = rects.filter(r => Math.abs(r.h - 40) < 2 && r.w > 30);
console.log('State boxes (h=40):', JSON.stringify(stateBoxes));

// Analyze first divider rect
if (divRects.length > 0) {
  const dr = divRects[0];
  console.log(`\n=== First divider rect: y=${dr.y} bottom=${dr.y+dr.h} h=${dr.h} ===`);
  // Find circles within this divider's x range
  const inDiv = bigCircles.filter(c => c.cx >= dr.x && c.cx <= dr.x + dr.w);
  console.log('Circles in div:', JSON.stringify(inDiv));
  if (inDiv.length > 0) {
    const c = inDiv[0];
    console.log(`Circle top: ${c.cy - c.r}, TOP pad: ${c.cy - c.r - dr.y}`);
  }
  // Find boxes within divider x range
  const boxInDiv = stateBoxes.filter(b => b.x >= dr.x && b.x + b.w <= dr.x + dr.w + 5);
  console.log('Boxes in div:', JSON.stringify(boxInDiv));
  if (boxInDiv.length > 0) {
    const b = boxInDiv[boxInDiv.length - 1];
    console.log(`Box bottom: ${b.y + b.h}, BOTTOM pad: ${dr.y + dr.h - (b.y + b.h)}`);
  }
}
