import puppeteer from 'puppeteer';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { writeFileSync } from 'fs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const MERMAID = join(__dirname, 'node_modules/mermaid/dist/mermaid.min.js');
const ZENUML_JS = join(__dirname, 'node_modules/@mermaid-js/mermaid-zenuml/dist/mermaid-zenuml.js');

const diagramSrc = `zenuml
    title Order Service
    @Actor Client #FFEBE6
    @Boundary OrderController #0747A6
    @EC2 <<BFF>> OrderService #E3FCEF
    group BusinessService {
      @Lambda PurchaseService
      @AzureFunction InvoiceService
    }

    @Starter(Client)
    // POST /orders
    OrderController.post(payload) {
      OrderService.create(payload) {
        order = new Order(payload)
        if(order != null) {
          par {
            PurchaseService.createPO(order)
            InvoiceService.createInvoice(order)      
          }      
        }
      }
    }`;

const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] });
const page = await browser.newPage();
await page.setContent('<!DOCTYPE html><html><body></body></html>');
await page.addScriptTag({ path: MERMAID });
// zenuml.js sets globalThis["mermaid-zenuml"]
await page.addScriptTag({ path: ZENUML_JS });

const result = await page.evaluate(async (text) => {
  const plugin = globalThis['mermaid-zenuml'];
  await window.mermaid.initialize({ startOnLoad: false, theme: 'default', securityLevel: 'loose' });
  if (plugin) {
    await window.mermaid.registerExternalDiagrams([plugin]);
  }
  const c = document.createElement('div'); c.id = 'mc'; document.body.appendChild(c);
  try {
    const r = await window.mermaid.render('zenuml-ref', text, c);
    c.remove();
    return { ok: true, svg: r.svg, hadPlugin: !!plugin };
  } catch(e) {
    c.remove();
    return { ok: false, err: e.message.substring(0,200), hadPlugin: !!plugin };
  }
}, diagramSrc);

await browser.close();

if (result.ok) {
  writeFileSync('./ref/live_editor_zenuml.svg', result.svg);
  console.log('OK (plugin=' + result.hadPlugin + '): ' + result.svg.length + ' bytes written to ref/live_editor_zenuml.svg');
} else {
  console.error('FAILED (plugin=' + result.hadPlugin + '):', result.err);
  process.exit(1);
}
