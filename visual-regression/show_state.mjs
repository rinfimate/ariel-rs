import puppeteer from 'puppeteer';
import { readFileSync, writeFileSync } from 'fs';

const svg = readFileSync('./grammar/rust_default/state.svg', 'utf8');
const vb = svg.match(/viewBox="([^"]+)"/)?.[1].split(/\s+/).map(Number) || [0,0,500,500];
const h = Math.ceil(1200 * vb[3] / vb[2]);
const browser = await puppeteer.launch({headless:true,args:['--no-sandbox']});
const page = await browser.newPage();
await page.setContent('<html><head><style>*{margin:0;padding:0}body{background:white}</style></head><body></body></html>');
await page.setViewport({width:1200,height:h,deviceScaleFactor:1});
await page.evaluate((s,hh)=>{
  document.body.innerHTML=s;
  const el=document.body.querySelector('svg');
  if(el){el.style.cssText=`display:block;width:1200px;height:${hh}px;`;el.setAttribute('width',1200);el.setAttribute('height',hh);}
},svg,h);
const png=await page.screenshot({clip:{x:0,y:0,width:1200,height:h},type:'png'});
writeFileSync('./grammar/png_rust_default/state.png',png);
console.log('done height='+h);
await browser.close();
