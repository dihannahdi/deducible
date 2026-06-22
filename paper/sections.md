# Compliance by Construction: Enforcing Shari'ah Substance in a Diminishing Partnership on a Public Ledger

> Draft sections (Introduction, Literature Review, Discussion, Conclusion). Methodology
> and Results are in `methodology_results.md`. All scriptural, hadith, and standards
> citations are flagged `[verify]` and must be confirmed against primary sources before
> submission; an unverified attribution to the Qur'an, the Sunnah, or AAOIFI must not stand.

## 1. Introduction

For half a century, Islamic finance has lived with a quiet contradiction. Its normative
ideal is participation — the sharing of profit and loss, the binding of finance to the
real economy, the refusal of a guaranteed return on money lent (Qur'an, al-Baqarah 2:275
`[verify]`). Its dominant practice, however, leans on debt-like instruments — *murabaha*,
and especially organized *tawarruq* — whose economic substance reproduces the
interest-bearing loan they were meant to replace. The critique is neither marginal nor
new: the OIC International Islamic Fiqh Academy declared organized *tawarruq*
impermissible (Resolution 179, Sharjah 2009 `[verify]`), and a leading jurist of the
field argued that the great majority of *sukuk* then in circulation did not, in
substance, meet the requirements of the *Shari'ah* (Usmani, c. 2007 `[verify]`). The
charge that recurs is *form over substance*: a contract certified once, in form, while
its substance drifts.

This paper begins from an inversion. The conventional framing asks what blockchain can do
*for* Islamic finance, casting the technology as protagonist and the *Shari'ah* as a
use-case. We reverse the order. The *Shari'ah* economy — its insistence on real
ownership, shared risk, and the just circulation of wealth "so that it does not circulate
only among the rich among you" (al-Hashr 59:7 `[verify]`) — is itself the radical
proposal. Blockchain is merely the first execution substrate adequate to run that
proposal *faithfully*, because it can make economic substance continuously verifiable and
contractually enforced rather than periodically attested.

From this inversion we identify a single research gap. In contemporary practice, and in
the existing literature on blockchain for Islamic finance, *Shari'ah* compliance is an
**attestation**: a board certifies an instrument, and thereafter no party can
continuously verify that its substance still matches its form. We argue that the four
fault lines most often pressed against Islamic finance — form-over-substance, the
under-use of risk-sharing, the tension between contractual flexibility and finality, and
the uncertainty (*gharar*) at the boundary with the real world — are not four diseases
but symptoms of one: **compliance is asserted, not enforced**.

Our research question follows: *can the conditions that distinguish a genuine
Shari'ah-compliant partnership from disguised debt be enforced by the contract's own
execution, rather than attested after the fact — and can this be done at industrially
viable cost?* We answer it constructively. We build a *Musharakah Mutanaqisah*
(diminishing partnership) as a smart contract on Hedera, encode four *fiqh*-derived
conditions as invariants the protocol enforces, and demonstrate the instrument through a
full lifecycle on a public test network at a cost of a fraction of one ℏ. We call the
principle **compliance by construction**.

The remainder of the paper proceeds as follows. Section 2 reviews the normative ideal and
its practical erosion, the form-over-substance critique, and prior work on blockchain in
Islamic finance, locating our gap. Section 3 sets out the design-science methodology and
the artifact. Section 4 reports functional, on-chain, and cost results. Section 5
discusses what the results mean, their limits, and the trust they relocate but do not
abolish. Section 6 concludes.

## 2. Literature Review

**The ideal and its erosion.** The prohibition of *riba* is the settled ground of Islamic
commercial law (al-Baqarah 2:275 `[verify]`), and from it the jurists derive a preference
for participatory finance — *musharakah* and *mudarabah* — over lending at interest.
Yet a substantial literature documents that these profit-and-loss-sharing (PLS)
instruments remain marginal in the balance sheets of Islamic banks, which concentrate
instead in *murabaha* and related mark-up sales. The explanation offered is largely one
of agency: under asymmetric information, the cost of verifying an entrepreneur's true
profit, and the moral hazard it invites, make PLS expensive to police, so institutions
default to debt-like structures (see the PLS agency-cost literature `[verify]`).

**The form-over-substance critique.** A second body of work interrogates whether the
debt-like instruments are *Shari'ah*-compliant in substance or only in form. The
strongest statements come from within the tradition: El-Gamal's analysis of *Shari'ah
arbitrage* and *hiyal* (legal stratagems) `[verify]`, and Usmani's critique of *sukuk*
structures that promise a fixed return while wearing the garment of partnership
`[verify]`. AAOIFI's *Shari'ah* Standards — including the standard on *Musharakah* and
diminishing partnership (No. 12 `[verify]`) — respond by tightening the conditions:
rent must attach to a genuinely co-owned asset, and the partner's purchase undertaking
must be at fair or market value, not a pre-fixed price that guarantees the financier's
capital. These conditions are precisely the ones an attestation regime cannot
*continuously* guarantee.

