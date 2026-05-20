import { Graph } from './node_modules/dagre-d3-es/src/graphlib/index.js';
import { layout } from './node_modules/dagre-d3-es/src/dagre/index.js';

// Try with width=0 for sp1/sp2 (labelRect may render as 0-dimension)
const g = new Graph({ multigraph: true, directed: true, compound: false });
g.setGraph({ rankdir:'TB', nodesep:140, edgesep:100, ranksep:101, marginx:8, marginy:8 });
g.setDefaultEdgeLabel(() => ({}));

g.setNode('PERSON',  { width:107.559, height:84 });
g.setNode('ADDRESS', { width:117.339, height:84 });
g.setNode('CITY',    { width:100,     height:84 });

const sp1='PERSON---PERSON---1', sp2='PERSON---PERSON---2';
// Try with width=0.1 (the rect in the reference SVG is 0.1×0.1)
g.setNode(sp1, { width:0.1, height:0.1 });
g.setNode(sp2, { width:0.1, height:0.1 });
g.setEdge('PERSON', sp1, {label:''}, 'cyc0');
g.setEdge(sp1, sp2, {label:'is married to'}, 'cyc1');
g.setEdge(sp2, 'PERSON', {label:''}, 'cyc2');
g.setEdge('PERSON','ADDRESS',{label:'lives at'},'e1');
g.setEdge('ADDRESS','CITY',{label:'is in'},'e2');

layout(g);

console.log('PERSON:', g.node('PERSON').x.toFixed(3), g.node('PERSON').y.toFixed(3));
console.log('sp1:', g.node(sp1).x.toFixed(3), g.node(sp1).y.toFixed(3));
console.log('sp2:', g.node(sp2).x.toFixed(3), g.node(sp2).y.toFixed(3));
console.log('ADDRESS:', g.node('ADDRESS').x.toFixed(3));
const e0 = g.edge('PERSON', sp1, 'cyc0');
const e1 = g.edge(sp1, sp2, 'cyc1');
const e2 = g.edge(sp2, 'PERSON', 'cyc2');
console.log('path1:', JSON.stringify(e0?.points?.map(p=>({x:+p.x.toFixed(3),y:+p.y.toFixed(3)}))));
console.log('path2:', JSON.stringify(e1?.points?.map(p=>({x:+p.x.toFixed(3),y:+p.y.toFixed(3)}))));
console.log('path3:', JSON.stringify(e2?.points?.map(p=>({x:+p.x.toFixed(3),y:+p.y.toFixed(3)}))));
