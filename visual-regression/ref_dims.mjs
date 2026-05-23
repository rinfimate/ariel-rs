import { readFileSync } from 'fs';
const ref = readFileSync('./grammar/ref_default/state.svg', 'utf8');

const concIdx = ref.indexOf('Concurrent');
const section = ref.substring(concIdx - 50, concIdx + 500);

// Concurrent outer rect
const outer = section.match(/rect[^>]*width="([^"]+)"[^>]*height="([^"]+)"/);
console.log('Concurrent outer rect:');
if (outer) console.log('  width=' + outer[1] + ' height=' + outer[2]);

// Divider boxes
const divs = [...ref.matchAll(/class="divider" x="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/g)];
console.log('\nDivider boxes:');
divs.forEach((d,i) => console.log('  div'+i+': x='+d[1]+' y='+d[2]+' w='+d[3]+' h='+d[4]));

// Cluster-label (title height)
const label = section.match(/foreignObject[^>]*width="([^"]+)" height="([^"]+)"/);
if (label) console.log('\nConcurrent title foreignObject: w='+label[1]+' h='+label[2]);
