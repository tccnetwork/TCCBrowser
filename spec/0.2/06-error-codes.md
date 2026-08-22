# 06 — Error codes

> **Draft.** 0.2 is not released. No implementation satisfies it. See
> [README.md](README.md).

## What this file is, and what it is not

A version directory is **self-contained**: a reader implementing 0.2 must not
have to open 0.1 to know what to return. This file does not yet meet that bar —
it states only what 0.2 **adds**. Restating the whole table is part of finishing
0.2, together with `01`–`04`.

Until then, read [0.1's table](../0.1/06-error-codes.md) first. Every rule there
holds here unchanged: codes are matched, not read; a wrapping error reports the
**root cause**; when several rules break at once, the **first** check in the
ordered list wins.

## Codes 0.2 adds

`VERSIONING.md` §3 permits a minor version to add a code. It permits no version
to remove or rename one, and 0.2 removes and renames none.

| Code | When |
|---|---|
| `bad-layout` | A layout declaration that cannot take effect: `size`/`min`/`max` on the root node; `fill` or a fraction on an axis whose parent extent is content-derived; `min` above `max` on the same axis. [§3, §4.3 of 05](05-interface.md) |
| `bad-scroll` | A scroll declaration that cannot take effect or cannot be checked: a scroll container whose scroll-axis extent is content-derived; a scroll container nested inside another on the same axis. [§9 of 05](05-interface.md) |

Both are **refusals of a declaration**, not failures to draw. An implementation
that cannot honour a layout it accepted has a bug; it does not have an error
code. That distinction is the reason neither code appears anywhere in the
renderer requirements of 05.

## Where they sit in the order

0.1's ordered list ends at content verification. Layout is checked **after** the
package is known to be intact and the interface tree is known to parse — a tree
that failed `bad-json` has no layout to check.

| # | Check | Codes |
|---|---|---|
| … | *0.1's checks 1–10, unchanged* | *see [0.1](../0.1/06-error-codes.md)* |
| 11 | Definiteness and clamping | `bad-layout` |
| 12 | Scrolling | `bad-scroll` |

Shape errors stay `bad-json`, exactly as in 0.1: an unknown key, a wrong type, a
word outside the closed vocabulary. `bad-layout` is for a declaration that is
**well-formed and impossible**, never for one that is malformed.
