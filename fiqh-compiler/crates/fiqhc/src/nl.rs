//! Experimental natural-language front-end (non-load-bearing).
//!
//! `fiqhc nl <text>` asks an LLM (DeepSeek, OpenAI-compatible) to DRAFT a `.fiqh`
//! specification from a natural-language description. The draft is then fed back
//! through the SAME formal compiler and fiqh invariant engine — the LLM never
//! relaxes the shariah; it only proposes syntax at the input edge, and a draft
//! that contradicts the rule-base is refused exactly like a hand-written one.
//!
//! Transport is `curl` (already present), so we avoid pulling an async HTTP stack;
//! only `serde_json` is added, for robust request/response handling.

use std::io::Write;
use std::process::{Command, Stdio};

const GRAMMAR: &str = r#"You translate a natural-language description of an Islamic-finance contract into the
`.fiqh` domain-specific language. Output ONLY the `.fiqh` specification — no markdown
fences, no commentary, no explanation.

GRAMMAR (block-structured):
  instrument <Name> : <class> { <sections> }
  class is one of: musharakah_mutanaqisah | mudarabah | ijarah_imbt
  sections:
    meta { key: value; ... }                         // e.g. basis: "AAOIFI SS No. 13"; currency: tinybar;
    parties { name : role flags; ... }               // flags e.g. `independent`
    capital { party : N bps; ... require <expr>; }
    returns { mechanism { key: value; ... } ... }     // mechanisms: rent | profit | buyout
    risk { loss: <value>; capital_guarantee: <value>; }
    invariant <name> { <expr> }                       // one block per invariant
    rescission { type { key: value; } ... }
    lifecycle { step; step(arg); ... }
  values: "string" | N unit | dotted.path | a == b | a != b | a + b | a -> b | bare_identifier

HARD SYNTAX RULES (a violation makes the compiler refuse the draft):
  - Invariant bodies are SIMPLE comparisons only. Allowed: `a == b`, `a != b`, `a + b == c`.
  - NEVER invent literals like `(6000 : 4000)`, tuples, ratios with colons, or `undefined`.
  - Copy the invariant bodies for the chosen class EXACTLY as written below (substitute party names only).
  - Every statement ends with `;` except `invariant NAME { ... }` blocks.

REQUIRED shape per class (emit ALL sections and ALL invariants verbatim, or the draft is refused):

musharakah_mutanaqisah:
  parties: financier, acquirer, oracle (independent), adjudicator, beneficiary
  capital: the two partners sum to 10000 bps
  returns: rent { basis: <financier>.share; rate: N per_bps_period; }
           buyout { price: oracle.fairValue * bps; transfers: <financier>.share -> <acquirer>.share; }
  risk: loss: proportional_to_ownership; capital_guarantee: none;
  invariant ownership_conserved  { <financier>.share + <acquirer>.share == 10000 }
  invariant rent_on_living_share { rent.basis == <financier>.share }
  invariant loss_follows_capital { loss == proportional_to_ownership }
  invariant price_attested       { buyout.price == oracle.fairValue }
  lifecycle: fund; payRent; buyShare(bps); settle;

mudarabah:
  parties: rabb_al_mal, mudarib, oracle (independent), adjudicator
  capital: <rabb> 10000 bps; <mudarib> 0 bps;
  returns: profit { source: oracle.realizedProfit; split: ratio; <rabb>: N bps; <mudarib>: M bps; }
  risk: loss: on_rabb_al_mal; capital_guarantee: none;
  invariant capital_from_rabb_al_mal_only { <mudarib> == 0 }
  invariant profit_by_ratio               { profit.split == ratio }
  invariant loss_on_rabb_al_mal           { loss == on_rabb_al_mal }
  invariant no_guaranteed_profit          { capital_guarantee == none }
  lifecycle: fund; reportReturn; settle;

ijarah_imbt:
  parties: lessor, lessee, oracle (independent), adjudicator
  capital: <lessor> 10000 bps;
  returns: rent { basis: usufruct; rate: N per_period; term: K; late_penalty: none; }
  risk: loss: on_lessor;
  invariant rent_for_usufruct            { rent.basis == usufruct }
  invariant lessor_bears_ownership_risk  { loss == on_lessor }
  invariant transfer_separate_from_lease { transferOwnership != payRent }
  invariant no_late_penalty_interest     { rent.late_penalty == none }
  lifecycle: activate; payRent; transferOwnership;

Choose the class that fits the description. Keep citations honest; if unsure, write a basis
string and let the human verify it. Output only the spec, nothing else."#;

/// Strip whitespace and a single layer of surrounding quotes, as `.env` values often
/// arrive quoted (e.g. DEEPSEEK_BASE_URL="https://api.deepseek.com").
fn clean(s: String) -> String {
    let t = s.trim();
    let t = t.strip_prefix('"').unwrap_or(t);
    let t = t.strip_suffix('"').unwrap_or(t);
    let t = t.strip_prefix('\'').unwrap_or(t);
    let t = t.strip_suffix('\'').unwrap_or(t);
    t.trim().to_string()
}

pub fn draft(nl_text: &str) -> Result<String, String> {
    let key = clean(
        std::env::var("DEEPSEEK_API_KEY")
            .map_err(|_| "DEEPSEEK_API_KEY not set (run: set -a; . /workspace/.env; set +a)".to_string())?,
    );
    let base = clean(std::env::var("DEEPSEEK_BASE_URL").unwrap_or_else(|_| "https://api.deepseek.com".to_string()));
    let model = clean(std::env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string()));
    let url = format!("{}/chat/completions", base.trim_end_matches('/'));

    let example = include_str!("../../../specs/musharakah_mutanaqisah.fiqh");
    let system = format!("{}\n\nA COMPLETE EXAMPLE (musharakah):\n{}", GRAMMAR, example);

    let body = serde_json::json!({
        "model": model,
        "temperature": 0,
        "stream": false,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": nl_text}
        ]
    });
    let body_str = serde_json::to_string(&body).map_err(|e| e.to_string())?;

    let mut child = Command::new("curl")
        .args([
            "-sS", "--max-time", "120", "-X", "POST", &url,
            "-H", &format!("Authorization: Bearer {}", key),
            "-H", "Content-Type: application/json",
            "--data-binary", "@-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn curl: {}", e))?;
    {
        let mut si = child.stdin.take().ok_or("no stdin")?;
        si.write_all(body_str.as_bytes()).map_err(|e| e.to_string())?;
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("curl failed: {}", String::from_utf8_lossy(&out.stderr)));
    }
    let resp = String::from_utf8_lossy(&out.stdout).to_string();
    let v: serde_json::Value = serde_json::from_str(&resp)
        .map_err(|e| format!("bad JSON from API: {} -- raw: {}", e, truncate(&resp, 400)))?;
    let content = v["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| format!("no completion content -- raw: {}", truncate(&resp, 400)))?;

    Ok(strip_fences(content))
}

fn strip_fences(s: &str) -> String {
    let lines: Vec<&str> = s.lines().filter(|l| !l.trim_start().starts_with("```")).collect();
    lines.join("\n").trim().to_string() + "\n"
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        format!("{}…", &s[..n])
    }
}
