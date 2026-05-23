import { readFileSync } from 'fs';

function analyze(file, label) {
  const svg = readFileSync(file, 'utf8');
  console.log('\n===', label, '===');
  // Find Moving outer rect
  const mIdx = svg.indexOf('id="mermaid-svg-Moving"');
  if (mIdx < 0) { console.log('Moving not found'); return; }
  // Search backwards for outer rect
  const outerM = svg.slice(Math.max(0, mIdx-500), mIdx+200).match(/<rect[^>]*class="outer"[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"/);
  if (outerM) console.log('Moving outer: x=' + outerM[1] + ' y=' + outerM[2] + ' w=' + outerM[3] + ' h=' + outerM[4]);

  // Find inner rect
  const innerM = svg.slice(Math.max(0, mIdx-500), mIdx+400).match(/<rect[^>]*class="inner"[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"/);
  if (innerM) console.log('Moving inner: x=' + innerM[1] + ' y=' + innerM[2] + ' w=' + innerM[3] + ' h=' + innerM[4]);

  // Find the Moving sub-graph translate group
  const transRe = /transform="translate\(([^,]+),([^)]+)\)"/g;
  let m;
  const translates = [];
  while ((m = transRe.exec(svg)) !== null) {
    translates.push({x: parseFloat(m[1]), y: parseFloat(m[2]), pos: m.index});
  }
  // Find translates near Moving (within 2000 chars of Moving id)
  const nearMoving = translates.filter(t => Math.abs(t.pos - mIdx) < 3000);
  console.log('Translates near Moving:', JSON.stringify(nearMoving));

  // Find Slow and Fast state nodes
  const slowIdx = svg.indexOf('id="mermaid-svg-Slow"');
  const fastIdx = svg.indexOf('id="mermaid-svg-Fast"');
  if (slowIdx > 0) {
    const slowRect = svg.slice(slowIdx, slowIdx+200).match(/<rect[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"/);
    if (slowRect) console.log('Slow rect (relative to translate):', 'x=' + slowRect[1], 'y=' + slowRect[2], 'w=' + slowRect[3], 'h=' + slowRect[4]);
  }
  if (fastIdx > 0) {
    const fastRect = svg.slice(fastIdx, fastIdx+200).match(/<rect[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"/);
    if (fastRect) console.log('Fast rect (relative to translate):', 'x=' + fastRect[1], 'y=' + fastRect[2], 'w=' + fastRect[3], 'h=' + fastRect[4]);
  }
}

analyze('visual-regression/grammar/rust_default/state.svg', 'OURS');
analyze('visual-regression/grammar/ref_default/state.svg', 'REF');
