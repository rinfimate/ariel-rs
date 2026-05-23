import { readFileSync } from 'fs';

const svg = readFileSync('visual-regression/grammar/ref_default/state.svg', 'utf8');

// Find all cluster/composite containers
const idRe = /id="[^"]*Moving[^"]*"/g;
let m;
while ((m = idRe.exec(svg)) !== null) {
  console.log('Found Moving id:', m[0], 'at pos', m.index);
  // Show context
  console.log(svg.slice(m.index, m.index + 300).replace(/\n/g, ' '));
  console.log('---');
}

// Look for rect with class="label" or outer/inner in context of Moving
// In mermaid JS, cluster rects might use different class names
const clusterRe = /<g[^>]*class="[^"]*cluster[^"]*"[^>]*>/g;
while ((m = clusterRe.exec(svg)) !== null) {
  const ctx = svg.slice(m.index, m.index + 200).replace(/\n/g, ' ');
  if (ctx.includes('Moving')) console.log('Cluster with Moving:', ctx);
}
