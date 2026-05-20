import { Graph } from './node_modules/dagre-d3-es/src/graphlib/index.js';
import { layout } from './node_modules/dagre-d3-es/src/dagre/index.js';

// Exact replication of mermaid's full pipeline for er_optional
// Node dims come from browser rendering (foreignObject getBoundingClientRect)
// Edge label dims: mermaid measures BEFORE dagre only for some edges
// sp1/sp2 are "labelRect" nodes → width rendered as 0.1 (tiny rect)

const g = new Graph({ multigraph: true, directed: true, compound: false });
// ranksep=80 as mermaid sets it (makeSpaceForEdgeLabels will halve it internally)
g.setGraph({ rankdir:'TB', nodesep:140, edgesep:100, ranksep:80, marginx:8, marginy:8 });
g.setDefaultEdgeLabel(() => ({}));

// Entity widths from browser rendering (matches reference SVG values)
g.setNode('PERSON',  { width:107.578125, height:84 });
g.setNode('ADDRESS', { width:117.359375, height:84 });
g.setNode('CITY',    { width:100,        height:84 });

const sp1='PERSON---PERSON---1', sp2='PERSON---PERSON---2';
// labelRect nodes: mermaid renders them as 0.1×0.1 rects
g.setNode(sp1, { width:0.1, height:0.1 });
g.setNode(sp2, { width:0.1, height:0.1 });

// Edges: mermaid measures label dims in browser BEFORE setting up the cyclic edges
// The "is married to" label: measured by browser as width=77.031, height=21
g.setEdge('PERSON', sp1, { width:0, height:0 }, 'cyc0');
g.setEdge(sp1, sp2,  { width:77.031, height:21, labelpos:'c' }, 'cyc1');
g.setEdge(sp2, 'PERSON', { width:0, height:0 }, 'cyc2');
g.setEdge('PERSON','ADDRESS', { width:43.578, height:21, labelpos:'c' }, 'e1');
g.setEdge('ADDRESS','CITY',   { width:24.906, height:21, labelpos:'c' }, 'e2');

layout(g);
console.log('PERSON:', g.node('PERSON').x.toFixed(3), g.node('PERSON').y.toFixed(3));
console.log('ADDRESS:', g.node('ADDRESS').x.toFixed(3), g.node('ADDRESS').y.toFixed(3));
console.log('CITY:', g.node('CITY').x.toFixed(3), g.node('CITY').y.toFixed(3));
console.log('sp1:', g.node(sp1).x.toFixed(3), g.node(sp1).y.toFixed(3));
console.log('sp2:', g.node(sp2).x.toFixed(3), g.node(sp2).y.toFixed(3));

const e0=g.edge('PERSON',sp1,'cyc0'), e1=g.edge(sp1,sp2,'cyc1'), e2=g.edge(sp2,'PERSON','cyc2');
console.log('path1:', JSON.stringify(e0?.points?.map(p=>({x:+p.x.toFixed(3),y:+p.y.toFixed(3)}))));
console.log('path2:', JSON.stringify(e1?.points?.map(p=>({x:+p.x.toFixed(3),y:+p.y.toFixed(3)}))));
console.log('path3:', JSON.stringify(e2?.points?.map(p=>({x:+p.x.toFixed(3),y:+p.y.toFixed(3)}))));
