import init, * as m from 'marlowe_lsp';

async function initialize(s:string) {
  await init(s+"/dist/0303f73d8038d4ee56bd.wasm");
  m.set_panic_hook();
  m.mainly();
}

self.onmessage = e => {
  if(e.data.startsWith("setPath")) {
    initialize(e.data.split("§")[1]);
  }
};

self.postMessage("getPath");