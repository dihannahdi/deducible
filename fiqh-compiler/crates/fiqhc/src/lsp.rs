//! A minimal Language Server (LSP) for `.fiqh` over stdio — Open Core pillar #1.
//!
//! On open/change/save it lexes, parses, and runs the fiqh invariant engine, then publishes
//! diagnostics with PRECISE ranges, the error code (e.g. `RIBA-1`), and the cited daleel — the
//! red squiggle a bank's engineer sees in the editor before ever touching a terminal.
//! Dependency-light: std + serde_json + the fiqhc crate itself (no async runtime, no tower-lsp).

use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};

pub fn run() -> ! {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut docs: HashMap<String, String> = HashMap::new();
    loop {
        // --- read one Content-Length-framed JSON-RPC message ---
        let mut content_length: usize = 0;
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => std::process::exit(0),
                Ok(_) => {}
                Err(_) => std::process::exit(0),
            }
            let t = line.trim_end_matches(|c| c == '\r' || c == '\n');
            if t.is_empty() {
                break;
            }
            if let Some(v) = t.strip_prefix("Content-Length:") {
                content_length = v.trim().parse().unwrap_or(0);
            }
        }
        if content_length == 0 {
            continue;
        }
        let mut buf = vec![0u8; content_length];
        if reader.read_exact(&mut buf).is_err() {
            std::process::exit(0);
        }
        let msg: Value = match serde_json::from_slice(&buf) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let method = msg["method"].as_str().unwrap_or("");
        let id = msg.get("id").cloned();
        match method {
            "initialize" => respond(
                id,
                json!({
                    "capabilities": { "textDocumentSync": 1 },
                    "serverInfo": { "name": "fiqhc-lsp", "version": env!("CARGO_PKG_VERSION") }
                }),
            ),
            "initialized" => {}
            "textDocument/didOpen" => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                let text = msg["params"]["textDocument"]["text"].as_str().unwrap_or("").to_string();
                publish(&uri, &text);
                docs.insert(uri, text);
            }
            "textDocument/didChange" => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                let text = msg["params"]["contentChanges"]
                    .as_array()
                    .and_then(|a| a.last())
                    .and_then(|c| c["text"].as_str())
                    .unwrap_or("")
                    .to_string();
                publish(&uri, &text);
                docs.insert(uri, text);
            }
            "textDocument/didSave" => {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                if let Some(text) = docs.get(&uri) {
                    let t = text.clone();
                    publish(&uri, &t);
                }
            }
            "shutdown" => respond(id, Value::Null),
            "exit" => std::process::exit(0),
            _ => {
                if id.is_some() {
                    respond(id, Value::Null);
                }
            }
        }
    }
}

fn send(v: &Value) {
    let s = serde_json::to_string(v).unwrap_or_default();
    let out = io::stdout();
    let mut o = out.lock();
    let _ = write!(o, "Content-Length: {}\r\n\r\n{}", s.len(), s);
    let _ = o.flush();
}

fn respond(id: Option<Value>, result: Value) {
    if let Some(id) = id {
        send(&json!({ "jsonrpc": "2.0", "id": id, "result": result }));
    }
}

fn publish(uri: &str, text: &str) {
    send(&json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": { "uri": uri, "diagnostics": diagnostics(text) }
    }));
}

fn line_len(text: &str, line0: usize) -> usize {
    text.lines().nth(line0).map(|l| l.chars().count()).unwrap_or(0)
}

fn lsp_diag(text: &str, line: usize, col: usize, severity: u8, code: &str, message: &str) -> Value {
    let line0 = line.saturating_sub(1);
    let start = col.saturating_sub(1);
    let ll = line_len(text, line0);
    let end = if ll > start { ll } else { start + 1 };
    json!({
        "range": {
            "start": { "line": line0, "character": start },
            "end": { "line": line0, "character": end }
        },
        "severity": severity,
        "code": code,
        "source": "fiqhc",
        "message": message
    })
}

fn diagnostics(text: &str) -> Vec<Value> {
    match crate::compile_check(text) {
        Ok((_spec, diags)) => diags
            .iter()
            .map(|d| {
                let sev = if d.is_error() { 1 } else { 2 };
                let mut m = format!("[{}] {}", d.code, d.message);
                if !d.citation.is_empty() {
                    m.push_str(&format!("\n  daleel: {}", d.citation));
                }
                lsp_diag(text, d.span.line, d.span.col, sev, &d.code, &m)
            })
            .collect(),
        Err((msg, span)) => {
            vec![lsp_diag(text, span.line, span.col, 1, "PARSE", &format!("parse error: {}", msg))]
        }
    }
}
