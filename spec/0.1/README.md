# The TCC Standard — version 0.1

> **Status:** working draft. Extracted from the running reference implementation,
> per rule 1 of [`spec/README.md`](../README.md). Not yet frozen.
>
> **This English text is normative.** [Bản tiếng Việt](vi/README.md) is a
> translation provided for the TCC team and community; where the two disagree,
> this text governs.

## Who this is for

Anyone writing their **own TCC implementation** — building packages, signing
them, or running them — in any language. If you finish reading and still have to
ask someone before you can produce a valid package, this document has failed.
That is its only test.

## What is normative, and what this version depends on

Everything an implementation must satisfy to conform to 0.1 is stated **inside
this directory**, with its own **MUST**. Nothing outside it is
conformance-bearing.

That is deliberate, and it has a consequence worth stating plainly. Four places
in these documents link to [`../VERSIONING.md`](../VERSIONING.md) — for why a
list only grows with a version bump, and for what counts as a breaking change.
Those links are **informative**: they explain why a rule is what it is, or tell
a spec editor what to do next. None of them attaches a requirement to the
linked text.

The reason to say so is that `VERSIONING.md` lives **outside** any versioned
directory, so unlike this directory it is not immutable. Its wording, and its
section numbers, can change after 0.1 freezes, with no erratum trail. If those
links were conformance-bearing, a requirement of 0.1 could be rewritten without
a version bump — which is the exact failure `VERSIONING.md` §1 exists to
prevent. They are not, so it cannot; a reader who follows one and finds
different prose has lost an explanation, not a rule.

An implementer who never opens those links can still produce a valid package.
If that ever stops being true, the rule has to move **into** this directory.

## Terms

| Term | Meaning |
|---|---|
| **MUST** | Required. Not doing it means not conforming. |
| **MUST NOT** | Forbidden. Doing it means not conforming. |
| **SHOULD** | Strongly recommended; deviating requires a reason you can write down. |
| **MAY** | Optional. |

## Read in this order

| | |
|---|---|
| [01 — Package](01-package.md) | On-disk layout, canonical form, content hash |
| [02 — Manifest](02-manifest.md) | Every field and constraint |
| [03 — Signature](03-signature.md) | Hybrid signature, byte layout, **the FIPS 204 interface** |
| [04 — Capabilities](04-capabilities.md) | Scopes and matching rules |
| [05 — Interface](05-interface.md) | The declarative component tree |
| [06 — Error codes](06-error-codes.md) | Stable codes for matching |

## Three sentences that decide the architecture

Read these first; everything else follows from them.

**1. Apps ship no code.** The entry point is a **declarative component tree** —
not markup, not script. The app states *what is on screen*; the implementation
decides *how it is drawn*. See [05](05-interface.md).

**2. A capability does not exist until it is granted.** An app has no default
permissions. Everything that reaches outside must be requested, and the user
answers **item by item**. See [04](04-capabilities.md).

**3. A signature proves the package was NOT MODIFIED — it does NOT prove who
signed it.** The public key sits inside the manifest; packages are **self-signed**.
An implementation **MUST NOT** display "verified publisher". Version 0.1 has no
key registry.

## Conformance

A conforming implementation **MUST** pass 100% of the vectors in
[`conformance/vectors/`](../../conformance/vectors/). That directory sits
outside this one, so one constraint keeps it from moving what 0.1 means: **a
vector may only test a requirement already stated in this directory.** Adding
such a vector is an erratum and needs no version bump, because an
implementation that failed it was already not conforming — the prose said so
before the vector existed. A vector testing anything else is a change to the
standard and needs a version bump, whatever it does to the reference
implementation.

| Group | Checks |
|---|---|
| `canonical` | Canonical form + content hash |
| `signature` | Hybrid signature: keygen · sign · verify |
| `acvp-mldsa65` | Post-quantum half, anchored to NIST |
| `manifest` | Accepting/rejecting manifests |
| `ui` | Accepting/rejecting interface trees |
| `capability` | Scope matching |

Vectors are **JSON data**, not code — readable from any language. Match on
**stable error codes** ([06](06-error-codes.md)), never on error messages.

## What version 0.1 does NOT have

Stated so nobody assumes otherwise:

- **Key registry / identity.** Signatures prove integrity only.
- **Signature context** (FIPS 204 `ctx`) — always EMPTY, see [03](03-signature.md).
- **Executable apps** (WASM). Declarative interface only.
- **Multiple screens / navigation.** One package, one entry point.
- **Package updates.** No upgrade mechanism is defined.
- **A container format.** A package is a *directory*; there is no archive and no
  `.tccapp` file format. See [01](01-package.md).
