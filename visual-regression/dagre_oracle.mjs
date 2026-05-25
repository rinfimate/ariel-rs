/**
 * dagre_oracle.mjs — dagre-js layout oracle for fidelity testing.
 *
 * Protocol: line-delimited JSON over stdin/stdout.
 *   stdin  each line: graph spec (see below)
 *   stdout each line: layout result (see below)
 *
 * Input schema:
 * {
 *   "graph": { "rankdir": "LR", "nodesep": 50, "ranksep": 50, "marginx": 8, "marginy": 8 },
 *   "nodes": [{ "id": "A", "width": 120, "height": 54, "parent": "sg1" }, ...],
 *   "edges": [{ "v": "A", "w": "B", "width": 0, "height": 0, "minlen": 1, "weight": 1 }, ...]
 * }
 *
 * Output schema:
 * {
 *   "nodes": [{ "id": "A", "x": 100, "y": 75, "width": 120, "height": 54 }, ...],
 *   "edges": [{ "v": "A", "w": "B", "x": 100, "y": 50, "points": [{x,y}, ...] }, ...]
 * }
 *
 * Uses dagre-d3-es which ships dagre-js internals, matching Mermaid's dependency.
 *
 * Usage: node dagre_oracle.mjs   (communicates via stdin/stdout)
 */

import { layout } from 'dagre-d3-es/src/dagre/index.js';
import { Graph } from 'dagre-d3-es/src/graphlib/index.js';
import * as readline from 'readline';

const rl = readline.createInterface({ input: process.stdin, terminal: false });

rl.on('line', (line) => {
  const trimmed = line.trim();
  if (!trimmed) return;

  let req;
  try {
    req = JSON.parse(trimmed);
  } catch (e) {
    process.stdout.write(JSON.stringify({ error: `bad JSON: ${e.message}` }) + '\n');
    return;
  }

  try {
    const g = new Graph({ multigraph: true, compound: true });

    // Graph-level options
    const gl = req.graph ?? {};
    g.setGraph({
      rankdir:  gl.rankdir  ?? 'LR',
      nodesep:  gl.nodesep  ?? 50,
      ranksep:  gl.ranksep  ?? 50,
      marginx:  gl.marginx  ?? 8,
      marginy:  gl.marginy  ?? 8,
    });
    g.setDefaultEdgeLabel(() => ({}));

    // Nodes
    const nodeIntersect = new Map();
    for (const node of (req.nodes ?? [])) {
      g.setNode(node.id, { width: node.width, height: node.height, label: node.id });
      if (node.parent) {
        g.setParent(node.id, node.parent);
      }
      if (node.intersect_type) {
        nodeIntersect.set(node.id, node.intersect_type);
      }
    }

    // Edges
    for (const edge of (req.edges ?? [])) {
      const label = {
        width:  edge.width  ?? 0,
        height: edge.height ?? 0,
        minlen: edge.minlen ?? 1,
        weight: edge.weight ?? 1,
      };
      if (edge.labelpos    != null) label.labelpos    = edge.labelpos;
      if (edge.labeloffset != null) label.labeloffset = edge.labeloffset;
      g.setEdge(edge.v, edge.w, label, edge.name ?? undefined);
    }

    layout(g);

    // Collect results
    const nodes = g.nodes().map((id) => {
      const n = g.node(id);
      return { id, x: n.x, y: n.y, width: n.width, height: n.height };
    });

    // Shape intersection: dagre returns edge points clipped to each node's
    // bounding rectangle. For diamond / circle nodes Mermaid post-processes
    // the first / last segment to clip against the actual polygon / ellipse.
    // We replicate that here so the fidelity oracle matches the live renderer.
    function intersectDiamond(node, point) {
      const cx = node.x, cy = node.y, hw = node.width / 2, hh = node.height / 2;
      let dx = point.x - cx, dy = point.y - cy;
      if (dx === 0 && dy === 0) return { x: cx, y: cy };
      const t = 1 / (Math.abs(dx) / hw + Math.abs(dy) / hh);
      return { x: cx + dx * t, y: cy + dy * t };
    }
    function intersectEllipse(node, point) {
      const cx = node.x, cy = node.y, hw = node.width / 2, hh = node.height / 2;
      const dx = point.x - cx, dy = point.y - cy;
      if (dx === 0 && dy === 0) return { x: cx, y: cy };
      const t = 1 / Math.sqrt((dx * dx) / (hw * hw) + (dy * dy) / (hh * hh));
      return { x: cx + dx * t, y: cy + dy * t };
    }
    function clipEndpoint(nodeId, neighborPoint) {
      const t = nodeIntersect.get(nodeId);
      if (!t) return null;
      const n = g.node(nodeId);
      if (!n || n.x == null) return null;
      if (t === 'diamond') return intersectDiamond(n, neighborPoint);
      if (t === 'circle' || t === 'ellipse') return intersectEllipse(n, neighborPoint);
      return null;
    }

    const edges = g.edges().map((e) => {
      const lbl = g.edge(e);
      const pts = (lbl.points ?? []).map(p => ({ x: p.x, y: p.y }));
      if (pts.length >= 2) {
        const startClip = clipEndpoint(e.v, pts[1]);
        if (startClip) pts[0] = startClip;
        const endClip = clipEndpoint(e.w, pts[pts.length - 2]);
        if (endClip) pts[pts.length - 1] = endClip;
      }
      return {
        v:      e.v,
        w:      e.w,
        name:   e.name ?? null,
        x:      lbl.x ?? null,
        y:      lbl.y ?? null,
        points: pts,
      };
    });

    const gGraph = g.graph();
    process.stdout.write(JSON.stringify({
      nodes,
      edges,
      graph: { width: gGraph.width ?? 0, height: gGraph.height ?? 0 },
    }) + '\n');
  } catch (e) {
    process.stdout.write(JSON.stringify({ error: e.message, nodes: [], edges: [] }) + '\n');
  }
});

rl.on('close', () => process.exit(0));
