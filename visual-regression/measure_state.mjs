import { readFileSync } from 'fs';

function measure(file, label) {
  const svg = readFileSync(file, 'utf8');
  console.log('\n======', label, '======');

  // Collect all transforms in document order
  const allTrans = [...svg.matchAll(/transform="translate\(([^,)]+)[, ]([^)]+)\)"/g)]
    .map(m => ({ tx: parseFloat(m[1]), ty: parseFloat(m[2]), pos: m.index }));

  // Find translate that applies at a given position (last transform before pos)
  function resolveTranslate(pos) {
    const before = allTrans.filter(t => t.pos < pos);
    if (!before.length) return { tx: 0, ty: 0 };
    return before[before.length - 1];
  }

  // ── MOVING ──────────────────────────────────────────────────────────────────
  const movTextIdx = svg.indexOf('>Moving<');
  const movCtx = svg.slice(Math.max(0, movTextIdx - 900), movTextIdx + 700);
  const movCtxOff = Math.max(0, movTextIdx - 900); // offset into full svg

  const outerM = svg.slice(0, movTextIdx + 700).match(/<rect[^>]*class="outer"[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"/);
  const innerM = svg.slice(0, movTextIdx + 700).match(/<rect[^>]*class="inner"[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"/);

  if (outerM && innerM) {
    const outerIdx = svg.indexOf(outerM[0]);
    const innerIdx = svg.indexOf(innerM[0]);
    const outerTr = resolveTranslate(outerIdx);
    const innerTr = resolveTranslate(innerIdx);

    const ox = parseFloat(outerM[1]) + outerTr.tx;
    const oy = parseFloat(outerM[2]) + outerTr.ty;
    const ow = parseFloat(outerM[3]);
    const oh = parseFloat(outerM[4]);
    const ix = parseFloat(innerM[1]) + innerTr.tx;
    const iy = parseFloat(innerM[2]) + innerTr.ty;
    const iw = parseFloat(innerM[3]);
    const ih = parseFloat(innerM[4]);
    const titleArea = parseFloat(innerM[2]) - parseFloat(outerM[2]);
    const botGap = oh - ih - titleArea;

    console.log('Moving outer:   x=' + ox.toFixed(2) + '  y=' + oy.toFixed(2) + '  w=' + ow.toFixed(3) + '  h=' + oh);
    console.log('Moving inner:   x=' + ix.toFixed(2) + '  y=' + iy.toFixed(2) + '  w=' + iw.toFixed(3) + '  h=' + ih);
    console.log('  title_area=' + titleArea + '  bottom_gap=' + botGap);
    console.log('  inner_right_pad=' + (ow - iw).toFixed(2) + '  inner_left_pad=0');

    // Sub-graph content translate (inside the inner rect)
    const afterInner = svg.slice(innerIdx);
    const subTrM = afterInner.match(/transform="translate\(([^,)]+)[, ]([^)]+)\)"/);
    if (subTrM) {
      console.log('  Sub-graph translate: tx=' + parseFloat(subTrM[1]).toFixed(2) + ' ty=' + parseFloat(subTrM[2]).toFixed(2));
    }
  }

  // Slow and Fast
  for (const name of ['Slow', 'Fast']) {
    const idx = svg.indexOf('>' + name + '<');
    if (idx < 0) { console.log('  ' + name + ': not found'); continue; }
    const trs = allTrans.filter(t => t.pos < idx);
    const lastTr = trs.length ? trs[trs.length - 1] : { tx: 0, ty: 0 };
    const rctx = svg.slice(Math.max(0, idx - 300), idx + 100);
    const r = rctx.match(/<rect[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"/);
    if (r) {
      const rx = parseFloat(r[1]), ry = parseFloat(r[2]), rw = parseFloat(r[3]), rh = parseFloat(r[4]);
      console.log('  ' + name + ' rect local:   x=' + rx.toFixed(2) + ' y=' + ry.toFixed(2) + ' w=' + rw.toFixed(2) + ' h=' + rh);
      console.log('    node_translate: (' + lastTr.tx.toFixed(2) + ', ' + lastTr.ty.toFixed(2) + ')');
    }
  }

  // ── CONCURRENT ──────────────────────────────────────────────────────────────
  const allOuterRects = [...svg.matchAll(/<rect[^>]*class="outer"[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"/g)];
  const allInnerRects = [...svg.matchAll(/<rect[^>]*class="inner"[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"/g)];
  // Pick the tallest outer rect as concurrent
  const bigOuter = allOuterRects.sort((a, b) => parseFloat(b[4]) - parseFloat(a[4]))[0];
  const bigInner = allInnerRects.sort((a, b) => parseFloat(b[4]) - parseFloat(a[4]))[0];

  if (bigOuter && bigInner) {
    const oIdx = bigOuter.index;
    const iIdx = bigInner.index;
    const oTr = resolveTranslate(oIdx);
    const iTr = resolveTranslate(iIdx);

    const cox = parseFloat(bigOuter[1]) + oTr.tx;
    const coy = parseFloat(bigOuter[2]) + oTr.ty;
    const cow = parseFloat(bigOuter[3]);
    const coh = parseFloat(bigOuter[4]);
    const cix = parseFloat(bigInner[1]) + iTr.tx;
    const ciy = parseFloat(bigInner[2]) + iTr.ty;
    const ciw = parseFloat(bigInner[3]);
    const cih = parseFloat(bigInner[4]);
    const titleArea = parseFloat(bigInner[2]) - parseFloat(bigOuter[2]);
    const botGap = coh - cih - titleArea;

    console.log('\nConcurrent outer: x=' + cox.toFixed(2) + ' y=' + coy.toFixed(2) + ' w=' + cow.toFixed(3) + ' h=' + coh);
    console.log('Concurrent inner: x=' + cix.toFixed(2) + ' y=' + ciy.toFixed(2) + ' w=' + ciw.toFixed(3) + ' h=' + cih);
    console.log('  title_area=' + titleArea + '  bottom_gap=' + botGap);

    // Divider rects
    const divRects = [...svg.matchAll(/<rect[^>]*class="divider"[^>]*x="([^"]+)"[^>]*y="([^"]+)"[^>]*width="([^"]+)"[^>]*height="([^"]+)"/g)];
    divRects.forEach((d, i) => {
      const dTr = resolveTranslate(d.index);
      const dx = parseFloat(d[1]) + dTr.tx;
      const dy = parseFloat(d[2]) + dTr.ty;
      const dw = parseFloat(d[3]);
      const dh = parseFloat(d[4]);
      const leftPad = dx - cix;
      const rightPad = cix + ciw - (dx + dw);
      const topPad = dy - ciy;
      const botPad = ciy + cih - (dy + dh);
      console.log('  div' + (i+1) + ': abs=(x=' + dx.toFixed(2) + ' y=' + dy.toFixed(2) + ' w=' + dw.toFixed(2) + ' h=' + dh + ')');
      console.log('    pads: left=' + leftPad.toFixed(1) + ' right=' + rightPad.toFixed(1) + ' top=' + topPad.toFixed(1) + ' bot=' + botPad.toFixed(1));
    });

    // Find circles within concurrent inner area and compute content padding
    const circles = [...svg.matchAll(/<circle[^>]*(?:cx="([^"]+)"[^>]*cy="([^"]+)"|cy="([^"]+)"[^>]*cx="([^"]+)")[^>]*r="([^"]+)"/g)];
    const inConcCircles = circles.filter(c => {
      const tr = resolveTranslate(c.index);
      const acx = (parseFloat(c[1]||c[4]) + tr.tx);
      const acy = (parseFloat(c[2]||c[3]) + tr.ty);
      return acx >= cix - 5 && acx <= cix + ciw + 5 && acy >= ciy - 5 && acy <= ciy + cih + 5;
    });
    inConcCircles.forEach((c, i) => {
      const tr = resolveTranslate(c.index);
      const acx = parseFloat(c[1]||c[4]) + tr.tx;
      const acy = parseFloat(c[2]||c[3]) + tr.ty;
      const r = parseFloat(c[5]);
      const topPad = acy - r - ciy;
      const botPad = ciy + cih - acy - r;
      console.log('  circle' + (i+1) + ': abs_cx=' + acx.toFixed(2) + ' abs_cy=' + acy.toFixed(2) + ' r=' + r + ' → top_pad=' + topPad.toFixed(1) + ' bot_pad=' + botPad.toFixed(1));
    });
  }
}

measure('visual-regression/grammar/rust_default/state.svg', 'OURS');
measure('visual-regression/grammar/ref_default/state.svg', 'REF');
