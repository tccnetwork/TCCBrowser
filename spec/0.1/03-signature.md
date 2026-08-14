# 03 — Signature

Version 0.1 defines exactly one scheme: **`hybrid-ed25519-mldsa65-v1`**.

## Why HYBRID, not post-quantum alone

Post-quantum cryptography is young. SIKE — a NIST fourth-round candidate — was
broken in **about an hour on a single CPU core** in 2022, using classical maths.

A hybrid signature means an attacker must break **BOTH**: Ed25519 (decades of
scrutiny, but a large enough quantum computer defeats it) **and** ML-DSA-65
(quantum-resistant, but new). If either breaks, the signature still stands.

## Byte layout — PART OF THE STANDARD

Not an implementation detail. Swapping the two halves is a different package.

| | Layout | Total |
|---|---|---|
| Secret key | `[ Ed25519 seed: 32B ][ ML-DSA-65 seed: 32B ]` | **64 B** |
| Public key | `[ Ed25519: 32B ][ ML-DSA-65: 1952B ]` | **1984 B** |
| Signature | `[ Ed25519: 64B ][ ML-DSA-65: 3309B ]` | **3373 B** |

`publisher` in the manifest is the public key in lowercase hex → **3968 characters**.

**The secret key is stored as SEEDS**, not expanded keys. Far shorter, and
expandable at any time per FIPS 204. Note this differs from NIST's ACVP, which
emits ML-DSA secret keys in expanded 4032-byte form. Both are valid FIPS 204 —
they are two representations.

## Signing

```text
sign(secret_key, message) =
      Ed25519.Sign( seed[0..32],  message )      64B
   ‖  ML-DSA.Sign(  seed[32..64], message )    3309B
```

The message is the **exact bytes of `manifest.json` as they sit on disk**. Parsing
the JSON and re-serialising it breaks the signature — whitespace and key order
both change.

### ⚠️ FIPS 204 interface: EXTERNAL, EMPTY context

FIPS 204 defines **two** signing interfaces:

| | Signs what |
|---|---|
| **External** (`ML-DSA.Sign`) | `M' = 0x00 ‖ len(ctx) ‖ ctx ‖ M` |
| **Internal** (`ML-DSA.Sign_internal`) | `M` directly |

**The TCC standard uses the EXTERNAL interface with an EMPTY `ctx`.** So the bytes
actually signed are `0x00 0x00 ‖ manifest_bytes`.

This is the most important sentence on this page. An implementation that picks the
wrong interface produces signatures the other side **cannot verify** — while both
sides are "correct per FIPS 204". It is the quietest interoperability trap in the
whole standard.

Determined by measurement against NIST's sigVer vectors: the `external` group
matches, the `internal` group does not.

### Signing is DETERMINISTIC

Both halves. Ed25519 is deterministic per RFC 8032; ML-DSA uses FIPS 204's
deterministic variant (`rnd = 0^256`).

So **the same key and the same message always yield byte-identical signatures**.
The `signature` vectors rely on this, and it also makes package rebuilds
reproducible.

## Verifying

```text
verify(public_key, message, signature):
    Ed25519.Verify( pk[0..32],    message, sig[0..64]    )  MUST pass
    ML-DSA.Verify(  pk[32..1984], message, sig[64..3373] )  MUST pass
```

**BOTH MUST pass.** There is no short-circuit returning `Ok` once one half
succeeds — doing that discards the entire point of a hybrid signature and leaves
only Ed25519.

A wrong length (short or long) is an **error**, not "truncate and verify".

## Verifying a package

The signature covers the manifest. The content is bound to the manifest through
`content_hash`. So package verification is **two** steps, and skipping either
leaves a hole:

```text
1. verify(publisher, manifest_bytes, signature)      → the manifest was not modified
2. content_hash == hash(canonical_form(content/))    → the contents were not swapped
```

With step 1 alone, an attacker replaces the whole `content/` directory and the
signature still verifies.

## External anchors

An implementation **SHOULD** check itself against outside references rather than
only against itself:

| Half | Anchor |
|---|---|
| Ed25519 | RFC 8032 §7.1 TEST 1: seed `9d61b1…7f60` → public key `d75a98…511a` |
| ML-DSA-65 | NIST ACVP `ML-DSA-keyGen-FIPS204`, parameter set ML-DSA-65 |

25 ACVP cases are already extracted into
[`conformance/vectors/acvp-mldsa65.json`](../../conformance/vectors/acvp-mldsa65.json).

The **signing** direction cannot be anchored by ACVP (it emits expanded secret
keys, see above). An implementation **SHOULD** cross-check against a second,
independent FIPS 204 implementation — and **MUST** first validate that second
implementation against the NIST vectors, otherwise it is merely a second opinion:
two implementations wrong the same way still agree with each other.
