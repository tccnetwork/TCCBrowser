# The TCC Standard

This directory is **the real product** of the project. The browser is merely its
reference implementation.

## Rules for this directory

1. **Extracted from running code, never written first.** Standards written ahead
   of an implementation mostly die (XHTML 2.0, SOAP, the WS-\* family). The ones
   that survive were extracted from something that already ran (HTML5, HTTP).
2. **Every clause needs at least one check** in [`conformance/`](../conformance/).
   Adding a clause without a vector is adding a promise nobody can verify.
3. **A version is never edited in place.** `0.1/` freezes on release; changes open
   `0.2/`. See [VERSIONING.md](VERSIONING.md).

## The only test that proves this is written clearly enough

> Someone who has never read the source code reads `spec/0.1/` and produces a
> valid `.tccapp` package **without asking anyone a question.**

This has **not** been done yet. Until it has, `spec/0.1/` is a description of an
implementation rather than a specification, however carefully it is worded — and
the author of a specification cannot perform this test, because knowing what was
meant is exactly the thing being tested for.

## Status

`0.1/` — **written** (2026-08-14), extracted from the running reference
implementation. A working draft; **not frozen**.

**English is NORMATIVE.** Vietnamese is a translation for the team and the TCC
community; where the two disagree, the English text governs.

| | Normative (en) | Translation (vi) |
|---|---|---|
| Overview | [README](0.1/README.md) | [README](0.1/vi/README.md) |
| Package | [01](0.1/01-package.md) | [01](0.1/vi/01-goi.md) |
| Manifest | [02](0.1/02-manifest.md) | [02](0.1/vi/02-ban-ke-khai.md) |
| Signature | [03](0.1/03-signature.md) | [03](0.1/vi/03-chu-ky.md) |
| Capabilities | [04](0.1/04-capabilities.md) | [04](0.1/vi/04-quyen-nang.md) |
| Interface | [05](0.1/05-interface.md) | [05](0.1/vi/05-giao-dien.md) |
| Error codes | [06](0.1/06-error-codes.md) | [06](0.1/vi/06-ma-loi.md) |

Two documents apply to **every version**, not just 0.1:

| | Normative (en) | Translation (vi) |
|---|---|---|
| Versioning & deprecation | [VERSIONING](VERSIONING.md) | [VERSIONING](vi/VERSIONING.md) |
| Governance | [GOVERNANCE](GOVERNANCE.md) | [GOVERNANCE](vi/GOVERNANCE.md) |

[`GOVERNANCE.md`](GOVERNANCE.md) §1 states the thing most easily glossed over:
this standard has **one author, one implementation, and one conformance suite,
all produced by the same party.** So "conformant to TCC 0.1" currently means
*agrees with one implementation*, and nothing more.

## Three CI rules keep the specification from drifting

| Rule | Checks | Why |
|---|---|---|
| 10 | Every error code in the specification **exists in the source** | A code that exists only on paper is a promise nobody keeps — an outside implementer following it would never match the conformance suite |
| 11 | The translation **does not drift** from the normative text (file count, error-code set, and every policy document has a translation) | A skewed translation is worse than no translation: its readers implement a different standard without anyone knowing |
| 12 | The specification contains **no dead links** | Outsiders reading it have no source code to guess from — a dead link is a rule pointing at nothing |

All three are mutation-tested: the check is proven to go red when the thing it
guards is broken, and green again when it is restored.

## Still missing before this deserves the word "standard"

- **A second, independent implementation.** This is the largest gap, and every
  other item on this list is smaller than it.
- **The Phase 2 exit gate, unverified**: someone who has never read the source
  building a valid package from `spec/0.1/` alone. Not something that can be
  self-verified.
