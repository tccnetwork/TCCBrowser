# Conformance vector format

Everything in `vectors/` is **plain JSON data**. Nothing here is Rust, and
nothing here needs this repository's code to be useful. That is the point: an
implementation in Go, Swift, TypeScript or anything else reads exactly these
files and decides for itself whether it conforms.

This document exists because the vectors are the *only* thing that can settle a
conformance claim, and a format nobody documented is a format nobody outside can
consume.

## Conventions across every file

| Key | Meaning |
|---|---|
| `description` | What this group covers, and why it exists |
| `cases` | The array of test cases |
| `case` | A short name for one case — this is what a runner prints when it fails |
| `expect_pass` | `true` = a conforming implementation must ACCEPT; `false` = must REJECT |
| `code` | On a rejecting case: the stable error code that must be produced |
| `notes`, `why`, `conclusion` | Prose for a human. **Never** parse these. |

Two rules a runner must follow:

1. **Match on `code`, never on an error message.** Messages are prose and may be
   reworded at any time; codes are part of the standard
   ([`spec/0.1/06-error-codes.md`](../spec/0.1/06-error-codes.md)). Rule 10 in CI
   checks that every code in the specification exists in this implementation.
2. **A rejecting case must reject for the stated reason.** An implementation that
   rejects everything passes every `expect_pass: false` case and is not
   conforming. The `expect_pass: true` cases are what stop that.

Some cases use placeholders that a runner substitutes before use, because the
real values are long and irrelevant to what the case is testing:

| Placeholder | Substitute with |
|---|---|
| `"PUB"` | any well-formed publisher key: 1984 bytes, lowercase hex |
| `"HASH"` | any well-formed content hash: 48 bytes, lowercase hex |

---

## `canonical.json` — 7 cases

The most important group for interoperability. Two implementations that disagree
here cannot verify each other's signatures at all, because the content hash goes
inside the manifest and the manifest is what gets signed.

```json
{ "case": "empty tree",
  "files": {},
  "canonical_hex": "",
  "hash_hex": "af1349b9f5f9a1a6…" }
```

| Key | Meaning |
|---|---|
| `files` | Path → file content, as a UTF-8 string |
| `canonical_hex` | The exact bytes of the canonical form, in hex |
| `hash_hex` | BLAKE3-384 of those bytes (48 bytes, 96 hex characters) |

The canonical form is defined in [`spec/0.1/01-package.md`](../spec/0.1/01-package.md):
for each file, sorted by the byte order of its path,

```text
u64 path length (big-endian) ‖ path ‖ u64 content length (big-endian) ‖ content
```

The length prefixes are not decoration. Without them a file `"ab"` containing
`"c"` and a file `"a"` containing `"bc"` both concatenate to `"abc"` — two
different trees, one signature valid for both.

The empty tree hashes to `af1349b9f5f9a1a6…`, which is the published BLAKE3 KAT
for the empty input. That anchors this group to something outside the project.

## `signature.json` — 15 cases

Checks the hybrid signature in **three** directions. Checking only verification
is not enough: an implementation that verifies our signatures but produces
signatures we cannot verify still cannot exchange packages with us.

| Section | Meaning |
|---|---|
| `keys` | `secret_hex` (64 B) and the `public_hex` (1984 B) it must derive |
| `valid_signatures` | `message_hex` + `signature_hex` that must verify |
| `broken_signatures` | Attacks that must all fail |
| `external_anchor` | RFC 8032 §7.1 TEST 1 for the Ed25519 half |
| `algorithm` | The byte layout, stated as part of the standard |
| `independent_crosscheck` | How the ML-DSA half was checked against another implementation |

Byte layout — **this is normative, not an implementation detail**:

```text
secret key  [Ed25519 seed 32B][ML-DSA-65 seed 32B]        =   64 B
public key  [Ed25519      32B][ML-DSA-65      1952B]      = 1984 B
signature   [Ed25519      64B][ML-DSA-65      3309B]      = 3373 B
```

Signing is **deterministic**: signing the same message with the same key twice
must produce identical bytes. One of the attack cases swaps the order of the two
halves, precisely because the layout is part of the standard.

> **The trap that costs the most time.** TCC uses the FIPS 204 **external**
> interface with an **empty** context, so the bytes actually signed are
> `0x00 0x00 ‖ message`. Choose the internal interface instead and both sides are
> "FIPS 204 compliant" while neither can verify the other. See
> [`spec/0.1/03-signature.md`](../spec/0.1/03-signature.md).

## `acvp-mldsa65.json` — 26 cases

NIST ACVP vectors for ML-DSA-65 keyGen, plus one sigVer case. This is the only
group whose authority is entirely external to this project.

| Key | Meaning |
|---|---|
| `seed_hex` | The 32-byte seed NIST supplies |
| `public_key_hex` | The public key that must be derived from it |
| `provenance` | Where the file came from, and when |
| `limitations` | What this group does **not** prove |

Only the **public key** is compared. NIST also publishes a 4032-byte `sk`, but
that is the *expanded* form, while this standard keeps secret keys as a 32-byte
seed. There is no bridge between the two, which is why the signing direction is
anchored by cross-checking against `dilithium-py` instead.

## `manifest.json` — 31 cases

The largest attack surface: everything downstream trusts the verdict reached
here, and this code runs **before the signature is verified**, because the
public key lives inside the manifest.

```json
{ "case": "app id missing a dot",
  "expect_pass": false,
  "code": "bad-app-id",
  "manifest": { "spec_version": "0.1", "id": "hello", … } }
```

The `manifest` object is a literal manifest, in the standard's own field names.
Three cases cover unknown fields — at the top level, inside a capability request,
and inside a capability scope. All three must be **rejected**; see
[`spec/0.1/02-manifest.md`](../spec/0.1/02-manifest.md) on why silently ignoring
a field can only ever widen a permission.

## `ui.json` — 17 cases

```json
{ "case": "minimal tree", "expect_pass": true,
  "tree": { "kind": "text", "content": "Xin chào" } }
```

The `tree` object is a literal interface tree. Content arriving from a package
must pass exactly the checks a hand-written tree passes — no leniency for being
data rather than code.

## `capability.json` — 8 cases

```json
{ "case": "exact match",
  "granted": ["shop.tcc-coin.com"],
  "requested": "shop.tcc-coin.com",
  "allowed": true }
```

| Key | Meaning |
|---|---|
| `granted` | The hosts the user actually granted |
| `requested` | The host the app is trying to reach |
| `allowed` | Whether the request may proceed |

Matching is **exact**. No suffix matching, no subdomains, no wildcards. A
one-character difference here is the app reaching an attacker's server.

---

## Running them

This repository's own runner is one implementation of these rules:

```sh
cargo run -p tcc-conformance                 # summary
cargo run -p tcc-conformance -- --chi-tiet   # case by case
```

A non-zero exit status means something failed. You are not expected to use it —
writing your own runner against this document is the point, and if this document
is not enough to write one, that is a defect in this document.
