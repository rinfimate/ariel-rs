#!/usr/bin/env node
// Audit: for every WARN/FAIL diagram, list specific positional / dimensional
// differences between ref/<name>.svg and fidelity/<name>.svg.
// Looks for: edgeLabel translates, node transforms, rect width/height, divider y,
// fork/join rect dimensions, and inner label group translates.

import fs from 'node:fs';
import path from 'node:path';

const reportPath = path.join(import.meta.dirname, 'report.json');
const report = JSON.parse(fs.readFileSync(reportPath, 'utf8'));

const targets = report.results.filter(r => r.status !== 'PASS').map(r => r.name);

function extractAll(svg, regex) {
  const out = [];
  let m;
  while ((m = regex.exec(svg)) !== null) out.push(m);
  return out;
}

function summarize(name) {
  const refPath = path.join(import.meta.dirname, 'ref', `${name}.svg`);
  const ourPath = path.join(import.meta.dirname, 'fidelity', `${name}.svg`);
  if (!fs.existsSync(refPath) || !fs.existsSync(ourPath)) return null;
  const ref = fs.readFileSync(refPath, 'utf8');
  const our = fs.readFileSync(ourPath, 'utf8');

  const diffs = [];

  // Edge label outer translates
  const reEL = /edgeLabel"\s+transform="translate\(([\d.\-eE]+),\s*([\d.\-eE]+)\)"/g;
  const refEL = extractAll(ref, reEL).map(m => [+m[1], +m[2]]);
  const ourEL = extractAll(our, reEL).map(m => [+m[1], +m[2]]);
  if (refEL.length === ourEL.length) {
    for (let i = 0; i < refEL.length; i++) {
      const dx = ourEL[i][0] - refEL[i][0];
      const dy = ourEL[i][1] - refEL[i][1];
      if (Math.abs(dx) > 0.5 || Math.abs(dy) > 0.5) {
        diffs.push(`edgeLabel #${i}: ours=(${ourEL[i].map(v=>v.toFixed(2))}) ref=(${refEL[i].map(v=>v.toFixed(2))}) Δ=(${dx.toFixed(2)},${dy.toFixed(2)})`);
      }
    }
  } else {
    diffs.push(`edgeLabel count: ours=${ourEL.length} ref=${refEL.length}`);
  }

  // Inner label group translates within edgeLabel (the -10.5 / -8.5 issue)
  const reInner = /edgeLabel"[^>]*>[\s\S]{0,400}?<g\s+class="label"[^>]*transform="translate\(([\d.\-eE]+),\s*([\d.\-eE]+)\)"/g;
  const refInner = extractAll(ref, reInner).map(m => [+m[1], +m[2]]);
  const ourInner = extractAll(our, new RegExp(reInner.source, 'g')).map(m => [+m[1], +m[2]]);
  if (refInner.length === ourInner.length) {
    for (let i = 0; i < refInner.length; i++) {
      const dy = ourInner[i][1] - refInner[i][1];
      if (Math.abs(dy) > 0.5) {
        diffs.push(`edgeLabel #${i} inner-g y: ours=${ourInner[i][1].toFixed(2)} ref=${refInner[i][1].toFixed(2)} Δy=${dy.toFixed(2)}`);
      }
    }
  }

  // Node transforms (id="...-<name>" transform="translate(...)")
  const reNode = /id="([^"]*?(?:state|classId|flowchart|entity)-[^"]+)"[^>]*transform="translate\(([\d.\-eE]+),\s*([\d.\-eE]+)\)"/g;
  const refNodes = new Map();
  const ourNodes = new Map();
  for (const m of extractAll(ref, reNode)) {
    const key = m[1].replace(/^[^-]*-[^-]*-/, '').replace(/-\d+$/, '');
    refNodes.set(key, [+m[2], +m[3]]);
  }
  for (const m of extractAll(our, new RegExp(reNode.source, 'g'))) {
    const key = m[1].replace(/^[^-]*-/, '').replace(/-\d+$/, '');
    ourNodes.set(key, [+m[2], +m[3]]);
  }
  for (const [k, refXY] of refNodes) {
    const ourXY = ourNodes.get(k);
    if (!ourXY) continue;
    const dx = ourXY[0] - refXY[0], dy = ourXY[1] - refXY[1];
    if (Math.abs(dx) > 0.5 || Math.abs(dy) > 0.5) {
      diffs.push(`node ${k}: Δ=(${dx.toFixed(2)},${dy.toFixed(2)})`);
    }
  }

  // Fork-join rect dimensions
  const reFork = /class="fork-join"\s+x="([\d.\-eE]+)"\s+y="([\d.\-eE]+)"\s+width="([\d.\-eE]+)"\s+height="([\d.\-eE]+)"/g;
  const ourForks = extractAll(our, reFork);
  for (const m of ourForks) {
    diffs.push(`our fork: x=${m[1]} y=${m[2]} w=${m[3]} h=${m[4]}`);
  }

  return diffs;
}

for (const name of targets) {
  const diffs = summarize(name);
  if (diffs && diffs.length) {
    console.log(`\n=== ${name} ===`);
    diffs.slice(0, 12).forEach(d => console.log('  ' + d));
    if (diffs.length > 12) console.log(`  ... (${diffs.length - 12} more)`);
  }
}
