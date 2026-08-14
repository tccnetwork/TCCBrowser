# TCC Standard — Governance

> **This English text is normative.** [Bản tiếng Việt](vi/GOVERNANCE.md) is a
> translation; where the two disagree, this text governs.

## 1. Where this actually stands

As of 2026-08-14, the TCC standard has **one author, one implementation, and one
conformance suite, all produced by the same party.** There is no working group,
no committee, no second implementation, and no external review.

This is written down rather than glossed over because everything else in this
repository makes security claims, and a security claim is worth what the process
behind it is worth. A standard, a reference implementation, and the test suite
that judges them, all written by the same author, cannot catch an error in the
author's *understanding* — only errors in their typing. Three real security bugs
in this project were found by asking a question no test encoded; a fourth kind,
the kind where the whole model is wrong, is exactly what a single author is
blind to.

Concretely: **conformance to this standard currently means agreeing with one
implementation.** Until a second, independent implementation exists, that is all
the phrase can honestly mean, and no one should represent it as more.

Two partial mitigations exist today, and their limits are worth stating:

| Mitigation | What it actually rules out |
|---|---|
| ML-DSA-65 vectors anchored to **NIST ACVP** | The post-quantum half is not a private dialect. It says nothing about the rest of the standard. |
| Cross-check against **`dilithium-py`**, a third-party implementation | Independent agreement on that half's arithmetic. It shares neither the packaging, the manifest, nor the capability model. |

Everything outside those two rows rests on a single author's judgement.

## 2. Who decides, today

Changes are decided by the maintainer of this repository. There is no appeal, and
no pretence of one.

That arrangement is legitimate for exactly as long as **nobody outside has
committed to the standard.** The moment a second party ships an implementation or
signs packages that other people rely on, unilateral change stops being a
maintainer's prerogative and becomes a breach of their reliance — and this
document must be replaced before that happens, not after.

## 3. The condition for taking this seriously as a standard

> A person who has not read the source code reads `spec/0.1/` and produces a
> valid **package directory** that the reference implementation accepts — **without
> asking anyone a question.**

This has **not** been done. Until it has, `spec/0.1/` is a description of an
implementation, not a specification, however carefully it is worded. The author
of a specification cannot perform this test; knowing what was meant is precisely
the thing being tested for.

The second condition, which follows it: an implementation written from the
specification alone passes 100% of the conformance vectors. The first failure
found that way will be more informative than every test in this repository.

## 4. Proposing a change

A proposal must state:

1. **The problem**, as something that can be observed — a package that ought to
   load and does not, an ambiguity two readers resolved differently, a rule that
   cannot be enforced.
2. **Who breaks**, per [`VERSIONING.md`](VERSIONING.md) §3.
3. **How it will be tested** — the conformance vectors, and per rule 2 of
   [`README.md`](README.md), no clause enters the standard without at least one.
4. **What was tried instead.** A change that could have been an erratum, an
   implementation fix, or nothing at all should be one of those.

Proposals that add a field carry a higher burden than proposals that remove one.
Because unknown fields are rejected, an added field breaks every existing
implementation ([`VERSIONING.md`](VERSIONING.md) §3), so "it's only additive" is
never an argument here.

## 5. Rules that are not the maintainer's to relax

These hold regardless of who governs the standard later, and a proposal to change
one is a proposal to abandon the design:

1. **No mainnet transaction before an independent security audit.** Stated in
   [`../SECURITY.md`](../SECURITY.md). No deadline, no demo, and no launch
   overrides it.
2. **Never display "publisher verified".** A signature proves the package was not
   modified. It proves nothing about who signed it, and 0.1 has no key registry.
   An implementation that shows a verified-publisher badge is not conformant,
   however careful the wording next to it.
3. **A capability does not exist until it is granted**, and the user answers item
   by item. Bundling permission requests defeats the model, whatever the UI
   pressure to do so.
4. **The hybrid signature never collapses to one algorithm.** See
   [`VERSIONING.md`](VERSIONING.md) §4.

Rules 2 and 3 are about what an implementation shows a person. They are part of
the standard because the cryptography is worthless if the interface misreports
what it proved — and misreporting it is the cheapest possible way to look
trustworthy.

## 6. What may be claimed

The standard and its reference implementation are published under **Apache-2.0**,
which includes a patent grant (§3). That is a deliberate part of making a second
implementation possible: permission to read is not permission to build.

| Claim | Allowed |
|---|---|
| "Passes the TCC 0.1 conformance vectors" | Yes, if it does — the vectors are public and the claim is checkable |
| "TCC 0.1 conformant" | Yes, with the standing caveat of §1: there is one implementation to agree with |
| "Audited" / "verified publisher" / "quantum-safe" | **No.** No audit has happened; 0.1 has no publisher identity; and no one can promise an algorithm holds |

There is no certification body, no trademark, and no registry of conformant
implementations. Nobody may claim to speak for the standard, including its
maintainer, beyond what these documents say.

## 7. Replacing this document

This is the arrangement for a standard with no outside users. It is expected to
be inadequate — and to be replaced before, not after, the first outside party
depends on it. The replacement must, at minimum, say who may freeze a version,
who may declare a cryptographic scheme deprecated, and what happens when the
maintainer disappears.

That last question has no answer today.
