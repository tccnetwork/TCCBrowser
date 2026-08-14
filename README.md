# TCC Browser

A browser engine for the TCC ecosystem, built for a post-quantum future.

Not a Chromium fork. Not an attempt to serve every legacy system ever shipped.

The real product of this repository is **[the TCC standard](spec/)** — a format
for signed, capability-gated applications that carry **no code**. The browser is
its reference implementation.

## Three sentences that decide the architecture

**1. Apps ship no code.** The entry point is a declarative component tree — not
markup, not script. The app states *what is on screen*; the implementation
decides *how it is drawn*.

**2. A capability does not exist until it is granted.** An app has no default
permissions. Everything reaching outside must be requested, and the user answers
item by item.

**3. A signature proves the package was NOT MODIFIED — it does NOT prove who
signed it.** Packages are self-signed and version 0.1 has no key registry. An
implementation **must not** display "verified publisher".

## Read in this order

| | |
|---|---|
| [spec/](spec/) | **The standard.** English is normative; a Vietnamese translation lives in `spec/0.1/vi/` |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Three layers, the dependency tree, the hard rules, the escape route from WebView |
| [SECURITY.md](SECURITY.md) | Threat model, what is proven and what is merely assumed |
| [conformance/](conformance/) | 104 vectors — what turns a specification into a standard |
| [examples/hello-tcc/](examples/) | A signed package, committed, verifiable in one command |

## Run it

```bash
cargo test --workspace              # 234 tests
cargo clippy --workspace --all-targets -- -D warnings
tools/kiem-luat-phu-thuoc.sh        # 13 architecture rules — MUST report 0 violations
cargo run -p tcc-conformance        # 104 conformance vectors
cargo run -p tcc-cli -- verify examples/hello-tcc
```

The architecture rules run **before** compilation in CI. Code that compiles but
violates the layering is still wrong, and finding out later is more expensive.

## Cryptography

Signatures are **hybrid**: Ed25519 **and** ML-DSA-65 (FIPS 204). Both halves
must verify. Key exchange follows the same shape: X25519 + ML-KEM.

Hybrid rather than pure post-quantum, because SIKE reached the final round of the
NIST competition and was broken **in about an hour on a single CPU core**. A
scheme surviving the whole selection process is not a proof that it holds.

The ML-DSA half is anchored to NIST ACVP vectors and cross-checked against
[`dilithium-py`](conformance/doi-chieu-doc-lap.py), an independent third-party
implementation.

> **The quietest interoperability trap in the standard:** TCC uses the FIPS 204
> **external** interface with an **empty** `ctx`. The bytes actually signed are
> `0x00 0x00 ‖ manifest_bytes`. Pick the internal interface instead and both
> sides are "FIPS 204 compliant" while neither can verify the other's
> signatures. See [spec/0.1/03-signature.md](spec/0.1/03-signature.md).

## Status, stated plainly

This is **pre-audit software implementing an unfrozen draft standard.**

- No independent security audit has happened. **No mainnet transaction may depend
  on any part of this** until one has — see [SECURITY.md](SECURITY.md).
- Version 0.1 is a working draft. It may change without notice until a freeze is
  announced.
- There is **one author, one implementation, and one conformance suite, all from
  the same party.** So "conformant to TCC 0.1" currently means *agrees with one
  implementation*, and nothing more. See [spec/GOVERNANCE.md](spec/GOVERNANCE.md).

The largest thing missing is not documentation. It is a **second, independent
implementation** — and a person who has never read this source producing a valid
package from the specification alone.

## Language

The specification is **English-normative** with a Vietnamese translation kept in
lockstep by a CI rule. Source code identifiers and comments are in Vietnamese,
as is `docs/` — the team maintaining this reads Vietnamese, and comments that
argue with the reader are worth more than comments in a language the maintainers
translate in their heads.
