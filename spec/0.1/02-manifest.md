# 02 — Manifest

`manifest.json` — JSON, UTF-8, at most **64 KiB**.

That cap is checked at **step 0**, before the JSON is even parsed: without it, a
several-hundred-megabyte file is fully parsed before the signature is checked.

JSON keys **MUST NOT** be duplicated. An implementation **MUST** reject rather
than take the last value — otherwise a display tool and a signature verifier can
see two different values.

## Full example

```json
{
  "spec_version": "0.1",
  "id": "com.tcc.vi-du.hello",
  "name": "Xin chào TCC",
  "version": "0.1.0",
  "publisher": "<3968 hex characters>",
  "scheme": "hybrid-ed25519-mldsa65-v1",
  "content_hash": "<96 hex characters>",
  "entry": "ui.json",
  "capabilities": [
    {
      "name": "network",
      "scope": { "kind": "network", "hosts": ["example.com"] },
      "reason": "Load a sample page"
    }
  ],
  "actions": [
    { "id": "tai-trang", "effect": { "kind": "fetch", "host": "example.com", "path": "/" } }
  ]
}
```

## `actions` — the only behaviour an app can declare

An app carries no code, so "what happens when this button is pressed" is a
**declaration in the manifest**, not a script. The UI tree carries an action id
and nothing else; joining that id to a behaviour is the standard's job.

```json
"actions": [
  { "id": "load-items", "effect": { "kind": "fetch", "host": "shop.example", "path": "/items" } }
]
```

| Field | Rule |
|---|---|
| `id` | MUST. Same character set as a UI `action`: lowercase ASCII, digits, `-`, `.` |
| `effect` | MUST. An object tagged by `kind` |

### Effects

`kind` has exactly **one** value in 0.1:

| `kind` | Fields | Meaning |
|---|---|---|
| `fetch` | `host` MUST · `path` MUST | Ask one host for one path |

- `host` **MUST** match a host granted by a `network` capability, under the
  matching rule in [04](04-capabilities.md). An action naming a host outside
  the granted scope **MUST** be rejected with `action-host-not-granted`.
- `path` **MUST** begin with `/`; otherwise `bad-scope`.
- A non-ASCII `host` **MUST** be rejected with `non-ascii-host`, before any
  matching is attempted — see [04](04-capabilities.md) on why a Unicode host is
  refused rather than punycoded.
- An unknown `kind`, or any field not listed, **MUST** be rejected with
  `bad-json`. It is not ignored — see the rule on unknown fields above.

There is deliberately no `kind` for writing, deleting, or opening a link. Every
value added here widens what an app can do without asking again, so the list
grows only with a version bump ([VERSIONING](../VERSIONING.md) §3).

⚠️ Until 2026-08-18 `effect` appeared **only in examples** in this document —
no field table, no rule for unknown `kind`. Two implementers reading it would
have guessed differently, and that is the whole failure this table exists to
prevent.

## Fields

| Field | Type | Required | |
|---|---|---|---|
| `spec_version` | string | MUST | Exactly `"0.1"`. Anything else is rejected, never guessed. |
| `id` | string | MUST | Application id, see below |
| `name` | string | MUST | Name shown to the user |
| `version` | string | MUST | App version; the standard does not constrain its format |
| `publisher` | string | MUST | Public key, lowercase hex |
| `scheme` | string | MUST | Exactly `"hybrid-ed25519-mldsa65-v1"` |
| `content_hash` | string | MUST | 96 hex characters, see [01](01-package.md) |
| `entry` | string | MUST | A path inside `content/` |
| `capabilities` | array | MUST | May be empty |
| `actions` | array | no | Absent means the app only displays information |

### Unknown fields

An implementation **MUST** reject a manifest containing any field this standard
does not define — at the top level, inside a capability request, and inside a
capability scope. Error code `bad-json`.

The reason is not tidiness. **The signature covers every byte of `manifest.json`,
including bytes no rule of this standard reads.** A field nobody validates is a
channel that carries meaning outside the standard: the same signed package means
one thing on an implementation that understands `x-acme-autostart` and another on
one that does not. That is precisely how vendor prefixes broke interoperability
on the web, and a signature makes it worse — the divergent behaviour arrives
looking authentic.

Inside a capability scope it is worse still. A scope reading
`{"kind":"network","hosts":["a.com"],"ports":[443]}` grants port 443 on an
implementation that knows `ports` and **every port** on one that ignores it.
Silently dropping a field can only ever widen a permission, never narrow one.

So: what is signed is exactly what is checked. New fields require a new
`spec_version` — which is why `spec_version` must match exactly and is never
guessed. See [`VERSIONING.md`](../VERSIONING.md).

## `id` — application id

Reverse-domain style. **MUST** satisfy:

- 1–128 characters
- At least **two segments** separated by `.` (so `hello` is invalid,
  `com.tcc.hello` is valid)
- No empty segment
- Only **lowercase ASCII letters, digits, and `.`**

**Why so strict:** a loose id opens the door to lookalikes. `com.tcc.vi` and
`com.TCC.vi` are **two identities that look identical** to a hurried reader, while
a permission store treats them as different apps — or worse, as the same one.

⚠️ **A trap already walked into:** many JSON libraries will deserialize straight
into a newtype wrapper **without calling its validating constructor**. An
implementation **MUST** validate `id` during shape checking, not rely on the
decoding layer. Error code: `bad-app-id`.

## Strings shown to the user

`name`, `version`, and each capability's `reason` appear on a security-critical
screen. They **MUST** pass the following check — they **MUST NOT** contain:

| Class | Examples | Why |
|---|---|---|
| Newline, tab | `\n` `\r` `\t` | breaks single-line dialog layout |
| C0/C1 control | `U+0000`–`U+001F`, `U+007F`–`U+009F` | |
| Bidirectional override | `U+202A`–`U+202E`, `U+2066`–`U+2069`, `U+200E`, `U+200F` | `"app-evil.exe"` renders as `"app-exe.live"` |
| Zero-width | `U+200B`–`U+200D`, `U+FEFF`, `U+2060` | two different strings look identical |
| Empty or all whitespace | | |

**Combining marks:** at most **8 consecutive** marks on one base character.

They cannot be banned outright — Vietnamese depends on them (`ỡ` = `o` + horn +
tilde = 2 marks). But without a cap, 500 marks stacked on one character render as
a **vertical smear covering the screen above** — and in a permission dialog, what
is above is the warning the user must read.

| | Max marks on one character |
|---|---|
| Vietnamese | 2 |
| Thai, Devanagari — heaviest clusters | ~4–6 |
| **This standard's cap** | **8** |
| UAX #15, for interchange | 30 |

UAX #15 allows 30 because it governs **interchange**; this governs **display** on
a screen where a security decision is made.

Error code for all of the above: `unsafe-display-string`.

## `entry`

A path inside `content/`, subject to the constraints in [01](01-package.md). The
file **MUST** actually exist in the package — checked right after the signature,
before bothering the user.

Its contents are the interface tree, see [05](05-interface.md).

Error codes: `bad-entry` (malformed path), `missing-entry` (no such file).
