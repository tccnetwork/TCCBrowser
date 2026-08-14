# 06 — Error codes

## Why codes, when there are already messages

An error message is **prose for a human**. It may be reworded for clarity at any
time, and it may be translated. Nothing can match against it.

An error code is **stable**. The conformance suite and every implementation match
on it. **Changing a code changes the standard** and requires a version bump.

## Rules

A conforming implementation **MUST** use these exact codes when rejecting.

A wrapping error **MUST** report the code of the **root cause**. A manifest
rejected for a spoofing character must report `unsafe-display-string` — reporting
a generic code like `spec` says nothing and cannot be matched by the conformance
suite.

## The list

### Package and paths

| Code | When |
|---|---|
| `empty-path` | Empty path |
| `bad-path` | Violates the path constraints in [01](01-package.md) |
| `duplicate-path` | Two files with the same path |
| `case-collision` | Two files differing only by case |

### Manifest

| Code | When |
|---|---|
| `bad-json` | Malformed JSON, or duplicate keys |
| `manifest-too-large` | Over 64 KiB |
| `unsupported-spec-version` | `spec_version` is not `"0.1"` |
| `bad-app-id` | `id` violates the constraints in [02](02-manifest.md) |
| `unsafe-display-string` | A user-visible string contains banned characters, or too many stacked marks |
| `not-hex` | A field required to be hex is not |
| `bad-hex-length` | Hex string of the wrong length |
| `publisher-not-hex` | `publisher` is not hex |
| `scheme-mismatch` | `scheme` is not the scheme in use |
| `content-hash-mismatch` | Content hash does not match the manifest |
| `bad-entry` | `entry` is not a valid path |
| `missing-entry` | `entry` is not present in the package |

### Capabilities

| Code | When |
|---|---|
| `unknown-capability` | `name` is not `network`/`storage`/`wallet` |
| `duplicate-capability` | A capability declared twice |
| `missing-reason` | `reason` is empty or all whitespace |
| `bad-scope` | `scope` has the wrong kind, is empty, contains a wildcard, or has a malformed host |
| `non-ascii-host` | Host name outside ASCII |

### Behaviour

| Code | When |
|---|---|
| `bad-action-id` | Action id violates its constraints |
| `duplicate-action` | An action declared twice |
| `action-host-not-granted` | A behaviour calls a host outside the requested capability |

### Interface

| Code | When |
|---|---|
| `external-image` | An image `source` points at the network |
| `text-too-long` | String over 4,096 characters |
| `too-deep` | Tree over 64 levels |
| `too-many-nodes` | Tree over 10,000 nodes |
| `not-a-container` | A leaf node was given children |

### Cryptography

| Code | When |
|---|---|
| `bad-signature` | Either half of the signature is invalid |
| `bad-length` | Key or signature of the wrong length |
| `bad-key` | Key unusable |

## What a code does NOT say

A code says **why something was rejected**. It does not say **which half of the
signature failed**, nor **which files exist in the package**. An implementation
**SHOULD NOT** return more detail than an app author needs to fix the problem —
for a package under probing, every detail is a clue.
