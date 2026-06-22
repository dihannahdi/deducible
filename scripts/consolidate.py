import os
here = "/workspace/paper"
src = open(os.path.join(here, "full_paper_v2.md")).read()

abs_add = (" In an extended artifact family we further represent ownership as a real Hedera "
           "Token Service token whose units move atomically with payment, and we encode the "
           "*fiqh* rescission rights (*khiyar al-shart* and *iqalah*) the sentence above notes "
           "as open — leaving their juristic adequacy to qualified scholars.")

new_sections = """
### 4.4 Tokenized ownership and HTS-native atomic buyout

Ownership is represented as a real Hedera Token Service (HTS) fungible token (10,000 finite
units = 100%); on testnet the financier and client held 8,000 and 2,000 units respectively,
i.e. genuine, transferable fractional ownership with on-chain provenance rather than a bare
counter. A custodian contract (testnet `0.0.9304674`) demonstrated that the enforcement
contract can itself hold and move these units via the HTS system contract (precompile
`0x167`). Building on this, an HTS-native partnership contract (V3, testnet `0.0.9304707`)
executed an **atomic buyout**: in a single transaction, 2,000 ownership units moved from the
contract's escrow to the client (escrow 8,000 -> 6,000) and the hbar price moved to the
financier. Ownership transfer and payment are thereby inseparable -- there is no off-ledger
interval between them. (The HTS precompile executes only on Hedera, so this layer is verified
on testnet rather than on a local EVM.)

### 4.5 Lawful rescission (khiyar and iqalah)

To reconcile the finality of on-chain execution with the flexibility classical *fiqh*
preserves, a rescission-enabled contract (V4) encodes two rights: *khiyar al-shart*, a
stipulated window after activation within which either partner may unilaterally rescind, and
*iqalah*, mutual cancellation that completes only with the other partner's consent. Both
refund each partner's funded capital, and both are barred once performance has begun (a buyout
has occurred). All fifteen unit tests pass, including: rescission within the window refunds
capital; rescission after the deadline reverts; *iqalah* requires the counter-party's
acceptance; and rescission is refused after a buyout. With this, all four fault lines named in
the introduction have a working mechanism: form-over-substance (enforced conditions),
risk-sharing (capital-custody settlement), the *gharar* boundary (an externalised valuer), and
now immutability-versus-flexibility (encoded rescission). Whether a particular encoding
satisfies the *fiqh* is a question for qualified scholars; we claim only that the mechanism is
expressible and enforceable.
"""

lim_add = """**Further limitations from the extended artifact.** The rescission rights currently unwind
only *before* performance; post-buyout unwinding, which must apportion partial ownership and
payments already made, is future work. A single HTS-native path that settles a *loss* by
redistributing both units and value on a downward revaluation is demonstrated in parts
(capital-custody settlement, and atomic buyout) but not yet unified. And whether the
*khiyar*/*iqalah* encodings, the tokenized-ownership model, and the diminishing partnership
itself are Shari'ah-compliant remains a scholarly ruling, not an engineering result.

"""

assert "\n\n**Keywords:**" in src
assert "\n## 5. Discussion" in src
assert "**Practical and policy implications.**" in src

src = src.replace("\n\n**Keywords:**", abs_add + "\n\n**Keywords:**", 1)
src = src.replace("\n## 5. Discussion", new_sections + "\n## 5. Discussion", 1)
src = src.replace("**Practical and policy implications.**", lim_add + "**Practical and policy implications.**", 1)

# refresh the version note line at the top
src = src.replace("> Revised manuscript (v2).", "> Consolidated manuscript (v3): adds tokenized ownership (HTS), HTS-native atomic buyout, and khiyar/iqalah rescission.", 1)

open(os.path.join(here, "full_paper_v3.md"), "w").write(src)
print("wrote full_paper_v3.md, chars:", len(src))
