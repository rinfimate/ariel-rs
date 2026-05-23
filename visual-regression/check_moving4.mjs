import { readFileSync } from 'fs';

const svg = readFileSync('visual-regression/grammar/ref_default/state.svg', 'utf8');

// Find Moving cluster
const movIdx = svg.indexOf('id="g-4-state-Moving-18"');
console.log('Moving cluster pos:', movIdx);
// Show context: 500 chars before to see parent group translate
console.log('Context before:', svg.slice(movIdx - 500, movIdx + 50).replace(/\n/g, ' '));
console.log('---');

// Find the parent translate for this cluster
const before = svg.slice(0, movIdx);
const lastTrans = before.lastIndexOf('transform="translate(');
console.log('Last translate before Moving:', svg.slice(lastTrans, lastTrans + 80));

// Look for a clusters section
const clustersIdx = svg.lastIndexOf('<g class="clusters">', movIdx);
console.log('Clusters group near Moving:', svg.slice(clustersIdx, clustersIdx + 200).replace(/\n/g,' '));

// Look for inner rect height and Slow/Fast positions in the FULL diagram translate context
// Find the section with state diagram nodes (Slow, Fast)
const slowIdx = svg.indexOf('Slow');
console.log('\nSlow context:', svg.slice(Math.max(0, slowIdx-200), slowIdx+100).replace(/\n/g,' '));

const fastIdx = svg.indexOf('Fast');
console.log('\nFast context:', svg.slice(Math.max(0, fastIdx-200), fastIdx+100).replace(/\n/g,' '));
