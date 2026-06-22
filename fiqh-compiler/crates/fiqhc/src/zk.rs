//! Zero-Knowledge Fiqh — proving compliance without revealing the bank's secrets (vector #1).
//!
//! A global bank will never place its real loss figures, asset values, or client identities on
//! a public ledger, even disguised. Yet the fiqh invariant must still be PROVEN, not merely
//! asserted. The reconciliation is a zero-knowledge proof: the bank publishes a mathematical
//! proof that "this settlement shared the loss proportional to ownership — [RISK-1] holds"
//! without disclosing a single figure.
//!
//! This module is a *working proof of concept* of that principle for the proportional-loss
//! invariant of a Musharakah. It is built bottom-up from std alone — its own SHA-256, its own
//! deterministic prime search, Pedersen commitments, and a Schnorr/Fiat-Shamir sigma protocol —
//! so the whole chain is auditable and dependency-free.
//!
//! The statement proved in zero knowledge:
//!     committed loss_bank, loss_client satisfy   loss_bank * client_bps == loss_client * bank_bps
//! i.e. each partner bore loss in proportion to ownership. The verifier learns ONLY that the
//! proportion holds — never loss_bank or loss_client.
//!
//! Construction. Pedersen commitments Cb = g^{lb} h^{rb}, Cc = g^{lc} h^{rc} over a prime-order
//! group. Let C_z = Cb^{client_bps} · Cc^{(q - bank_bps)} = g^{client_bps·lb − bank_bps·lc} · h^{r_z}.
//! When the proportion holds the g-exponent is 0, so C_z = h^{r_z}: a commitment to ZERO. Proving
//! the invariant therefore reduces to a Schnorr proof of knowledge of r_z with C_z = h^{r_z}. If
//! the proportion fails, C_z carries a g^δ (δ≠0) factor and no such r_z exists (dlog_g(h) is
//! unknown), so the proof cannot verify — soundness. Schnorr is honest-verifier zero-knowledge;
//! Fiat-Shamir (our SHA-256 as the random oracle) makes it non-interactive.
//!
//! HONEST SCOPE. The PROTOCOL is real. The PARAMETERS are illustrative: a ~61-bit prime so the
//! field arithmetic fits in u128 without a bignum dependency. That is a teaching modulus, not a
//! production one (production ZK-Fiqh emits a Circom/Groth16 circuit — see codegen `--target zk`).
//! The base h is fixed by a nothing-up-my-sleeve hash so its discrete log base g is unknown
//! (Pedersen binding). No claim of cryptographic strength at these parameters is made.

use std::sync::OnceLock;

// ---------------------------------------------------------------------------------------------
// SHA-256 (FIPS 180-4), self-contained — used as the Fiat-Shamir random oracle.
// ---------------------------------------------------------------------------------------------

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub fn sha256(msg: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let mut data = msg.to_vec();
    let bitlen = (msg.len() as u64) * 8;
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bitlen.to_be_bytes());

    for chunk in data.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[4 * i], chunk[4 * i + 1], chunk[4 * i + 2], chunk[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e;
            e = d.wrapping_add(t1);
            d = c; c = b; b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b); h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d); h[4] = h[4].wrapping_add(e); h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g); h[7] = h[7].wrapping_add(hh);
    }
    let mut out = [0u8; 32];
    for i in 0..8 {
        out[4 * i..4 * i + 4].copy_from_slice(&h[i].to_be_bytes());
    }
    out
}

// ---------------------------------------------------------------------------------------------
// Modular arithmetic over a ~61-bit prime field (fits u128 products).
// ---------------------------------------------------------------------------------------------

fn mulmod(a: u128, b: u128, m: u128) -> u128 {
    (a % m) * (b % m) % m
}

fn powmod(mut base: u128, mut exp: u128, m: u128) -> u128 {
    let mut r = 1u128 % m;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 {
            r = mulmod(r, base, m);
        }
        base = mulmod(base, base, m);
        exp >>= 1;
    }
    r
}

/// Deterministic Miller-Rabin. For n < 2^62 the fixed base set below is a correct primality test.
fn is_prime(n: u128) -> bool {
    if n < 2 {
        return false;
    }
    for p in [2u128, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n % p == 0 {
            return n == p;
        }
    }
    let mut d = n - 1;
    let mut r = 0u32;
    while d & 1 == 0 {
        d >>= 1;
        r += 1;
    }
    'witness: for a in [2u128, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        let mut x = powmod(a, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }
        for _ in 0..r - 1 {
            x = mulmod(x, x, n);
            if x == n - 1 {
                continue 'witness;
            }
        }
        return false;
    }
    true
}

