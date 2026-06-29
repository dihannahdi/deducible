# The Five Great Qawāʿid and the Compiled Invariants

*How the engine's machine-checkable rules descend from the five universal maxims (al-qawāʿid
al-fiqhiyya al-kubrā). This is a documented mapping for the reader and the ratifying scholar, not
an inference engine: the compiler enforces the specific invariants below; it does not derive new
ones from the maxims. Citations from al-Suyūṭī / Ibn Nujaym (al-Ashbāh wa-l-Naẓāʾir) and Ibn
al-Mulqin; all [scholar-verify]. Allāhu aʿlam.*

The Turath knowledge-graph (see `turath-graph/GRAPH_REPORT.md`) surfaced these five as the corpus's
own "god-nodes" — the maxims the literature organises itself around. Each maps to a family of the
engine's diagnostic codes:

## 1. al-umūr bi-maqāṣidihā — "matters are by their intentions"
The substance-over-form principle. → the **maqāṣid / ḥiyal-risk surfacing** layer (`MAQASID-1/2`,
`crates/fiqhc/src/maqasid.rs`): the engine polices form and *surfaces* where the maqṣad (a possible
ḥiyal / circumvention) must be judged by a scholar. It is the one place the engine warns rather than
rules — precisely because intention is not syntactic. Also the **Composition** cycle detector
(`INAH-1/2/3`, `composite.rs`): ʿīnah/tawarruq are forbidden by the *shape* (intent) of the ring, not
the markup.

## 2. al-yaqīn lā yuzāl bi-l-shakk — "certainty is not removed by doubt"
Determinacy against gharar. → `GHARAR-1/2`, `ORACLE-1..5` (the consensus oracle: a *majhūl* price
reverts), and the definite-price/known-object requirements of the sales (`MUR-1`, `SALAM-1/2/3`,
`ISTISNA-1/3`, `SARF-1/2`, `JUALA-1`, the `CERTAINTY-*` of the common-law engine). The contract's
terms must be known, not doubtful.

## 3. al-mashaqqa tajlib al-taysīr — "hardship brings ease"
The licence of relief. → the **Contingency** off-ramps (`CONT-1` waḍʿ al-jawāʾiḥ — rent abates under
calamity, never accrues as interest; `CONT-2/3` farāʾiḍ on death) and the rescission family (khiyār
al-sharṭ, iqāla, faskh). The law bends to relieve genuine hardship without breaking the riba line.

## 4. al-ḍarar yuzāl — "harm is removed"
Liability and the blocking of means. → the risk-bearing invariants (`RISK-1/2/3` — loss follows
capital/ownership, the owner bears the asset; *al-kharāj bi-l-ḍamān*), the no-penalty-riba rules
(`RIBA-3`), the **Capacity** layer (ahliyyah — barring the incompetent from harm), and *sadd
al-dharāʾiʿ* operationalised as the cycle detector. Harm — to a partner, a debtor, the incapable — is
designed out.

## 5. al-ʿāda muḥakkama — "custom is arbitral"
Ratified practice governs. → the **pluggable rule modules** (`rules/*.rules.json`): AAOIFI, DSN-MUI,
and the four madhāhib (ḥanafī/mālikī/shāfiʿī/ḥanbalī) each encode a community's ratified ʿurf/ijtihād,
and the *same engine* yields a different verdict per authority (the live khilāf, `tests/madhhab.rs`).
What is "the custom" is the authority's to declare; the engine only applies the declared module.

---

**The boundary, restated.** These maxims are the *why*; the diagnostic codes are the *what the
engine checks*. The engine does not reason from maxim to ruling — that is ijtihād, the scholar's. It
applies a human-authored, citation-bearing rule-base, and (in maxim 1's spirit) surfaces the question
of intent it cannot itself settle. *wa-Allāhu aʿlam.*
