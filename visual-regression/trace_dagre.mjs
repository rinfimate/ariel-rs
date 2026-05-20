import { Graph } from './node_modules/dagre-d3-es/src/graphlib/index.js';
import { layout } from './node_modules/dagre-d3-es/src/dagre/index.js';

const g = new Graph({ multigraph: true, directed: true, compound: false });
g.setGraph({ rankdir:'TB', nodesep:140, edgesep:100, ranksep:101, marginx:8, marginy:8 });
g.setDefaultEdgeLabel(() => ({}));

// Entities
g.setNode('PERSON',  { width:107.559, height:84 });
g.setNode('ADDRESS', { width:117.339, height:84 });
g.setNode('CITY',    { width:100,     height:84 });

// Self-loop: PERSON }|..|{ PERSON
const sp1='PERSON---PERSON---1', sp2='PERSON---PERSON---2';
g.setNode(sp1, { width:10, height:10, shape:'labelRect', label:'', padding:0 });
g.setNode(sp2, { width:10, height:10, shape:'labelRect', label:'', padding:0 });
g.setEdge('PERSON', sp1, {arrowTypeEnd:'none', label:''}, 'cyc0');
g.setEdge(sp1, sp2, {arrowTypeStart:'none', arrowTypeEnd:'none', label:'is married to'}, 'cyc1');
g.setEdge(sp2, 'PERSON', {arrowTypeStart:'none', label:''}, 'cyc2');

// Regular edges
g.setEdge('PERSON','ADDRESS',{label:'lives at'},'e1');
g.setEdge('ADDRESS','CITY',{label:'is in'},'e2');

layout(g);

console.log('PERSON:', g.node('PERSON').x.toFixed(3), g.node('PERSON').y.toFixed(3));
console.log('ADDRESS:', g.node('ADDRESS').x.toFixed(3), g.node('ADDRESS').y.toFixed(3));
console.log('CITY:', g.node('CITY').x.toFixed(3), g.node('CITY').y.toFixed(3));
console.log('sp1:', g.node(sp1).x.toFixed(3), g.node(sp1).y.toFixed(3));
console.log('sp2:', g.node(sp2).x.toFixed(3), g.node(sp2).y.toFixed(3));

const e0 = g.edge('PERSON', sp1, 'cyc0');
const e1 = g.edge(sp1, sp2, 'cyc1');
const e2 = g.edge(sp2, 'PERSON', 'cyc2');
console.log('path1 points:', JSON.stringify(e0?.points?.map(p=>({x:+p.x.toFixed(3),y:+p.y.toFixed(3)}))));
console.log('path2 points:', JSON.stringify(e1?.points?.map(p=>({x:+p.x.toFixed(3),y:+p.y.toFixed(3)}))));
console.log('path3 points:', JSON.stringify(e2?.points?.map(p=>({x:+p.x.toFixed(3),y:+p.y.toFixed(3)}))));
