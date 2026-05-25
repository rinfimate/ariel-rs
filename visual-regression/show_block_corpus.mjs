import { readFileSync } from 'fs';
const c = JSON.parse(readFileSync('grammar_corpus/grammar_corpus.json', 'utf8'));
console.log(c.block);
