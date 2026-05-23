import { readFileSync } from 'fs';

const ref = readFileSync('./grammar/ref_default/state.svg', 'utf8');
const ours = readFileSync('./grammar/rust_default/state.svg', 'utf8');

// Find concurrent divider boxes in reference
console.log('=== REFERENCE ===');
const refDiv1 = ref.indexOf('divider-id-1');
if (refDiv1 > 0) {
  const ctx = ref.substring(refDiv1, refDiv1 + 400);
  const rects = [...ctx.matchAll(/rect[^>]*/g)].map(m => m[0].substring(0, 100));
  rects.forEach(r => console.log('  rect:', r));
  // Find transforms for sub-graph
  const transforms = [...ctx.matchAll(/translate\([^)]+\)/g)].map(m => m[0]);
  console.log('  translates:', transforms);
}

// Find Concurrent sub-graph translate
const concIdx = ref.indexOf('"Concurrent"');
if (concIdx > 0) {
  const ctx = ref.substring(concIdx, concIdx + 800);
  console.log('  Concurrent context:', ctx.substring(0, 300));
}

// Check our divider box sizes
console.log('\n=== OURS ===');
const ourDiv1 = ours.indexOf('divider-id-1');
if (ourDiv1 > 0) {
  const ctx = ours.substring(ourDiv1, ourDiv1 + 400);
  const rects = [...ctx.matchAll(/rect[^>]*/g)].map(m => m[0].substring(0, 100));
  rects.forEach(r => console.log('  rect:', r));
}
