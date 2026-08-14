# TCC Standard — Versioning and Deprecation

> **This English text is normative.** [Bản tiếng Việt](vi/VERSIONING.md) is a
> translation; where the two disagree, this text governs.
>
> This policy applies to **the standard**, not to the reference implementation
> and not to apps. An app's own `version` field is its business; the standard
> does not constrain its format.

## 1. A version is a directory, and it is immutable

The standard is versioned as `spec/<major>.<minor>/`. Once a version is
**frozen**, its files never change again — not a typo fix, not a clarification.
Changing them would mean two people who both "implemented 0.1" implemented
different things, with nothing in either copy to reveal it.

Corrections to a frozen version go in `spec/<version>/ERRATA.md`, which is
append-only. An erratum **MUST NOT** change what conforms; it may only make the
existing rule clearer. If a correction would change what conforms, it is not an
erratum — it is the next version.

Version 0.1 is **not yet frozen**. It is a working draft and may change without
notice until the freeze is announced per [`GOVERNANCE.md`](GOVERNANCE.md).

## 2. There is no forward compatibility, by design

`spec_version` **MUST** match exactly. An implementation of 0.1 rejects a 0.2
package; it never guesses, never runs it partially, never falls back.

This is a deliberate choice and it is the reason unknown fields are rejected
([02 §Unknown fields](0.1/02-manifest.md)). Most formats buy graceful degradation
by ignoring what they don't understand. TCC cannot afford that, because in this
format the thing being ignored can be a **permission boundary**: a scope that
grows a narrowing field is *widened* by any implementation that skips it. Failing
to open a package is a visible, recoverable failure. Silently granting more than
the manifest says is neither.

So the trade is stated plainly: **TCC has no graceful degradation, and gains that
what is signed is exactly what is checked.**

## 3. What counts as breaking

The usual "additive is safe" rule does not hold here. Because unknown fields are
rejected, **every added field is a breaking change** — a package using it will
not load on any older implementation.

| Change | Version bump |
|---|---|
| Adding any manifest, scope, or interface field | **major or minor — always** |
| Adding a component kind or capability name | **major or minor — always** |
| Removing or renaming anything | **major** |
| Changing canonical serialization or the content hash | **major** |
| Changing which bytes are signed, or the FIPS 204 interface | **major** |
| Narrowing what an existing field permits | **major** |
| Adding a new error code for a case that previously had none | minor |
| Clarifying prose, adding examples, adding conformance vectors that pass on the existing implementation | none — erratum |

**Minor versions are not compatible with each other in either direction.** 0.1
and 0.2 are two standards that happen to be related. The distinction between
major and minor here records *how much of the design survived*, for humans
reading the history — it is not a compatibility promise, and no implementation
may derive one from it.

Any change that adds a field **MUST** be accompanied by conformance vectors that
**fail** on the previous version's implementation. A version bump nobody can
detect is not a version bump.

## 4. Deprecating cryptography

This is the part that will actually be exercised. `hybrid-ed25519-mldsa65-v1`
exists because nobody knows which half breaks first.

**Signing and verifying deprecate separately, and never together.**

1. **Discouraged.** The scheme still signs and still verifies. Tools warn when
   signing. Nothing an existing user holds stops working.
2. **Verify-only.** New packages **MUST NOT** be signed with it; implementations
   **MUST** still verify existing ones. This step lasts at minimum until the
   ecosystem has re-signed, and there is no maximum.
3. **Refused.** Verification fails. This makes every package ever signed with the
   scheme unopenable.

Step 3 **MUST NOT** be taken in the same version as step 2, and **MUST NOT** be
taken while any known package still relies on the scheme. Removing verification
is deleting data — the app becomes unopenable and its stored state unreachable —
so it is treated as data destruction, not as maintenance.

**One exception, and it inverts the order:** if a scheme is broken such that
*forgery* is practical, verify-only is not a safe resting place, because
verifying a forgeable signature is worse than refusing a genuine one. Then step 3
follows immediately and the standard says so explicitly, naming the break. A
scheme that is merely *weakened* — not forgeable, just no longer at the target
security level — follows the normal order.

**A new scheme never replaces a hybrid with a single algorithm.** The hybrid
exists so that one broken half is survivable. Retiring `ed25519` means replacing
it with another classical algorithm alongside the post-quantum half, not dropping
to ML-DSA alone. This constraint may only be lifted by a major version that
argues the case in the specification text itself.

## 5. Deprecating a capability

A capability may not be silently narrowed or removed: an app that requested it is
already installed, already granted, and its stored permission decisions are keyed
by scope ([`ghi_nho.rs`](../crates/tcc-shell/src/ghi_nho.rs)).

Removing a capability in version N means: apps declaring it fail to load under N.
That is acceptable — it is visible and it fails closed. What is **NOT** permitted
is keeping the capability name while changing what its scope grants, in either
direction. Widening silently gives an installed app more than the user agreed to;
narrowing silently breaks it with no diagnosis. Change the meaning, change the
name.

## 6. Announcing a version

A new version is not released by committing files. It requires, in one place:

- what changed, and for each change, why the previous text was wrong;
- which conformance vectors are new, and which now fail on the previous version;
- what an implementer of the previous version has to do;
- for any deprecation, the current step (§4) and what triggers the next one.

If none of that can be written, the change is not ready.

## 7. Version 0.1 in particular

0.1 has **no upgrade mechanism** — no way to replace an installed package with a
newer one, and therefore no migration story for stored data. This is listed as a
known gap in [`0.1/README.md`](0.1/README.md), and it means a real 0.2 will have
to define package updates before anything can be deployed that expects to
outlive it.

Nothing in 0.1 should be treated as frozen until the freeze is announced. **No
mainnet transaction may depend on any of it before an independent security
audit** — that gate is stated in [`../SECURITY.md`](../SECURITY.md) and this
policy does not override it.
