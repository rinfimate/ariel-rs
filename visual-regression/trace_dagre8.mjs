import { Graph } from './node_modules/dagre-d3-es/src/graphlib/index.js';
import { layout } from './node_modules/dagre-d3-es/src/dagre/index.js';

// Try different label widths for edgeMid to find combination giving sp1≈46.5 AND PERSON≈126.5
for (const lw of [0, 10, 20, 30, 40, 50, 60, 70, 77]) {
    const g = new Graph({ multigraph: true, directed: true, compound: false });
    g.setGraph({ rankdir:'TB', nodesep:140, edgesep:100, ranksep:101, marginx:8, marginy:8 });
    g.setDefaultEdgeLabel(() => ({}));
    g.setNode('PERSON', { width:107.578, height:84 });
    g.setNode('ADDRESS', { width:117.359, height:84 });
    g.setNode('CITY', { width:100, height:84 });
    const sp1='sp1', sp2='sp2';
    g.setNode(sp1, { width:0.1, height:0.1 });
    g.setNode(sp2, { width:0.1, height:0.1 });
    g.setEdge('PERSON', sp1, {}, 'cyc0');
    const edgeMid = lw > 0 ? {width: lw, height: 21, labelpos:'c'} : {};
    g.setEdge(sp1, sp2, edgeMid, 'cyc1');
    g.setEdge(sp2, 'PERSON', {}, 'cyc2');
    g.setEdge('PERSON','ADDRESS',{},'e1');
    g.setEdge('ADDRESS','CITY',{},'e2');
    layout(g);
    console.log(`lw=${lw}: PERSON=${g.node('PERSON').x.toFixed(3)} sp1=${g.node(sp1).x.toFixed(3)} ADDRESS_y=${g.node('ADDRESS').y.toFixed(0)}`);
}

// Try width=0
const g2 = new Graph({ multigraph: true, directed: true, compound: false });
g2.setGraph({ rankdir:'TB', nodesep:140, edgesep:100, ranksep:101, marginx:8, marginy:8 });
g2.setDefaultEdgeLabel(() => ({}));
g2.setNode('PERSON', { width:107.578, height:84 });
g2.setNode('ADDRESS', { width:117.359, height:84 });
g2.setNode('CITY', { width:100, height:84 });
const sp1b='sp1b', sp2b='sp2b';
g2.setNode(sp1b, { width:0, height:0 });
g2.setNode(sp2b, { width:0, height:0 });
g2.setEdge('PERSON', sp1b, {}, 'cyc0');
g2.setEdge(sp1b, sp2b, {width:77, height:21, labelpos:'c'}, 'cyc1');
g2.setEdge(sp2b, 'PERSON', {}, 'cyc2');
g2.setEdge('PERSON','ADDRESS',{},'e1');
g2.setEdge('ADDRESS','CITY',{},'e2');
layout(g2);
console.log(`width=0: PERSON=${g2.node('PERSON').x.toFixed(3)} sp1=${g2.node(sp1b).x.toFixed(3)}`);
