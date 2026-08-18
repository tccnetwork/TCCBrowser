# 01 — Package

## On-disk layout

```text
<package-dir>/
├── manifest.json      ← the manifest
├── signature.hex      ← signature, lowercase hex — see "Reading signature.hex"
└── content/           ← EVERYTHING in here feeds the content hash
```

All three **MUST** be present. A package missing any of them **MUST** be
rejected with `missing-file`.

### Reading `signature.hex`

The file contains the signature of [03](03-signature.md) as hex, and nothing
else. An implementation **MUST**:

| Rule | On violation |
|---|---|
| Accept **lowercase** hex digits only | `not-hex` |
| Accept **one optional** trailing `\n` or `\r\n`, nothing more | `not-hex` |
| Reject leading whitespace, inner whitespace, `0x` prefixes | `not-hex` |
| Require exactly `2 × 3373` = 6746 hex digits | `bad-signature-length` |

Uppercase is rejected rather than normalised for the same reason the manifest
forbids unknown fields: two encodings of one value are two things to compare,
and the comparison is what the signature protects.

Anything **outside** those three **MUST NOT** enter the signature, and an
implementation **MUST NOT** read it when running the app.

A package that contains such extra entries is nevertheless **valid**: an
implementation **MUST NOT** reject it for their presence, and there is no error
code for them. Ignoring them is safe precisely because they are unreadable and
unsigned, whereas rejecting them would make a package's validity depend on
whatever a file manager, an editor or a backup tool happened to leave in the
directory. (The row about rejecting unknown entries in the table below is
guidance for a **future container format**, which 0.1 does not define. Inside a
container the same entries stop being harmless: they are parsed before the
signature is checked.)

## A package is BYTES, and transports must not touch them

The signature covers the raw bytes of `manifest.json`; the content hash covers
the raw bytes of every file under `content/`. Nothing is normalised first — not
line endings, not Unicode, not whitespace.

So any transport that "helpfully" rewrites text destroys the package. The one
that will bite first is **git**, which converts LF to CRLF on checkout under
Windows by default: the reference implementation's own example verified on macOS
and Linux and failed on Windows the first time it was tried there, because 4,682
bytes had become 4,713.

An implementation distributing packages **MUST** treat them as opaque bytes.
Whoever builds that distribution should check, before shipping, that a package
signed on one operating system still verifies after a round trip through their
channel on another.

## There is NO container format in 0.1

What this section defines is a **directory**. Version 0.1 defines no archive, no
single-file container, and no `.tccapp` file format. A package is a directory
laid out as above, and that is the whole story.

This is stated loudly because the phrase "a `.tccapp` package" appears in
project documents, and it names something that does not exist yet. An
implementation is conforming without reading a single archive.

**Why it is not defined here.** Rule 1 of [`spec/README.md`](../README.md): the
standard is extracted from running code, never written ahead of it. No container
is implemented, so any format written now would be a guess.

**What a future container must address**, so whoever defines it does not start
from nothing:

| Concern | Why it is not cosmetic |
|---|---|
| It is parsed **before** the signature is verified | The manifest and the signature both live inside the container, so the container parser is exposed to entirely unauthenticated input — the same position `serde_json` is in today, but with a far larger attack surface |
| Path traversal on extraction | The classic archive bug: an entry named `../../etc/passwd`. The path rules below must apply to container entries too, and must be applied **before** anything is written to disk |
| Decompression ratio | A compressed container invites a bomb. The 256 MiB cap below is on the *uncompressed* content and must be enforced during extraction, not after |
| Duplicate entries | Archive formats routinely permit two entries with one name. One reader takes the first, another the last — one signature, two packages |
| Entries outside the three known names | Must be rejected, not ignored, for the reason given in [02](02-manifest.md) about unknown fields |
| Extraction must not be required | Verification should be possible by reading the container, without writing anything to disk |

Until that exists, "build a valid package" means building the directory
described above.

## Paths inside `content/`

Paths are **relative to `content/`** and use `/` as the separator on every
operating system.

A valid path **MUST** satisfy all of:

| Rule | Why |
|---|---|
| Non-empty, at most **1024** characters | |
| No `..` | escapes the package |
| Does not start with `/` | absolute path |
| No `\` | Windows treats it as a separator |
| No `:` | Windows treats it as a drive letter or alternate data stream |
| No `//` | two spellings of one file |
| Does not end with `/` | that is a directory, not a file |
| Is not `.` | |
| No control characters | |

**Duplicates:** two files **MUST NOT** share a path, *including when they differ
only by case*. macOS and Windows treat `Logo.png` and `logo.png` as one file — the
same signature would then yield different results on different machines. Error
code: `case-collision`.

**Symlinks:** `content/` **MUST NOT** contain symbolic links. What gets signed is
the link; what gets read is the target — two different things.

**Size cap:** total content is at most **256 MiB**. Without a cap, building the
canonical form exhausts memory before anything can be verified.

## Canonical form

This is the byte string the content hash is computed over. Two implementations
**MUST** produce the **same bytes** — a one-byte difference means each side's
signatures fail on the other, while both believe they are correct.

Sort all files by **ascending byte order of the path** (compare raw UTF-8 bytes,
**not** a locale-aware collation). Then, for each file, append:

```text
u64 path length     (big-endian, 8 bytes)
path                (UTF-8, no terminator)
u64 content length  (big-endian, 8 bytes)
content             (raw bytes)
```

### Why length prefixes are REQUIRED

Without them, two different trees produce identical bytes:

| Tree | Naive concatenation |
|---|---|
| `{"ab": "c"}` | `abc` |
| `{"a": "bc"}` | `abc` |

Two different packages, one hash, one signature. An attacker swaps the contents
and the signature still verifies. Length prefixes are the only thing stopping it.

### The empty tree

A tree with no files yields the **empty** byte string. Its hash is the hash of the
empty input — see the `canonical` vectors, case "cây rỗng" (empty tree).

## Content hash

**BLAKE3** in XOF mode, first **48 bytes**, written as **lowercase hex**
(96 characters).

```text
content_hash = lowercase_hex( BLAKE3_XOF( canonical_form )[0..48] )
```

Quick check: the empty tree **MUST** yield exactly

```text
af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262e00f03e7b69af26b7faaf09fcd333050
```

The first 32 bytes are BLAKE3's published KAT for empty input, so half of this
value can be checked against the BLAKE3 specification itself rather than
against us.

⚠️ Until 2026-08-18 this document printed that value **truncated with an
ellipsis** and pointed the reader at the conformance vectors. That made the
one self-check in the whole document unusable by anyone reading only the
specification — which is exactly the reader this document exists for.

**Why BLAKE3 rather than SHA-2:** to match the TCC chain, which already uses
BLAKE3. Two hash functions in one ecosystem are two places to get it wrong.

**48 bytes rather than 32:** margin against Grover, and it matches SHA-384's
length so swapping the hash later does not change any field width.

## Verifying a package — ORDER IS A SECURITY PROPERTY

```text
1. Read the three items      → still just bytes; trust nothing yet
2. Verify the SIGNATURE      → until this passes, NOTHING in the manifest is trustworthy
3. Check the entry point exists
4. Ask the user              → the name and reason shown are now backed by the signature
5. Grant capabilities
```

Step 4 **MUST NOT** precede step 2. Asking before verifying means the dialog shows
a name and a reason taken from an **unauthenticated** manifest — whatever the
attacker wrote is what the user reads.