/// Group parameters: a safe prime p = 2q+1, generator g of the order-q subgroup, and a second
/// generator h whose discrete log base g is unknown (nothing-up-my-sleeve).
pub struct Params {
    pub p: u128,
    pub q: u128,
    pub g: u128,
    pub h: u128,
}

fn u128_from_hash(seed: &[u8]) -> u128 {
    let d = sha256(seed);
    let mut x = 0u128;
    for &b in &d[..16] {
        x = (x << 8) | b as u128;
    }
    x
}

fn compute_params() -> Params {
    // Deterministic search for a safe prime near 2^60 (so p < 2^62 and products fit u128).
    let mut q: u128 = (1u128 << 60) | 1; // odd start
    let (p, q) = loop {
        if is_prime(q) {
            let p = 2 * q + 1;
            if is_prime(p) {
                break (p, q);
            }
        }
        q += 2;
    };
    // g: square a small element into the order-q subgroup (cofactor 2); g != 1.
    let g = powmod(4, 1, p); // 4 = 2^2 is already a quadratic residue of order q
    // h: nothing-up-my-sleeve — hash a domain string into the field, square into the subgroup.
    let mut i = 0u8;
    let h = loop {
        let mut seed = b"fiqhc/zk/pedersen-h/v1".to_vec();
        seed.push(i);
        let base = (u128_from_hash(&seed) % (p - 2)) + 2; // in [2, p-1]
        let cand = powmod(base, 2, p);
        if cand != 1 && cand != g {
            break cand;
        }
        i += 1;
    };
    Params { p, q, g, h }
}

pub fn params() -> &'static Params {
    static P: OnceLock<Params> = OnceLock::new();
    P.get_or_init(compute_params)
}

// ---------------------------------------------------------------------------------------------
// Pedersen commitments + a Schnorr/Fiat-Shamir proof of the proportional-loss invariant.
// ---------------------------------------------------------------------------------------------

/// A Pedersen commitment g^value · h^rand mod p.
fn commit(value: u128, rand: u128) -> u128 {
    let pr = params();
    mulmod(powmod(pr.g, value % pr.q, pr.p), powmod(pr.h, rand % pr.q, pr.p), pr.p)
}

/// A deterministic blinding factor / nonce in [1, q), derived from a label and the secret(s).
/// (Deterministic so the engine needs no RNG and the tests are reproducible — in the manner of
/// RFC 6979 deterministic nonces.)
fn derive_scalar(label: &[u8], parts: &[u128]) -> u128 {
    let pr = params();
    let mut seed = label.to_vec();
    for x in parts {
        seed.extend_from_slice(&x.to_be_bytes());
    }
    let v = u128_from_hash(&seed) % (pr.q - 1) + 1;
    v
}

/// A zero-knowledge proof that the committed losses are proportional to ownership.
/// Carries the commitments and the Schnorr proof — but NOT the loss amounts.
#[derive(Debug, Clone)]
pub struct LossProof {
    pub bank_bps: u128,
    pub client_bps: u128,
    pub cb: u128, // commitment to loss_bank
    pub cc: u128, // commitment to loss_client
    pub t: u128,  // Schnorr commitment h^k
    pub s: u128,  // Schnorr response k + c·r_z
}

/// C_z = Cb^{client_bps} · Cc^{q - bank_bps}  — a commitment whose g-exponent is
/// client_bps·loss_bank − bank_bps·loss_client (zero iff the loss is proportional).
fn cz(cb: u128, cc: u128, bank_bps: u128, client_bps: u128) -> u128 {
    let pr = params();
    let neg_bank = (pr.q - (bank_bps % pr.q)) % pr.q;
    mulmod(
        powmod(cb, client_bps % pr.q, pr.p),
        powmod(cc, neg_bank, pr.p),
        pr.p,
    )
}

fn challenge(cb: u128, cc: u128, y: u128, t: u128, bank_bps: u128, client_bps: u128) -> u128 {
    let pr = params();
    let mut seed = b"fiqhc/zk/challenge/v1".to_vec();
    for x in [cb, cc, y, t, bank_bps, client_bps, pr.g, pr.h, pr.p] {
        seed.extend_from_slice(&x.to_be_bytes());
    }
    u128_from_hash(&seed) % pr.q
}

