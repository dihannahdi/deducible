//! `fiqhc` command-line interface.

use fiqhc::sema::{Diagnostic, Severity};
use std::process::exit;

fn usage() -> ! {
    eprintln!(
        "fiqhc — a compliance-by-construction compiler for Islamic-finance contracts\n\
         \n\
         usage:\n\
         \x20 fiqhc parse <file.fiqh>          parse and dump the AST\n\
         \x20 fiqhc check <file.fiqh>          run the fiqh invariant engine (no codegen)\n\
         \x20 fiqhc build <file.fiqh> [opts]   check, then emit Solidity + test + deploy descriptor\n\
         \x20 fiqhc nl    <file.txt>           draft a .fiqh spec from natural language (experimental)\n"
    );
    exit(2);
}

fn read(path: &str) -> String {
    match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("fiqhc: cannot read {}: {}", path, e);
            exit(2);
        }
    }
}

/// Print diagnostics in a compiler-like form; return the number of errors.
fn report(path: &str, diags: &[Diagnostic]) -> usize {
    let mut errors = 0;
    for d in diags {
        let label = match d.severity {
            Severity::Error => {
                errors += 1;
                "error"
            }
            Severity::Warning => "warning",
        };
        eprintln!("{}:{}:{}: {}[{}]: {}", path, d.span.line, d.span.col, label, d.code, d.message);
        if !d.citation.is_empty() {
            eprintln!("      fiqh: {}", d.citation);
        }
    }
    errors
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("");
    match cmd {
        "parse" => {
            let path = args.get(2).unwrap_or_else(|| usage());
            let src = read(path);
            match fiqhc::compile_parse(&src) {
                Ok(spec) => println!("{:#?}", spec),
                Err((msg, span)) => {
                    eprintln!("{}:{}:{}: parse error: {}", path, span.line, span.col, msg);
                    exit(1);
                }
            }
        }
        "check" => {
            let path = args.get(2).unwrap_or_else(|| usage());
            let src = read(path);
            match fiqhc::compile_check(&src) {
                Ok((spec, diags)) => {
                    let errors = report(path, &diags);
                    if errors > 0 {
                        eprintln!(
                            "\nrefused: '{}' is INCONSISTENT with its declared fiqh rule-base ({} error(s)). No contract emitted.",
                            spec.name, errors
                        );
                        exit(1);
                    } else {
                        println!(
                            "consistent: '{}' ({}) is consistent with its declared rule-base. (Consistency is not a fatwa — a qualified scholar must ratify the rule-base. Allahu a'lam.)",
                            spec.name, spec.class
                        );
                    }
                }
                Err((msg, span)) => {
                    eprintln!("{}:{}:{}: parse error: {}", path, span.line, span.col, msg);
                    exit(1);
                }
            }
        }
        "build" => {
            let mut spec_path: Option<String> = None;
            let mut root = String::from("..");
            let mut target = String::from("all"); // solidity | manifest | all
            let mut it = args.iter().skip(2);
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--root" => root = it.next().cloned().unwrap_or_else(|| usage()),
                    "--target" => target = it.next().cloned().unwrap_or_else(|| usage()),
                    _ => {
                        if spec_path.is_none() {
                            spec_path = Some(a.clone());
                        }
                    }
                }
            }
            let path = spec_path.unwrap_or_else(|| usage());
            let src = read(&path);
            let (spec, diags) = match fiqhc::compile_check(&src) {
                Ok(x) => x,
                Err((msg, span)) => {
                    eprintln!("{}:{}:{}: parse error: {}", path, span.line, span.col, msg);
                    exit(1);
                }
            };
            let errors = report(&path, &diags);
            if errors > 0 {
                eprintln!(
                    "\nrefused: '{}' is INCONSISTENT with its declared fiqh rule-base ({} error(s)). No contract emitted.",
                    spec.name, errors
                );
                exit(1);
            }
            let g = match fiqhc::codegen::generate(&spec) {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("codegen error: {}", e);
                    exit(1);
                }
            };
            let want_sol = target == "solidity" || target == "all";
            let want_manifest = target == "manifest" || target == "all";
            if !want_sol && !want_manifest {
                eprintln!("fiqhc: unknown --target '{}' (use solidity | manifest | all)", target);
                exit(2);
            }
            let mut emitted: Vec<String> = Vec::new();
            if want_sol {
                emitted.push(write_out(&root, &format!("contracts/generated/{}.sol", g.contract_name), &g.sol));
                emitted.push(write_out(&root, &format!("test/generated/{}.test.js", g.contract_name), &g.test_js));
                emitted.push(write_out(&root, &format!("fiqh-compiler/out/{}.deploy.json", g.contract_name), &g.descriptor));
            }
            if want_manifest {
                let manifest = fiqhc::codegen::build_manifest(&spec);
                emitted.push(write_out(&root, &format!("fiqh-compiler/out/{}.manifest.json", g.contract_name), &manifest));
            }
            println!(
                "emitted from '{}' ({}) — consistent-by-construction:\n    {}",
                spec.name, g.instrument, emitted.join("\n    ")
            );
        }
        "nl" => {
            let path = args.get(2).unwrap_or_else(|| usage());
            let src = read(path);
            match fiqhc::nl::draft(&src) {
                Ok(draft) => {
                    // stdout carries ONLY the drafted .fiqh (so it can be redirected to a file);
                    // the formal-gate verdict goes to stderr.
                    print!("{}", draft);
                    eprintln!("\n--- formal gate (the LLM draft must pass the same compiler) ---");
                    match fiqhc::compile_check(&draft) {
                        Ok((spec, diags)) => {
                            let errors = report("<nl-draft>", &diags);
                            if errors > 0 {
                                eprintln!("REFUSED: the NL draft of '{}' is inconsistent with the rule-base ({} error(s)).", spec.name, errors);
                                exit(1);
                            } else {
                                eprintln!("PASSED: the NL draft of '{}' ({}) is consistent and will lower to a contract.", spec.name, spec.class);
                            }
                        }
                        Err((msg, span)) => {
                            eprintln!("<nl-draft>:{}:{}: parse error: {}", span.line, span.col, msg);
                            exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("fiqhc nl: {}", e);
                    exit(1);
                }
            }
        }
        _ => usage(),
    }
}

fn write_out(root: &str, rel: &str, content: &str) -> String {
    let p = format!("{}/{}", root, rel);
    if let Some(parent) = std::path::Path::new(&p).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(&p, content) {
        eprintln!("fiqhc: cannot write {}: {}", p, e);
        exit(1);
    }
    p
}
