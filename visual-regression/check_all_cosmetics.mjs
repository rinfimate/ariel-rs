import { readFileSync } from 'fs';
const ref = readFileSync('./grammar/ref_default/state.svg', 'utf8');
const ours = readFileSync('./grammar/rust_default/state.svg', 'utf8');

// ── Concurrent divider boxes ──────────────────────────────────────────────
const getDivs = svg => [...svg.matchAll(/class="divider" x="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/g)]
  .map(d => ({x:+d[1], y:+d[2], w:+d[3], h:+d[4]}));

console.log('=== Concurrent divider boxes ===');
console.log('REF:', getDivs(ref).map(d=>`x=${d.x} y=${d.y} w=${d.w.toFixed(2)} h=${d.h}`).join(' | '));
console.log('OUR:', getDivs(ours).map(d=>`x=${d.x} y=${d.y} w=${d.w.toFixed(2)} h=${d.h}`).join(' | '));

// Concurrent outer cluster
const refConc = ref.match(/data-id="Concurrent"[^>]*><g><rect[^>]*x="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/);
if (refConc) console.log(`REF Concurrent outer: x=${refConc[1]} y=${refConc[2]} w=${refConc[3]} h=${refConc[4]}`);
const ourConc = ours.match(/id="mermaid-svg-Concurrent"><rect class="outer" x="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/);
if (ourConc) console.log(`OUR Concurrent outer: x=${ourConc[1]} y=${ourConc[2]} w=${ourConc[3]} h=${ourConc[4]}`);

// Internal nodes in dividers (start circles and A/B boxes)
console.log('\n=== Internal node positions in concurrent boxes ===');
// Ref - find start circles and A/B within Concurrent section
const concIdx = ref.indexOf('Concurrent');
const concSection = ref.substring(concIdx, concIdx + 3000);
const starts = [...concSection.matchAll(/class="state-start" r="([^"]+)"/g)].map(m=>m[1]);
const aBox = concSection.match(/class="basic label-container"[^>]*x="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/);
console.log(`REF start circle r: ${starts.slice(0,2).join(', ')}`);
if (aBox) console.log(`REF A/B box: x=${aBox[1]} y=${aBox[2]} w=${aBox[3]} h=${aBox[4]}`);

// Ours
const ourConcIdx = ours.indexOf('Concurrent');
const ourConcSection = ours.substring(ourConcIdx, ourConcIdx + 4000);
const ourStarts = [...ourConcSection.matchAll(/class="state-start" cx="([^"]+)" cy="([^"]+)" r="([^"]+)"/g)].map(m=>`cx=${m[1]} cy=${m[2]} r=${m[3]}`);
const ourABox = ourConcSection.match(/class="basic" x="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/);
console.log(`OUR start circles: ${ourStarts.slice(0,2).join(' | ')}`);
if (ourABox) console.log(`OUR A/B box: x=${ourABox[1]} y=${ourABox[2]} w=${ourABox[3]} h=${ourABox[4]}`);

// ── Moving composite ─────────────────────────────────────────────────────
console.log('\n=== Moving composite ===');
const refMovIdx = ref.indexOf('"Moving"');
if (refMovIdx > 0) {
  const ctx = ref.substring(refMovIdx, refMovIdx+600);
  const outer = ctx.match(/class="outer"[^>]*x="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/);
  const inner = ctx.match(/class="inner"[^>]*x="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/);
  if (outer) console.log(`REF Moving outer: x=${outer[1]} y=${outer[2]} w=${outer[3]} h=${outer[4]}`);
  if (inner) console.log(`REF Moving inner: x=${inner[1]} y=${inner[2]} w=${inner[3]} h=${inner[4]}`);
}
const ourMovIdx = ours.indexOf('mermaid-svg-Moving"');
if (ourMovIdx > 0) {
  const ctx = ours.substring(ourMovIdx, ourMovIdx+400);
  const outer = ctx.match(/class="outer"[^>]*x="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/);
  const inner = ctx.match(/class="inner"[^>]*x="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/);
  if (outer) console.log(`OUR Moving outer: x=${outer[1]} y=${outer[2]} w=${outer[3]} h=${outer[4]}`);
  if (inner) console.log(`OUR Moving inner: x=${inner[1]} y=${inner[2]} w=${inner[3]} h=${inner[4]}`);
}
