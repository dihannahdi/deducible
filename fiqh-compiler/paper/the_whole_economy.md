# Compliance-by-Construction for the Whole Muʿāmalāt Economy

*Paper IV. A design-science report on extending the `deducible` compiler from a single instrument to
the breadth of the fiqh of transactions. Siblings: `paper.md` (the instrument), `algorithmic_jurisprudence.md`
(the compiler and the four visions), `beyond_the_bilateral_contract.md` (the five doctrinal vectors).
All scriptural and fiqh citations are flagged `[scholar-verify]`; the engine issues no fatwa. Allāhu aʿlam.*

## Abstract

Earlier work showed that *compliance-by-construction* is possible: a `.fiqh` specification whose declared
economics contradict its declared rule-base cannot be lowered to Solidity (a riba spec will not compile).
That was demonstrated on four contract classes. This paper reports the extension of the method across the
breadth of the muʿāmalāt economy that a knowledge graph of the turath literature (`GAP_ANALYSIS.md`) showed
the compiler did not yet reach: the sale-based instruments that are the market's majority, the full law of
zakat, the social economy, the capital-markets layer, and — beneath all of it — the living disagreement of
the schools. The compiler now carries **~27 contract classes**, a **multilateral pool primitive** that moves
beyond the bilateral ʿaqd, **madhhab-level rule modules** that make khilāf executable, and a **maqāṣid
surfacing layer** that marks — without ruling on — the question of intent. Each instrument is proven to a
uniform standard: a positive spec compiles and lowers to a contract that passes generated tests on a local
EVM; a negative control is refused with cited, span-precise diagnostics and emits nothing.

## 1. The gap, and the thesis

`deducible` began as a deep gate before a narrow door: a diminishing partnership, a profit-sharing trust, a
lease, and a common-law escrow. A descriptive knowledge graph built blind over twelve turath books
re-discovered, bottom-up, the very doctrines the compiler encoded top-down — and, by the same light, showed
how much of the economy lay outside the gate: murābaḥah and salam (the market's majority), sukūk and takāful,
the eight aṣnāf of zakat, waqf and qarḍ ḥasan, ṣarf and the monetary layer, and the madhhab divergence the
literature treats as its real subject.

The thesis of this paper is that the method scales: the *same* engine — generic AST, generic parser,
per-class semantic checks, per-class codegen — absorbs each of these as a module, and the property
"non-compliance is unrepresentable" holds across the breadth, not merely the depth.

## 2. Method: the instrument recipe

Adding an instrument is a fixed, five-step recipe: (1) name the class; (2) write its semantic check —
the cited invariants whose violation is an error; (3) lower it to a Solidity template with a generated
Hardhat test and a portable JSON manifest; (4) write a positive spec and a negative control; (5) enrol the
generated contract in the static codegen-safety property (every external value-mover carries a reentrancy
guard). Cross-cutting doctrines (zakat, contingency, the maqāṣid surfacing) are modules threaded through the
pipeline; authorities (AAOIFI, DSN-MUI, the four madhāhib) are *pure data* checked by one engine.

Two items required more than the recipe. The **multilateral primitive** — a `pool { … }` section lowering to
parallel `address[] / uint16[]` arrays with a sum-to-10000 guard and pro-rata distribution — was added
additively (alongside the scalar `parties`), so no existing instrument changed; it is what lets sukūk,
takāful, and a pooled muḍāraba exist at all. The **maqāṣid layer** is the one place the engine *warns*
rather than errors.

## 3. Results: the instruments

| Family | Classes | The riba/gharar form each refuses |
|---|---|---|
| Sale (buyūʿ) | murābaḥah, salam, istiṣnāʿ, ṣarf, tawarruq | time-priced markup; deferred-both (kāliʾ bi-kāliʾ); undescribed object; non-spot/unequal exchange; the ʿīnah ring |
| Partnership | mushārakah, muḍāraba pool | loss not by capital; a guaranteed return |
| Lease & service | ijārah, juʿāla, ʿāriyya, wakāla | rent on principal; reward not on completion; a charged "loan"; a guaranteeing agent |
| Security & credit | qarḍ ḥasan, rahn, kafāla, ḥawāla, wadīʿa | any stipulated increase; creditor-benefit/forfeit; a paid guarantee; a debt sold up; a used/guaranteed deposit |
| Capital markets | sukūk, takāful | a bond coupon; commercial insurance (gharar + maysir) |
| Social | waqf, hibah, wasiyya | a spent corpus; a gift-for-return; a bequest over one-third or to an heir |
| Charge | zakat (all genera + 8 aṣnāf) | a wrong rate per genus; a policy not summing to 100% |

Every row: positive consistent, negative **REFUSED** with cited diagnostics, generated contract green on a
local EVM.

## 4. Khilāf, made executable

The deepest gap the graph exposed was that the economy runs on *living disagreement*. The compiler now ships
four madhhab rule-modules. On a single mushārakah specification that does not stipulate that a sleeping
partner's profit tracks his capital, the engine returns **consistent under the Ḥanafī and Ḥanbalī modules**
(profit by free stipulation) and **refused under the Mālikī and Shāfiʿī modules** (profit must track capital)
— the same input, four schools, four readings, each carrying its own citation. Disagreement is no longer
flattened into a single rule; it is a selectable module, and the scholar of each school ratifies his own.

## 5. The ceiling: maqāṣid and the limit of form

A riba spec will not compile. But a *formally* perfect murābaḥah can still be, in substance, a circumvention
of riba — the graph itself isolated the hyperedge *taḥāyul al-murābaḥa ʿalā al-ribā*. Intent is not
syntactic, and a compiler cannot read niyya. The maqāṣid layer therefore does the only honest thing: on the
recognised sites of ḥiyal it raises a **warning**, never an error, pointing the faqīh at the question it
cannot itself settle. This is the deliberate boundary — the engine widens the gate; the scholar still stands
at the threshold.

## 6. Limitations and what remains human

The contribution is the Rust engine and the method. `solc` and the Hedera SDK are orchestrated as
subprocesses; the on-chain models are faithful proxies, not production custody. Three limits are
institutional, not code, and are stated plainly: (a) every rule module is authoritative only once a
qualified scholar of that authority/school ratifies it; (b) the chain holds *representations* — legal title
and real possession (qabḍ) remain an off-chain bridge; (c) the maqāṣid/intent question stays, by design,
with the human. Live testnet deployment of one representative per family, and the consensus-oracle live
proof, remain operational follow-ups (faucet-gated).

## 7. Conclusion

The fiqh of muʿāmalāt is structured enough to be computed with — not to be *ruled* by a machine, but to be
*checked* by one against a cited, human-ratified rule-base, across the whole breadth of the economy and its
schools. From four instruments to twenty-seven, from the bilateral ʿaqd to the multilateral pool, from one
standard to four madhāhib, the property held: compliance became a property of the language. The compiler does
not replace the faqīh. It gives him a wider gate to guard. *wa-Allāhu aʿlam.*
