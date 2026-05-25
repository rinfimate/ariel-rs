import { Resvg } from '@resvg/resvg-js';
import { readFileSync, writeFileSync } from 'fs';
const svg = readFileSync('ref/live_editor_block.svg');
try {
  const resvg = new Resvg(svg, { fitTo: { mode: 'width', value: 1200 }, font: { loadSystemFonts: true } });
  const data = resvg.render().asPng();
  console.log('SUCCESS, size:', data.length);
  writeFileSync('ref/live_editor_block.png', data);
  console.log('Written!');
} catch(e) {
  console.error('ERROR:', e.message);
}