**Blockchain and Islamic finance.** A third, rapidly growing literature applies
distributed ledgers to Islamic finance. Most of it treats blockchain as a *transparency*
and *tokenisation* layer — "smart *sukuk*", asset provenance, *zakat* and *waqf* tracking
`[verify]`. This work is valuable but, with few exceptions, leaves the compliance
*decision* where it has always been: with a human board, certifying ex post. The ledger
records, but does not enforce.

**The gap.** Across these literatures, *Shari'ah* compliance is consistently modelled as
something *attested* — by boards, by audits, by disclosure. We find no body of work that
re-casts compliance as a property *enforced by the instrument's own execution*, such that
the conditions distinguishing genuine partnership from disguised debt cannot be violated
without the transaction failing. That re-casting — compliance by construction — is the
gap this paper addresses, and the diminishing partnership, sitting at the intersection of
the form-over-substance and risk-sharing fault lines, is the instrument on which we test
it.

## 5. Discussion

**What the results show.** The artifact demonstrates that the conditions a diminishing
partnership must satisfy can be enforced by the contract itself. On a public network,
rent fell automatically as ownership transferred; a buyout could not be priced except
from an independent attested valuation; and a fall in asset value was borne by both
partners in proportion to their live ownership, so the financier could not exit whole.
None of these required a board's intervention; each is a property of execution. The
critique of *form over substance* is met not by argument but by construction: here the
substance *is* the form, because the form will not execute unless the substance holds.

**Dialogue with the literature.** Where the agency-cost literature explains the retreat
from PLS by the prohibitive cost of verifying a partner's position, our results suggest a
mechanism that lowers that cost toward zero for the verifiable components: ownership,
rent, and loss apportionment become machine-checked state transitions rather than audited
claims. Where El-Gamal and Usmani indict *Shari'ah arbitrage*, compliance by construction
removes the space in which the arbitrage operates — the gap between certified form and
unobserved substance. We do not claim to have abolished that gap entirely; we claim to
have closed it for the properties the protocol can see.

**Theoretical implication: Shari'ah as protocol.** The deeper implication is a shift in
where compliance *lives*. It moves from a periodic, human, ex-post judgement to a
continuous, computational, ex-ante property. This reframes the Shari'ah board's role from
*certifier of an instrument* to *auditor of a protocol* — a change with consequences for
standard-setting bodies such as AAOIFI, which may in time express standards as verifiable
contract specifications rather than prose alone.

**Limitations, stated plainly.** Three boundaries deserve candour. First, the *oracle*.
The artifact enforces that no contracting partner can set the asset's value, but the
integrity of the independent valuer remains an external assumption — the locus of
*gharar* is relocated and made explicit, not eliminated (the Prophet, peace be upon him,
forbade *bay' al-gharar*, reported in Sahih Muslim `[verify]`). Second, *flexibility*.
Classical *fiqh al-mu'amalat* provides for *khiyar* (options to rescind) and *iqalah*
(mutual cancellation); a contract whose virtue is finality is in tension with these, and
our present artifact does not encode them. Reconciling enforced finality with lawful
rescission is necessary future work, not a solved problem. Third, *scope*: a single
instrument, on a test network, with the two partner roles exercised from one account for
demonstration; production deployment, multi-party operation, and a decentralised valuation
network remain ahead. A fourth, smaller finding — that on-chain value granularity (whole
tinybars) constrains how real-world prices and micro-rents are represented — is a concrete
engineering constraint any deployment must design around.

**Practical and policy implications.** For practitioners, the cost results
(a full lifecycle for a fraction of one ℏ) place the approach within industrial reach,
particularly against the recurring servicing, custody, and audit costs of the
conventional analogue. For regulators and *Shari'ah* boards, the artifact suggests a
supervisory model based on auditing an enforced specification rather than sampling
completed transactions.

## 6. Conclusion

This paper asked whether the conditions distinguishing a genuine *Shari'ah*-compliant
partnership from disguised debt could be enforced by a contract's own execution rather
than attested after the fact, and at viable cost. Building a *Musharakah Mutanaqisah* on
Hedera and exercising it through a full lifecycle on a public network, we answered yes for
the properties a protocol can verify: rent that diminishes with ownership, a buyout priced
only from an independent valuation, and loss shared by live ownership ratio — at a cost of
a fraction of one ℏ for the entire lifecycle.

The contribution is a principle, *compliance by construction*, and a demonstration that it
is buildable. But the larger claim is the one we began with: the *Shari'ah* economy is the
revolutionary proposal, and the ledger is only the first instrument honest enough to run
it. The work that remains — encoding *khiyar* and *iqalah*, decentralising the valuer,
moving from one instrument to the full family of participatory contracts, and submitting
all of it to qualified scholarly review — is the measure of how much remains to be done,
not of whether the road is real. Allahu a'lam.
