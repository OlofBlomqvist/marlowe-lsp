mod common;


use std::collections::{HashMap, VecDeque};

use crate::common::*;

use lsp_types::{request::Request, LogMessageParams};
use web_sys::{DedicatedWorkerGlobalScope, console};
use wasm_bindgen::{JsCast, prelude::wasm_bindgen, JsValue};
use console_error_panic_hook;
#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

use js_sys::JSON;
#[wasm_bindgen]
pub fn mainly() {

    let mut server = MarloweLSPServer { 
        state: State {
            files: codespan::Files::new(),
            sources: HashMap::new(),
            marlowe_asts: HashMap::new(),
            marlowe_parser_error: None,
            outgoing_diagnostics: VecDeque::new(),
            outgoing_log_messages: VecDeque::new()
            ,
            sexpression_parser_error: None,
            sexpression_asts: HashMap::new()
        } 
    };

    let closure = wasm_bindgen::prelude::Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
        
        web_sys::console::log_1(&"AAA".into());
        let lsp_msg = event.data();
        
        let xx = JSON::stringify(&lsp_msg).unwrap().as_string().unwrap();
        web_sys::console::log_1(&format!("HI FROM RUST WORKER, I GOT YOUR MESSAGE: {xx}").into());
        web_sys::console::log_1(&"BBB".into());
   
        let result = server.run_with_json(&xx);

        if let Ok(Some(response)) = result {
            web_sys::console::log_1(&format!("ok we got this and will send it to js : {:?}",response).into());
            let js_value = JsValue::from_serde(&response).unwrap();
            send_msg(&js_value);
        } else {
            web_sys::console::log_1(&format!("meh : {:?}",result).into());
        }

        while let Some(x) = server.state.outgoing_log_messages.pop_front() {
            let msg = LogMessageParams { typ: x.0, message: x.1 };
            let jj = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "window/logMessage",
                "params": msg
            });
            let js_value = JsValue::from_serde(&jj).unwrap();
           send_msg(&js_value);
        }

        while let Some(x) = server.state.outgoing_diagnostics.pop_front() {
            let jj = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "textDocument/publishDiagnostics",
                "params": x
            });
            web_sys::console::log_1(&format!("published diagnostics for {}: {:?}",x.uri,x.diagnostics).into());
            let js_value = JsValue::from_serde(&jj).unwrap();
           send_msg(&js_value);
        }

        web_sys::console::log_1(&format!("Ok ive processed all outgoing log messages successfully.").into());
    

    }) as Box<dyn FnMut(_)>);

    let worker: DedicatedWorkerGlobalScope = js_sys::global().dyn_into().unwrap();
    worker.set_onmessage(Some(closure.as_ref().unchecked_ref()));

    // Remember to not drop the closure
    closure.forget();

    worker.post_message(&"ready!".into()).unwrap();
}


fn send_msg(value:&wasm_bindgen::JsValue) {
    let worker: DedicatedWorkerGlobalScope = js_sys::global().dyn_into().unwrap();        
    worker.post_message(value).unwrap();
}