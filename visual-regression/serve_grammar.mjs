/**
 * serve_grammar.mjs — Static server for grammar regression HTML
 * Usage: node serve_grammar.mjs [port]
 * Then open http://localhost:3000/compare_grammar_default.html
 */

import { createServer } from 'http';
import { readFileSync, existsSync } from 'fs';
import { join, extname, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const PORT = parseInt(process.argv[2] ?? '3000');
const BASE = join(__dirname, 'grammar');

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.svg':  'image/svg+xml; charset=utf-8',
  '.png':  'image/png',
  '.json': 'application/json; charset=utf-8',
  '.js':   'application/javascript; charset=utf-8',
  '.css':  'text/css; charset=utf-8',
};

createServer((req, res) => {
  const url = decodeURIComponent(req.url.split('?')[0]);
  const path = join(BASE, url === '/' ? '/compare_grammar_default.html' : url);

  if (!existsSync(path)) {
    res.writeHead(404); res.end('Not found: ' + url); return;
  }

  const ext = extname(path).toLowerCase();
  res.writeHead(200, { 'Content-Type': MIME[ext] ?? 'application/octet-stream' });
  res.end(readFileSync(path));
}).listen(PORT, () => {
  console.log(`Grammar regression server: http://localhost:${PORT}/`);
  console.log('  default: http://localhost:'+PORT+'/compare_grammar_default.html');
  console.log('  dark:    http://localhost:'+PORT+'/compare_grammar_dark.html');
  console.log('  forest:  http://localhost:'+PORT+'/compare_grammar_forest.html');
  console.log('  neutral: http://localhost:'+PORT+'/compare_grammar_neutral.html');
});
