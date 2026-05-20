import { layout } from './node_modules/dagre-d3-es/src/dagre/index.js';
import { Graph } from './node_modules/dagre-d3-es/src/graphlib/index.js';

// Test: does layout() call makeSpaceForEdgeLabels internally?
// If so, ranksep=80 should effectively behave as if edges have minlen=2
const g = new Graph({ multigraph: true, directed: true, compound: false });
g.setGraph({ rankdir:'TB', nodesep:140, edgesep:100, ranksep:80, marginx:8, marginy:8 });
g.setDefaultEdgeLabel(() => ({}));

g.setNode('PERSON',  { width:107.578, height:84 });
g.setNode('ADDRESS', { width:117.359, height:84 });
g.setNode('CITY',    { width:100,     height:84 });

const sp1='PERSON---PERSON---1', sp2='PERSON---PERSON---2';
g.setNode(sp1, { width:0.1, height:0.1 });
g.setNode(sp2, { width:0.1, height:0.1 });
g.setEdge('PERSON', sp1, {}, 'cyc0');
g.setEdge(sp1, sp2, {}, 'cyc1');  // NO label dims
g.setEdge(sp2, 'PERSON', {}, 'cyc2');
g.setEdge('PERSON','ADDRESS',{},'e1');
g.setEdge('ADDRESS','CITY',{},'e2');

layout(g);
console.log('ranksep=80, no label dims:');
console.log('PERSON:', g.node('PERSON').x.toFixed(3), g.node('PERSON').y.toFixed(3));
console.log('ADDRESS:', g.node('ADDRESS').x.toFixed(3), g.node('ADDRESS').y.toFixed(3));
console.log('sp1:', g.node(sp1).x.toFixed(3), g.node(sp1).y.toFixed(3));
