import init, * as m from 'marlowe_lsp';

async function initialize(s:string) {
  await init(s+"/dist/3e01cd3d41263a324488.wasm");
  m.set_panic_hook();
  m.mainly();
}

self.onmessage = e => {
  if(e.data.startsWith("setPath")) {
    initialize(e.data.split("§")[1]);
  }
};

self.postMessage("getPath");