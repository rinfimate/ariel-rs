import { readFileSync } from 'fs';

for (const [label, file] of [['OURS', './grammar/rust_default/state.svg'], ['REF', './grammar/ref_default/state.svg']]) {
  const svg = readFileSync(file, 'utf8');
  console.log('\n' + label);

  // ── MOVING ──────────────────────────────────────────────────────────────────
  // Find the Moving cluster section
  const movPos = svg.indexOf('id="g-4-state-Moving-18"') !== -1
    ? svg.indexOf('id="g-4-state-Moving-18"')
    : svg.indexOf('>Moving outer:') !== -1 ? svg.indexOf('>Moving<') : svg.indexOf('"mermaid-svg-Moving"');
  const movSec = svg.slice(Math.max(0, movPos - 200), movPos + 1500);

  const outerH = movSec.match(/class="outer"[^>]*width="([\d.]+)"[^>]*height="([\d.]+)"/);
  const innerH = movSec.match(/class="inner"[^>]*[^>]*width="([\d.]+)"[^>]*height="([\d.]+)"/);
  const outerXY = movSec.match(/class="outer"[^>]*x="([\d.]+)"[^>]*y="([\d.]+)"/);
  const innerXY = movSec.match(/class="inner"[^>]*x="([\d.]+)"[^>]*y="([\d.]+)"/);

  // Parent translate (last one before Moving cluster)
  const beforeMov = svg.slice(0, movPos);
  const parentTr = [...beforeMov.matchAll(/transform="translate\(([\d.]+)[, ]+([\d.]+)\)"/g)].slice(-1)[0];

  console.log('  MOVING');
  if (parentTr) console.log('    parent translate : (' + parseFloat(parentTr[1]).toFixed(3) + ', ' + parseFloat(parentTr[2]).toFixed(3) + ')');
  if (outerXY && outerH) console.log('    outer            : x=' + parseFloat(outerXY[1]).toFixed(3) + ' y=' + parseFloat(outerXY[2]).toFixed(3) + ' w=' + parseFloat(outerH[1]).toFixed(3) + ' h=' + outerH[2]);
  if (innerXY && innerH) console.log('    inner            : x=' + parseFloat(innerXY[1]).toFixed(3) + ' y=' + parseFloat(innerXY[2]).toFixed(3) + ' w=' + parseFloat(innerH[1]).toFixed(3) + ' h=' + innerH[2]);
  if (outerXY && innerXY && outerH && innerH) {
    const titleArea = parseFloat(innerXY[2]) - parseFloat(outerXY[2]);
    const botGap = parseFloat(outerH[2]) - parseFloat(innerH[2]) - titleArea;
    console.log('    title area       : ' + titleArea);
    console.log('    bottom gap       : ' + botGap);
  }

  // Node positions (local to Moving sub-graph)
  const nodeTranslates = [...movSec.matchAll(/transform="translate\(([\d.-]+)[, ]+([\d.-]+)\)"/g)];
  const subGraphTr = nodeTranslates.find(t => parseFloat(t[2]) > 30); // first non-label translate
  if (subGraphTr) console.log('    sub-graph tr     : (' + parseFloat(subGraphTr[1]).toFixed(3) + ', ' + parseFloat(subGraphTr[2]).toFixed(3) + ')');

  // Slow/Fast nodes
  for (const name of ['Slow', 'Fast']) {
    const idx = svg.indexOf('>' + name + '<');
    if (idx < 0) continue;
    const ctx = svg.slice(Math.max(0, idx - 350), idx + 50);
    const trs = [...ctx.matchAll(/transform="translate\(([\d.-]+)[, ]+([\d.-]+)\)"/g)];
    const lastTr = trs.slice(-1)[0];
    const rect = ctx.match(/<rect[^>]*x="([\d.-]+)"[^>]*y="([\d.-]+)"[^>]*width="([\d.]+)"[^>]*height="([\d.]+)"/);
    if (lastTr) {
      const cx = parseFloat(lastTr[1]), cy = parseFloat(lastTr[2]);
      if (rect) {
        const rw = parseFloat(rect[3]), rh = parseFloat(rect[4]);
        console.log('    ' + name + ' node center   : (' + cx.toFixed(3) + ', ' + cy.toFixed(3) + ')  w=' + rw.toFixed(3) + ' h=' + rh);
      }
    }
  }

  // ── CONCURRENT ──────────────────────────────────────────────────────────────
  const concPos = svg.indexOf('data-id="Concurrent"') !== -1
    ? svg.indexOf('data-id="Concurrent"')
    : (() => { const i = svg.indexOf('class="outer"', svg.indexOf('h=329') !== -1 ? svg.indexOf('h=329') - 200 : 0); return i; })();

  // Find concurrent section by searching for outer h=329
  let concSec = '';
  const allOuters = [...svg.matchAll(/class="outer"[^>]*width="([\d.]+)"[^>]*height="([\d.]+)"/g)];
  const bigOuter = allOuters.find(m => parseFloat(m[2]) > 300);
  if (bigOuter) {
    const start = Math.max(0, bigOuter.index - 500);
    concSec = svg.slice(start, bigOuter.index + 5000);
  }

  const cOuter = concSec.match(/class="outer"[^>]*x="([\d.]+)"[^>]*y="([\d.]+)"[^>]*width="([\d.]+)"[^>]*height="([\d.]+)"/);
  const cInner = concSec.match(/class="inner"[^>]*x="([\d.]+)"[^>]*y="([\d.]+)"[^>]*width="([\d.]+)"[^>]*height="([\d.]+)"/);

  const beforeConc = svg.slice(0, bigOuter?.index ?? 0);
  const concParentTr = [...beforeConc.matchAll(/transform="translate\(([\d.]+)[, ]+([\d.]+)\)"/g)].slice(-1)[0];

  const divRects = [...concSec.matchAll(/class="divider"[^>]*x="([\d.]+)"[^>]*y="([\d.]+)"[^>]*width="([\d.]+)"[^>]*height="([\d.]+)"/g)];
  // Div group translates (within concurrent sub-graph)
  const divGroupTrs = [...concSec.matchAll(/transform="translate\(([\d.]+)[, ]+([\d.]+)\)"/g)]
    .filter(t => parseFloat(t[2]) > 20 && parseFloat(t[2]) < 100 && parseFloat(t[1]) > 20);

  console.log('  CONCURRENT');
  if (concParentTr) console.log('    parent translate : (' + parseFloat(concParentTr[1]).toFixed(3) + ', ' + parseFloat(concParentTr[2]).toFixed(3) + ')');
  if (cOuter) console.log('    outer            : x=' + parseFloat(cOuter[1]).toFixed(3) + ' y=' + parseFloat(cOuter[2]).toFixed(3) + ' w=' + parseFloat(cOuter[3]).toFixed(3) + ' h=' + cOuter[4]);
  if (cInner) console.log('    inner            : x=' + parseFloat(cInner[1]).toFixed(3) + ' y=' + parseFloat(cInner[2]).toFixed(3) + ' w=' + parseFloat(cInner[3]).toFixed(3) + ' h=' + cInner[4]);
  if (cOuter && cInner) {
    const ta = parseFloat(cInner[2]) - parseFloat(cOuter[2]);
    const bg = parseFloat(cOuter[4]) - parseFloat(cInner[4]) - ta;
    console.log('    title area       : ' + ta);
    console.log('    bottom gap       : ' + bg);
  }

  divGroupTrs.slice(0, 2).forEach((t, i) => {
    console.log('    div' + (i+1) + ' group tr    : (' + parseFloat(t[1]).toFixed(3) + ', ' + parseFloat(t[2]).toFixed(3) + ')');
  });
  divRects.slice(0, 2).forEach((d, i) => {
    console.log('    div' + (i+1) + ' rect        : x=' + parseFloat(d[1]).toFixed(3) + ' y=' + parseFloat(d[2]).toFixed(3) + ' w=' + parseFloat(d[3]).toFixed(3) + ' h=' + d[4]);
  });

  // Circles within concurrent section
  const concCircles = [...concSec.matchAll(/<circle[^>]*(?:cx="([\d.]+)"[^>]*cy="([\d.]+)"|cy="([\d.]+)"[^>]*cx="([\d.]+)")[^>]*r="([\d.]+)"/g)]
    .filter(c => parseFloat(c[5]) >= 5);
  const circleGroupTrs = [...concSec.matchAll(/transform="translate\(([\d.-]+)[, ]+([\d.-]+)\)"/g)]
    .filter(t => parseFloat(t[1]) > 30 && parseFloat(t[2]) > 30 && parseFloat(t[2]) < 200);
  circleGroupTrs.slice(0, 2).forEach((t, i) => {
    console.log('    circle' + (i+1) + ' tr      : (' + parseFloat(t[1]).toFixed(3) + ', ' + parseFloat(t[2]).toFixed(3) + ')  [node center in sub-graph space]');
  });
  concCircles.slice(0, 2).forEach((c, i) => {
    const cx = parseFloat(c[1]||c[4]), cy = parseFloat(c[2]||c[3]), r = parseFloat(c[5]);
    console.log('    circle' + (i+1) + '          : cx=' + cx.toFixed(3) + ' cy=' + cy.toFixed(3) + ' r=' + r);
  });

  // State boxes in concurrent section
  const concBoxes = [...concSec.matchAll(/<rect[^>]*x="([\d.-]+)"[^>]*y="([\d.-]+)"[^>]*width="([\d.]+)"[^>]*height="([\d.]+)"/g)]
    .filter(r => Math.abs(parseFloat(r[4]) - 40) < 2 && parseFloat(r[3]) > 10);
  concBoxes.slice(0, 2).forEach((b, i) => {
    console.log('    state box' + (i+1) + '      : x=' + parseFloat(b[1]).toFixed(3) + ' y=' + parseFloat(b[2]).toFixed(3) + ' w=' + parseFloat(b[3]).toFixed(3) + ' h=' + b[4]);
  });
}
