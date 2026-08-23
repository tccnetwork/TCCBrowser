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

## `signature.json` — 9 cases

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

## `acvp-mldsa65.json` — 25 cases

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

## `manifest.json` — 34 cases

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

## `ui.json` — 27 cases

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


## `package.json` — 26 cases

Package-layer rules from [`01-package.md`](../spec/0.1/01-package.md): path
constraints and case collisions. Three arrays — `cases`, `signature_file`,
`missing_file` — because the last two vary one file at a time against an
otherwise valid package, and flattening them would lose which file was touched.

⚠️ `duplicate-path` **cannot be expressed here**: a JSON object cannot hold the
same key twice. A conforming implementation still has to reject it; this file
cannot be what proves that. Stated in the file's own `description`, and repeated
here because a reader who only skims section headings would otherwise conclude
the code is untested.

## `verify.json` — 10 cases

End-to-end verification, and — the part that matters — the **order** the checks
run in. [`01-package.md`](../spec/0.1/01-package.md) calls that order a security
property: the size cap comes before parsing, the scheme comparison before the
cryptography. A package that breaks two rules at once must report the code of the
**first** check to fail, so these cases pin an ordering, not just a verdict.
## `layout.json` — 59 cases — **DRAFT, not conformance-bearing**

⚠️ This file belongs to [`spec/0.2`](../spec/0.2/), which is **not released**. No
implementation satisfies it, this repository's runner does not load it, and the
two error codes it uses — `bad-layout`, `bad-scroll` — exist in no released
error-code table. Treat it as a proposal that happens to be machine-readable.

Twenty-five accept/reject cases and thirty-four geometry cases. It is the only
file in `vectors/` with **two kinds of case**, and that is the part worth
documenting: every other group answers *accept or reject*, and a layout
requirement is neither. "These two children are the same size" cannot be written
as a verdict on a package.

### `cases` — the shape every other file already uses

`case`, `clause`, `expect_pass`, `code` on a rejecting case, and `tree`. Nothing
new. `clause` names the section of
[`spec/0.2/05-interface.md`](../spec/0.2/05-interface.md) the case checks — rule
2 of [`spec/README.md`](../spec/README.md) requires every clause to have at least
one vector, and `clause` is how that is checked mechanically.

### `geometry` — a second shape, and why it had to exist

Each case has `case`, `clause`, `frame`, `direction`, `tree`, an optional
`scroll`, and `assert`. A runner lays `tree` out into `frame`, applies `scroll`,
then checks every entry of `assert`.

| Key | Meaning |
|---|---|
| `frame` | `{"width": w, "height": h}` in the implementation's **own units**. These are not app-declared lengths — no package can see or set them — so they do not reopen the hole that "no pixels, no colours" closes |
| `direction` | Always `"ltr"` in this draft. Right-to-left geometry is unspecified |
| `path` | Child indices from the root. **The root is `[]`, not `[0]`.** `[1, 0]` is the first child of the second child of the root |
| `scroll` | Optional list of `{"path": p, "to": "start"｜"end"}`, applied before measuring, so a case can assert what is true at the far end of a scroll container |
| `assert` | A list of relations, described below |

**Assertions are RELATIONS, never magnitudes.** This is the rule that makes the
shape work at all: implementations choose their own `gap` and `padding` scale, so
"this box is 8 units from that one" is not a conformance question — but "these
two boxes are the same width" is. Four kinds:

| `kind` | Holds when |
|---|---|
| `equal` | `a == b` |
| `at_least` | `a >= b` |
| `greater` | `a > b` |
| `contains` | the `inner` node's box lies entirely within the `outer` node's inner box |

The three numeric kinds take `a` and `b`, each either `{"path": p, "of": "x"｜"y"｜"width"｜"height"}` or a literal `{"value": n}`. `contains` takes `outer` and
`inner`, both paths.

A literal `{"value": n}` is only legitimate where the number comes from the
**frame**, which the runner supplied — never from the implementation's own
spacing choices. The root-node case is the archetype: the root's box *is* the
frame, so asserting it equals `1200 × 800` is asserting the standard, not a unit
system.

### What this format still cannot express

Several clauses in 05 bind the **renderer**, not the validator: the spacing scale
must increase, content is never clipped, every part of a scroll container must be
reachable, focus scrolls into view. No package can violate them, so they carry no
error code — and the geometry cases are the only thing watching them. A clause
with nothing watching it is the first kind of defect
[`spec/README.md`](../spec/README.md)'s audit names.

## Running them

This repository's own runner is one implementation of these rules:

```sh
cargo run -p tcc-conformance                 # summary
cargo run -p tcc-conformance -- --chi-tiet   # case by case
```

A non-zero exit status means something failed. You are not expected to use it —
writing your own runner against this document is the point, and if this document
is not enough to write one, that is a defect in this document.
