# TCC conformance suite — specification version 0.1

> This is what turns a **specification** into a **standard**. Without it, "an app
> must implement things the TCC way in order to run" is unfalsifiable. With it,
> the claim becomes measurable: run the suite, and only 100% earns the name.

**The schema is documented in [FORMAT.md](FORMAT.md).** Read that before writing
a runner; this file explains why the suite exists, that one explains how to
consume it.

## Vectors are DATA, not code

Every case lives in `vectors/*.json`. That is deliberate: an implementation in
Go, Swift, TypeScript or anything else reads exactly these files. Vectors written
in Rust would be **our** unit tests, not the **standard's** conformance suite.

## Match on ERROR CODES, not messages

Every failing case names a `code` — for example `unsafe-display-string`. Error
messages are prose written for a human and may be reworded at any time; codes may
not. **Changing a code changes the standard.**

Wrapped errors report the code of the **root cause**. A manifest rejected for a
spoofing character reports `unsafe-display-string`, not `spec`.

Two CI rules keep this honest: every code in the specification must exist in the
source, and the translation's set of codes must match the normative one exactly.

## Run

```sh
cargo run -p tcc-conformance                 # everything, with a summary
cargo run -p tcc-conformance -- --chi-tiet   # case by case
```

A non-zero exit status means something failed.

## Six vector groups — 104 cases

| File | What it checks | Why it matters |
|---|---|---|
| `canonical.json` | Canonical form + content hash | **Interop**: two implementations must produce the SAME bytes and the same hash, or neither can verify the other's signatures |
| `signature.json` | Hybrid signature: keygen · sign · verify | Both halves must verify; a package passing on only one is a forgery that looks genuine |
| `acvp-mldsa65.json` | The post-quantum half, anchored to **NIST ACVP** | Proves ML-DSA-65 here is not a private dialect. It is the only part of this suite whose authority is external |
| `manifest.json` | Accepting and rejecting manifests | The largest attack surface — everything downstream trusts the verdict reached here |
| `ui.json` | Accepting and rejecting interface trees | The tree arrives from the package and must clear the same checks as a hand-written one |
| `capability.json` | Capability scope matching | A one-character mismatch is the app reaching an attacker's server |

## An independent cross-check

[`doi-chieu-doc-lap.py`](doi-chieu-doc-lap.py) re-verifies the ML-DSA half
against [`dilithium-py`](https://github.com/GiacomoPope/dilithium-py), a
third-party implementation sharing no code with this one.

Its limits are worth stating: it agrees only on that half's arithmetic. It shares
none of the packaging, the manifest rules, or the capability model. Everything
outside those bounds still rests on a single author's judgement.

## Adding a vector

Every clause in `spec/` must have **at least one** vector here. Adding a rule to
the specification without adding a vector is adding a promise nobody can check.

A vector that passes both before and after the change it is meant to cover proves
nothing. Mutate the implementation, watch the vector go red, then restore it.
