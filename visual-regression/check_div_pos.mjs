import { readFileSync } from 'fs';
const ref = readFileSync('./grammar/ref_default/state.svg','utf8');
const ours = readFileSync('./grammar/rust_default/state.svg','utf8');

const getDivs = svg => [...svg.matchAll(/class="divider" x="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/g)]
  .map((d,i) => `div${i}: x=${d[1]} y=${d[2]} w=${d[3]} h=${d[4]}`);

console.log('REF:', getDivs(ref).join(', '));
console.log('OURS:', getDivs(ours).join(', '));

// ref div2 expected x
const refDiv1 = ref.match(/class="divider" x="([^"]+)" y="[^"]+" width="([^"]+)"/);
if (refDiv1) {
  const x1 = +refDiv1[1], w1 = +refDiv1[2];
  console.log(`\nRef div1 right edge: ${x1+w1}`);
  console.log(`Expected div2 x (no gap): ${x1+w1}`);
}
