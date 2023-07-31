#![feature(start)]

mod common;
use common::*;
use lsp_types::LogMessageParams;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::result::Result::Ok;
use std::result::Result::Err;
use std::io::Read;
use std::io::Write;

fn main() {

    let mut server =  MarloweLSPServer { 
        state: State {
            files: codespan::Files::new(),
            sources: HashMap::new(),
            marlowe_asts: HashMap::new(),
            marlowe_parser_error: None,
            outgoing_diagnostics: VecDeque::new(),
            outgoing_log_messages: VecDeque::new(),
            sexpression_parser_error: None,
            sexpression_asts: HashMap::new()
        } 
    };

    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    loop {
        let mut buffer = Vec::new();

        let mut prev_byte = None;
        loop {
            let mut byte = [0; 1];
            match handle.read_exact(&mut byte) {
                Ok(_) => {
                    buffer.push(byte[0]);
                    if byte[0] == b'\n' && prev_byte == Some(b'\r') {
                        if buffer.len() >= 4 && &buffer[(buffer.len()-4)..] == b"\r\n\r\n" {
                            buffer.truncate(buffer.len() - 4);
                            break; 
                        }
                    }
                    prev_byte = Some(byte[0]);
                }
                Err(_) => {
                    return;
                }
            }
        }

        let headers_str = String::from_utf8(buffer).unwrap();
        
        let mut headers = HashMap::new();
        for line in headers_str.lines() {
            let parts: Vec<&str> = line.split(": ").collect();
            if parts.len() == 2 {
                let header_name = parts[0].trim();
                let header_value = parts[1].trim();
                headers.insert(header_name.to_string(), header_value.to_string());
            }
        }
        
        let content_length: usize = headers
            .get("Content-Length")
            .expect("Missing Content-Length")
            .parse()
            .unwrap();

        let mut body = vec![0; content_length];

        handle.read_exact(&mut body).unwrap();
        
        let body_str = String::from_utf8(body).unwrap();
        match server.run_with_json(&body_str) {
            Ok(Some(v)) => send_msg(&v),
            Ok(None) => {},
            Err(_e) => {
                // ??
            },
        }

        
        while let Some(x) = server.state.outgoing_log_messages.pop_front() {
            let msg = LogMessageParams { typ: x.0, message: x.1 };
            let jj = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "window/logMessage",
                "params": msg
            });
            send_msg(&jj);
        }

        while let Some(x) = server.state.outgoing_diagnostics.pop_front() {
            let jj = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "textDocument/publishDiagnostics",
                "params": x
            });
           send_msg(&jj);
        }
        
    }
}

fn send_msg(message: &Value) {
    let message_str = serde_json::to_string(message).unwrap();
    let message_bytes = message_str.as_bytes();
    std::io::stdout().write_all(message_bytes).unwrap();
    std::io::stdout().flush().unwrap();
}
