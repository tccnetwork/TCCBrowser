# 01 — Package

## On-disk layout

```text
<package-dir>/
├── manifest.json      ← the manifest
├── signature.hex      ← signature, lowercase hex, trailing newline allowed
└── content/           ← EVERYTHING in here feeds the content hash
```

All three **MUST** be present. A package missing any of them is invalid.

Anything **outside** those three **MUST NOT** enter the signature, and an
implementation **MUST NOT** read it when running the app.

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

Quick check: the empty tree **MUST** yield
`af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262…`
(the first 32 bytes are BLAKE3's published KAT for empty input).

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
