import { readFileSync } from 'fs';
const c = JSON.parse(readFileSync('corpus/corpus.json', 'utf8'));
console.log('=== block_basic ===');
console.log(c.block_basic);
console.log('=== block_arrows ===');
console.log(c.block_arrows);
