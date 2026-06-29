//! `deducible` command-line interface.

use fiqhc::sema::{Diagnostic, Severity};
use std::process::exit;

fn usage() -> ! {
    eprintln!(
        "deducible — a compliance-by-construction compiler for Islamic-finance contracts\n\
         \n\
         usage:\n\
         \x20 deducible parse <file.fiqh>          parse and dump the AST\n\
         \x20 deducible check <file.fiqh>          run the fiqh invariant engine (no codegen)\n\
         \x20 deducible build <file.fiqh> [opts]   check, then emit code (--target solidity|manifest|zk|all)\n\
         \x20 deducible nl    <file.txt>           draft a .fiqh spec from natural language (experimental)\n\
         \x20 deducible lsp                        run the Language Server (stdio JSON-RPC, for editors)\n\
         \x20 deducible fuzz [N]                   fuzz the front-end + engine for N iterations (default 100000)\n"
    );
    exit(2);
}

fn read(path: &str) -> String {
    match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("deducible: cannot read {}: {}", path, e);
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
            match fiqhc::compile_parse_unit(&src) {
                Ok(fiqhc::parser::Unit::Instrument(spec)) => println!("{:#?}", spec),
                Ok(fiqhc::parser::Unit::Bundle(b)) => println!("{:#?}", b),
                Err((msg, span)) => {
                    eprintln!("{}:{}:{}: parse error: {}", path, span.line, span.col, msg);
                    exit(1);
                }
            }
        }
        "check" => {
            let mut spec_path: Option<String> = None;
            let mut rules: Option<String> = None;
            let mut it = args.iter().skip(2);
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--rules" => rules = it.next().cloned(),
                    _ => {
                        if spec_path.is_none() {
                            spec_path = Some(a.clone());
                        }
                    }
                }
            }
            let path = spec_path.unwrap_or_else(|| usage());
            let src = read(&path);
            // Composite bundle? Route to the graph-based invariant checker (al-'uqud al-murakkabah).
            if matches!(fiqhc::compile_parse_unit(&src), Ok(fiqhc::parser::Unit::Bundle(_))) {
                match fiqhc::compile_check_bundle(&src) {
                    Ok((bundle, diags)) => {
                        let errors = report(&path, &diags);
                        if errors > 0 {
                            eprintln!(
                                "\nrefused: composite '{}' encodes a riba structure by COMPOSITION ({} error(s)). No package emitted.",
                                bundle.name, errors
                            );
                            exit(1);
                        }
                        println!(
                            "consistent: composite '{}' is free of bay' al-'inah / organized-tawarruq cycles by construction. (Consistency is not a fatwa. Allahu a'lam.)",
                            bundle.name
                        );
                        return;
                    }
                    Err((msg, span)) => {
                        eprintln!("{}:{}:{}: parse error: {}", path, span.line, span.col, msg);
                        exit(1);
                    }
                }
            }
            let spec = match fiqhc::compile_parse(&src) {
                Ok(s) => s,
                Err((msg, span)) => {
                    eprintln!("{}:{}:{}: parse error: {}", path, span.line, span.col, msg);
                    exit(1);
                }
            };
            // Effective rule-base: --rules flag, else `meta { rules: "..."; }`, else the builtin engine.
            let eff = rules.or_else(|| {
                spec.meta()
                    .into_iter()
                    .find(|k| k.key == "rules")
                    .and_then(|k| k.val.as_str().map(|s| s.to_string()).or_else(|| k.val.as_ident().map(|s| s.to_string())))
            });
            let (diags, authority) = match eff {
                Some(name) => match load_ruleset(&name) {
                    Ok(rs) => {
                        let label = rs.label();
                        (fiqhc::sema::check_with_ruleset(&spec, &rs), Some(label))
                    }
                    Err(e) => {
                        eprintln!("deducible: {}", e);
                        exit(2);
                    }
                },
                None => (fiqhc::sema::check(&spec), None),
            };
            let errors = report(&path, &diags);
            let base = authority.as_deref().unwrap_or("builtin engine");
            if errors > 0 {
                eprintln!(
                    "\nrefused: '{}' is INCONSISTENT with the {} rule-base ({} error(s)). No contract emitted.",
                    spec.name, base, errors
                );
                exit(1);
            } else {
                println!(
                    "consistent: '{}' ({}) is consistent with the {} rule-base. (Consistency is not a fatwa — the authority must ratify the rule module. Allahu a'lam.)",
                    spec.name, spec.class, base
                );
            }
        }
        "build" => {
            let mut spec_path: Option<String> = None;
            let mut root = String::from("..");
            let mut target = String::from("all"); // solidity | manifest | all
            let mut rules: Option<String> = None; // optional governing authority / madhhab module
            let mut it = args.iter().skip(2);
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--root" => root = it.next().cloned().unwrap_or_else(|| usage()),
                    "--target" => target = it.next().cloned().unwrap_or_else(|| usage()),
                    "--rules" => rules = it.next().cloned(),
                    _ => {
                        if spec_path.is_none() {
                            spec_path = Some(a.clone());
                        }
                    }
                }
            }
            let path = spec_path.unwrap_or_else(|| usage());
            let src = read(&path);
            // Composite bundle? Gate on the flow graph, then emit a composite invariant manifest.
            if matches!(fiqhc::compile_parse_unit(&src), Ok(fiqhc::parser::Unit::Bundle(_))) {
                match fiqhc::compile_check_bundle(&src) {
                    Ok((bundle, diags)) => {
                        let errors = report(&path, &diags);
                        if errors > 0 {
                            eprintln!(
                                "\nrefused: composite '{}' encodes a riba structure by COMPOSITION ({} error(s)). No package emitted.",
                                bundle.name, errors
                            );
                            exit(1);
                        }
                        let manifest = fiqhc::composite::build_manifest(&bundle);
                        let out = write_out(&root, &format!("fiqh-compiler/out/{}.composite.json", bundle.name), &manifest);
                        println!(
                            "emitted composite invariant manifest from '{}' — free of riba cycles by construction:\n    {}",
                            bundle.name, out
                        );
                        return;
                    }
                    Err((msg, span)) => {
                        eprintln!("{}:{}:{}: parse error: {}", path, span.line, span.col, msg);
                        exit(1);
                    }
                }
            }
            let spec = match fiqhc::compile_parse(&src) {
                Ok(s) => s,
                Err((msg, span)) => {
                    eprintln!("{}:{}:{}: parse error: {}", path, span.line, span.col, msg);
                    exit(1);
                }
            };
            // The builtin engine always runs (the universal fiqh core + maqasid warnings). If an
            // authority/madhhab module is chosen (--rules or meta { rules }), its constraints are
            // LAYERED on top, so the chosen authority governs whether the contract is GENERATED.
            let mut diags = fiqhc::sema::check(&spec);
            let eff = rules.or_else(|| {
                spec.meta()
                    .into_iter()
                    .find(|k| k.key == "rules")
                    .and_then(|k| k.val.as_str().map(|s| s.to_string()).or_else(|| k.val.as_ident().map(|s| s.to_string())))
            });
            let authority = match eff {
                Some(name) => match load_ruleset(&name) {
                    Ok(rs) => {
                        let label = rs.label();
                        diags.extend(fiqhc::sema::check_with_ruleset(&spec, &rs));
                        Some(label)
                    }
                    Err(e) => {
                        eprintln!("deducible: {}", e);
                        exit(2);
                    }
                },
                None => None,
            };
            let errors = report(&path, &diags);
            if errors > 0 {
                let base = authority.as_deref().unwrap_or("its declared fiqh");
                eprintln!(
                    "\nrefused: '{}' is INCONSISTENT with the {} rule-base ({} error(s)). No contract emitted.",
                    spec.name, base, errors
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
            let want_zk = target == "zk" || target == "all";
            if !want_sol && !want_manifest && !want_zk {
                eprintln!("deducible: unknown --target '{}' (use solidity | manifest | zk | all)", target);
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
            if want_zk {
                let zk = fiqhc::codegen::build_zk(&spec);
                emitted.push(write_out(&root, &format!("fiqh-compiler/out/{}.circom", zk.circuit_name), &zk.circom));
                emitted.push(write_out(&root, &format!("fiqh-compiler/out/{}.zk.json", zk.circuit_name), &zk.manifest));
                if !zk.verifier_consumer.is_empty() {
                    emitted.push(write_out(&root, &format!("contracts/generated/{}ZkGate.sol", spec.name), &zk.verifier_consumer));
                }
            }
            let gov = authority.as_deref().map(|a| format!(" [governed by {}]", a)).unwrap_or_default();
            println!(
                "emitted from '{}' ({}){} — consistent-by-construction:\n    {}",
                spec.name, g.instrument, gov, emitted.join("\n    ")
            );
        }
        "lsp" => fiqhc::lsp::run(),
        "fuzz" => {
            let n = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(100_000);
            let seeds = [
                include_str!("../../../specs/musharakah_mutanaqisah.fiqh"),
                include_str!("../../../specs/commercial_escrow.fiqh"),
                include_str!("../../../specs/riba_disguised.fiqh"),
            ];
            match fiqhc::fuzz::run(n, &seeds) {
                None => println!("fuzz: {} iterations — no panic, no crash. The engine holds.", n),
                Some(input) => {
                    eprintln!("fuzz: PANIC on input ({} bytes):\n{}", input.len(), input);
                    exit(1);
                }
            }
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
                    eprintln!("deducible nl: {}", e);
                    exit(1);
                }
            }
        }
        _ => usage(),
    }
}

/// Load a pluggable rule-base module by name from the rules directory
/// (`$FIQHC_RULES_DIR` or `./rules`), e.g. "aaoifi" -> rules/aaoifi.rules.json.
fn load_ruleset(name: &str) -> Result<fiqhc::sema::RuleSet, String> {
    let dir = std::env::var("FIQHC_RULES_DIR").unwrap_or_else(|_| "rules".to_string());
    let path = format!("{}/{}.rules.json", dir, name);
    let s = std::fs::read_to_string(&path).map_err(|e| format!("cannot read rule module '{}': {}", path, e))?;
    fiqhc::sema::RuleSet::from_json(&s)
}

fn write_out(root: &str, rel: &str, content: &str) -> String {
    let p = format!("{}/{}", root, rel);
    if let Some(parent) = std::path::Path::new(&p).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(&p, content) {
        eprintln!("deducible: cannot write {}: {}", p, e);
        exit(1);
    }
    p
}
