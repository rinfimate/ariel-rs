import { readFileSync } from 'fs';
const svg = readFileSync('./grammar/rust_default/state.svg', 'utf8');
const ref = readFileSync('./grammar/ref_default/state.svg', 'utf8');

function measure(svg, label) {
  const concIdx = svg.indexOf('Concurrent');
  const section = svg.substring(concIdx, concIdx + 6000);

  const divs = [...section.matchAll(/class="divider" x="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/g)];
  const circles = [...section.matchAll(/class="state-start" cx="([^"]+)" cy="([^"]+)" r="([^"]+)"/g)];
  const boxes = [...section.matchAll(/class="basic" x="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/g)];

  console.log(`\n=== ${label} ===`);
  divs.slice(0,1).forEach(d => {
    const rectTop = +d[2], rectH = +d[4], rectBot = rectTop + rectH;
    console.log(`Divider rect: top=${rectTop} bottom=${rectBot} height=${rectH}`);

    const c = circles[0];
    const b = boxes[0];
    if (c) {
      const circTop = +c[2] - +c[3];
      const circBot = +c[2] + +c[3];
      const topPad = circTop - rectTop;
      console.log(`  Circle: cy=${c[2]} r=${c[3]} → top=${circTop.toFixed(1)} bottom=${circBot.toFixed(1)}`);
      console.log(`  TOP padding (rect→circle_top): ${topPad.toFixed(1)}px`);
    }
    if (b) {
      const boxBot = +b[2] + +b[4];
      const botPad = rectBot - boxBot;
      console.log(`  A/B box: y=${b[2]} h=${b[4]} → bottom=${boxBot.toFixed(1)}`);
      console.log(`  BOTTOM padding (box_bot→rect_bot): ${botPad.toFixed(1)}px`);
    }
  });
}

measure(svg, 'OURS');
measure(ref, 'REFERENCE');