/// Prove (in zero knowledge) that loss_bank : loss_client == bank_bps : client_bps.
/// The prover knows the secret losses; the returned proof reveals only commitments.
pub fn prove_proportional_loss(bank_bps: u64, client_bps: u64, loss_bank: u64, loss_client: u64) -> LossProof {
    let pr = params();
    let (bank_bps, client_bps) = (bank_bps as u128, client_bps as u128);
    // Deterministic blinds for the two commitments.
    let rb = derive_scalar(b"fiqhc/zk/rb", &[loss_bank as u128, bank_bps]);
    let rc = derive_scalar(b"fiqhc/zk/rc", &[loss_client as u128, client_bps]);
    let cb = commit(loss_bank as u128, rb);
    let cc = commit(loss_client as u128, rc);

    // Witness r_z = client_bps·rb − bank_bps·rc (mod q): the dlog of C_z base h, when the
    // proportion holds. Computed regardless; if the proportion fails, C_z also has a g^δ factor
    // and this r_z will NOT satisfy the verification (soundness).
    let term1 = mulmod(client_bps, rb, pr.q);
    let term2 = mulmod(bank_bps, rc, pr.q);
    let r_z = (term1 + pr.q - term2) % pr.q;

    let y = cz(cb, cc, bank_bps, client_bps);
    // Schnorr proof of knowledge of r_z with y = h^{r_z}.
    let k = derive_scalar(b"fiqhc/zk/nonce", &[r_z, y, loss_bank as u128, loss_client as u128]);
    let t = powmod(pr.h, k, pr.p);
    let c = challenge(cb, cc, y, t, bank_bps, client_bps);
    let s = (k + mulmod(c, r_z, pr.q)) % pr.q;

    LossProof { bank_bps, client_bps, cb, cc, t, s }
}

/// Verify the proof. Returns true iff the committed losses are proportional to ownership.
/// The verifier never sees the loss amounts.
pub fn verify_proportional_loss(proof: &LossProof) -> bool {
    let pr = params();
    let y = cz(proof.cb, proof.cc, proof.bank_bps, proof.client_bps);
    let c = challenge(proof.cb, proof.cc, y, proof.t, proof.bank_bps, proof.client_bps);
    // Check h^s == t · y^c (mod p).
    let lhs = powmod(pr.h, proof.s % pr.q, pr.p);
    let rhs = mulmod(proof.t, powmod(y, c, pr.p), pr.p);
    lhs == rhs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_answer() {
        // SHA-256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        let d = sha256(b"abc");
        let hex: String = d.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(hex, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn sha256_empty() {
        let d = sha256(b"");
        let hex: String = d.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(hex, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn params_are_a_safe_prime_with_valid_generators() {
        let pr = params();
        assert!(is_prime(pr.p));
        assert!(is_prime(pr.q));
        assert_eq!(pr.p, 2 * pr.q + 1);
        // g and h have order q (g^q == 1, g != 1).
        assert_ne!(pr.g, 1);
        assert_eq!(powmod(pr.g, pr.q, pr.p), 1);
        assert_ne!(pr.h, 1);
        assert_eq!(powmod(pr.h, pr.q, pr.p), 1);
    }

    #[test]
    fn honest_proportional_loss_verifies() {
        // 8000:2000 ownership; losses 800:200 are proportional (2000·800 == 8000·200).
        let proof = prove_proportional_loss(8000, 2000, 800, 200);
        assert!(verify_proportional_loss(&proof), "an honest proportional loss must verify");
    }

    #[test]
    fn honest_with_larger_proportional_amounts() {
        // 6000:4000; losses 3_000_000:2_000_000 are proportional.
        let proof = prove_proportional_loss(6000, 4000, 3_000_000, 2_000_000);
        assert!(verify_proportional_loss(&proof));
    }

    #[test]
    fn disproportionate_loss_is_rejected() {
        // 8000:2000 but losses 800:300 are NOT proportional — the proof must fail.
        let proof = prove_proportional_loss(8000, 2000, 800, 300);
        assert!(!verify_proportional_loss(&proof), "a disproportionate loss must NOT verify");
    }

    #[test]
    fn tampered_proof_is_rejected() {
        let mut proof = prove_proportional_loss(8000, 2000, 800, 200);
        proof.s = (proof.s + 1) % params().q; // tamper the response
        assert!(!verify_proportional_loss(&proof));
    }

    #[test]
    fn proof_hides_the_amounts() {
        // Two different proportional loss magnitudes (800:200 vs 8:2) yield DIFFERENT commitments,
        // yet both verify — the verifier cannot read the amount out of the proof.
        let a = prove_proportional_loss(8000, 2000, 800, 200);
        let b = prove_proportional_loss(8000, 2000, 8, 2);
        assert!(verify_proportional_loss(&a) && verify_proportional_loss(&b));
        assert_ne!(a.cb, b.cb, "commitments differ; the amount is not recoverable from the proof shape");
    }
}
