//! A dependency-free fuzz / property harness over the whole front-end + engine (Open Core
//! pillar #4). Random byte soup, random token soup, and structured mutations of valid specs are
//! fed through `compile_check` inside `catch_unwind`: the engine must NEVER panic or crash — it
//! may only return `Ok(diagnostics)` or `Err(parse error)`. A compiler that dictates the movement
//! of real value must not be brought down by malformed input.

use std::panic;

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }
    fn next(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() % (n as u64)) as usize
        }
    }
}

const TOKENS: &[&str] = &[
    "instrument", "X", ":", "musharakah_mutanaqisah", "mudarabah", "ijarah_imbt",
    "commercial_escrow", "{", "}", "(", ")", ";", "meta", "parties", "capital", "returns",
    "risk", "oracle", "dispute", "invariant", "rescission", "lifecycle", "rent", "buyout",
    "profit", "release", "basis", "principal", "bank.share", "==", "!=", "->", "bps", "8000",
    "0", "10000", "none", "proportional_to_ownership", "oracle.fairValue", "financier",
    "acquirer", "bank", "client", "valuer", "\"s\"", "// c\n", "\n", " ", "\t",
];

fn gen_random(rng: &mut Rng) -> String {
    if rng.next() & 1 == 0 {
        let n = rng.below(220);
        let mut s = String::new();
        for _ in 0..n {
            s.push((33 + rng.below(94)) as u8 as char);
        }
        s
    } else {
        let n = rng.below(70);
        let mut s = String::new();
        for _ in 0..n {
            s.push_str(TOKENS[rng.below(TOKENS.len())]);
            s.push(' ');
        }
        s
    }
}

fn mutate(seed: &str, rng: &mut Rng) -> String {
    let mut b: Vec<u8> = seed.bytes().collect();
    if b.is_empty() {
        return String::new();
    }
    let ops = 1 + rng.below(10);
    for _ in 0..ops {
        if b.is_empty() {
            break;
        }
        match rng.below(4) {
            0 => {
                let i = rng.below(b.len());
                b[i] = (33 + rng.below(94)) as u8;
            }
            1 => {
                let i = rng.below(b.len());
                b.truncate(i);
            }
            2 => {
                let i = rng.below(b.len() + 1);
                b.insert(i.min(b.len()), (33 + rng.below(94)) as u8);
            }
            _ => {
                let i = rng.below(b.len());
                b.remove(i);
            }
        }
    }
    String::from_utf8_lossy(&b).to_string()
}

/// Run `iterations` fuzz cases. Returns `Some(input)` on the first input that PANICS, else `None`.
pub fn run(iterations: usize, seeds: &[&str]) -> Option<String> {
    let prev = panic::take_hook();
    panic::set_hook(Box::new(|_| {})); // keep the fuzz quiet
    let mut rng = Rng::new(0x00C0_FFEE ^ iterations as u64);
    let mut found = None;
    for _ in 0..iterations {
        let input = if seeds.is_empty() || rng.next() & 1 == 0 {
            gen_random(&mut rng)
        } else {
            mutate(seeds[rng.below(seeds.len())], &mut rng)
        };
        let inp = input.clone();
        let r = panic::catch_unwind(|| {
            let _ = crate::compile_check(&inp);
        });
        if r.is_err() {
            found = Some(input);
            break;
        }
    }
    panic::set_hook(prev);
    found
}
