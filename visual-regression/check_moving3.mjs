import { readFileSync } from 'fs';

const svg = readFileSync('visual-regression/grammar/ref_default/state.svg', 'utf8');

// Find Moving cluster at pos 23500
const movingStart = 23500;
// Extract Moving cluster section (until next major cluster)
const movingEnd = svg.indexOf('</g></g>', movingStart + 1000);
const movingSection = svg.slice(movingStart, movingEnd + 200);

console.log('Moving cluster section length:', movingSection.length);

// Find all translates inside
const transRe = /transform="translate\(([^,)]+)[, ]([^)]+)\)"/g;
let m;
while ((m = transRe.exec(movingSection)) !== null) {
  console.log('translate:', m[1], m[2]);
}

// Find rect elements
const rectRe = /<rect[^>]*>/g;
while ((m = rectRe.exec(movingSection)) !== null) {
  const tag = m[0];
  const x = tag.match(/\bx="([^"]+)"/)?.[1];
  const y = tag.match(/\by="([^"]+)"/)?.[1];
  const w = tag.match(/width="([^"]+)"/)?.[1];
  const h = tag.match(/height="([^"]+)"/)?.[1];
  const cls = tag.match(/class="([^"]+)"/)?.[1];
  if (w && h) console.log('rect', cls || '', 'x=' + x, 'y=' + y, 'w=' + w, 'h=' + h);
}

// Find Slow and Fast
const slowIdx = movingSection.indexOf('Slow');
const fastIdx = movingSection.indexOf('Fast');
if (slowIdx > 0) console.log('Slow context:', movingSection.slice(Math.max(0, slowIdx-100), slowIdx+50).replace(/\n/g,' '));
if (fastIdx > 0) console.log('Fast context:', movingSection.slice(Math.max(0, fastIdx-100), fastIdx+50).replace(/\n/g,' '));
