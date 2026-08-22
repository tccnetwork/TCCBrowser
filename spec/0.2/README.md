# The TCC Standard — version 0.2

> ## ⛔ THIS IS A DRAFT. NO IMPLEMENTATION SATISFIES IT. IT IS NOT CONFORMANCE-BEARING.
>
> This is the **opposite** of 0.1's status, and the difference is the whole
> point of the label.
>
> [`spec/0.1/README.md`](../0.1/README.md) says 0.1 was *extracted from the
> running reference implementation*. This directory was **written first**, ahead
> of any code, which rule 1 of [`spec/README.md`](../README.md) says is the
> thing standards die of. It is written anyway because the alternative — an
> implementer guessing at layout from an empty vocabulary — is worse. But it is
> labelled honestly:
>
> - **No implementation, including this project's, satisfies any clause here.**
> - **Nothing here may be cited in a conformance claim.** A statement of the
>   form "conforms to TCC 0.2" is meaningless today and will stay meaningless
>   until this directory is rewritten *from* code that runs.
> - **Nothing here may be used to reject a package.** The two error codes it
>   proposes exist nowhere.
> - Its vectors are **not wired into the runner** and are not counted by
>   `tools/kiem-so-lieu.sh`. Nothing passes them; nothing fails them.
>
> **This English text is normative** for the draft. [Bản tiếng
> Việt](vi/README.md) is a translation; where the two disagree, this text
> governs.

## What is here

| | Normative (en) | Translation (vi) |
|---|---|---|
| Interface — layout | [05](05-interface.md) | [05](vi/05-giao-dien.md) |

**That is all there is, and one file is not a version.**

[`spec/0.1/README.md`](../0.1/README.md) promises that everything an
implementation must satisfy is inside its own version directory, and that
nothing outside it is conformance-bearing. A real 0.2 therefore has to **restate
all six documents**, not link back to 0.1 — because 0.1 is not frozen, so a 0.2
requirement resting on 0.1's wording is a requirement that can change without a
version bump, which is the failure
[`VERSIONING.md`](../VERSIONING.md) §1 exists to prevent.

This directory does none of that. Every reference it makes into `0.1/` is
**informative**: it records what this draft assumes is still true. Before this
could be released, `01`–`04` and `06` have to be brought across and a
`06-error-codes.md` of its own has to exist.

## Why layout, and why now

0.1 gives an app two layout words, `flow` and `gap`. Every screen that is not a
single vertical list is unbuildable: a header that stays put while a list
scrolls under it, a sidebar beside content, a row of buttons pushed to the far
end. That is the gap this draft addresses and the only one.

The model is **Flexbox**, in the subset the `taffy` crate implements, because
`taffy` is what the implementation will use. Describing a different model from
the one the reference implementation runs would guarantee the two drift, and
0.1's rule 11 exists because drift is what this project keeps finding.

Two Flexbox behaviours are removed rather than described — shrinking, and growth
weights — and [05](05-interface.md) says which and why in each place.

## What writing this down revealed

Three things that were not visible before the clauses existed:

1. **No pixels forces the entire vocabulary.** 0.1 forbids an app from
   declaring appearance, for a security reason: an app that can set sizes in
   device units can draw something indistinguishable from the browser's own
   chrome. That rules out every length unit, so sizes had to become a **closed
   set of words**. The result is nine words and no numbers — more restrictive
   than any layout system in use, and it is 0.1's security rule that made it so,
   not a preference.
2. **Percentages and content sizing form a cycle**, and something has to
   forbid it. A fraction makes a child depend on its parent; `content` makes a
   parent depend on its children. CSS resolves the cycle by silently discarding
   the percentage. TCC cannot: §3 of [05](05-interface.md) rejects it instead,
   and the same rule is what bounds layout to one pass down and one pass up, so
   no new tree limit is needed.
3. **The conformance format cannot express a layout requirement.** Every
   existing vector says *accept* or *reject*. "These two children are the same
   size" is neither, so [05](05-interface.md) §12 introduces a second kind of
   case that asserts **relations between boxes** — never absolute geometry,
   since implementations choose their own spacing magnitudes.

## Three places this draft is blocked, honestly stated

### 1. CI rule 22 forbids a new version from adding an error code

`tools/kiem-luat-phu-thuoc.sh` rule 22 requires every hyphenated token in
backticks **anywhere under `spec/`** to appear in
[`spec/0.1/06-error-codes.md`](../0.1/06-error-codes.md), or in a short
hand-edited exemption list inside the script.

That rule reads **0.1's** table for **every** version's files. So it is not
possible to name a new error code in any future version's prose, and this draft
names two: `bad-layout` and `bad-scroll`. The gate goes red at exactly that
line and at no other.

The rule is right and its wording is wrong. Its stated purpose is to stop
prose inventing a code that exists in no table — a real defect it caught once
before. Its implementation assumes the standard has exactly one version. The
fix is to resolve a token against the error-code table of **the version
directory the file lives in**, falling back to 0.1's for files outside any
version. That is a change to `tools/`, which this draft does not own, so it is
reported rather than made.

It was **not** worked around by renaming the two codes to single words, which
would pass the check while removing the thing the check is for.
[VERSIONING](../VERSIONING.md) §3 explicitly permits a minor version to add an
error code, so the standard and its CI currently disagree.

### 2. Every other spec rule is hardcoded to 0.1, so this directory is unwatched

Three rules in the same script do the reverse of rule 22 — they simply do not
see this directory:

| Rule | What it checks | Does it constrain `0.2/`? |
|---|---|---|
| 11 | translation does not drift: file count in `spec/0.1/` vs `spec/0.1/vi/`, and identical error-code sets | **No.** Both paths are literal. A missing or skewed `0.2/vi/` would not be noticed. |
| 23 | no requirement in 0.1 rests on a document outside 0.1 | **No.** It walks `spec/0.1/` only, so this draft's `MUST` clauses may lean on files outside `0.2/` — and they do. |
| 10 | every code in the spec exists in the source | **No.** It reads 0.1's table only — which is why the two new codes do not trip it, correctly, since a draft's codes should not exist yet. |
| 12 | no dead links | **Yes**, it walks all of `spec/`. |
| 22 | hyphenated backtick tokens are real codes | **Yes**, see above. |

So this directory is checked for dead links and for error-code names, and for
nothing else. The translation beside it is maintained by hand and by nothing
else. That is worth knowing before trusting it.

### 3. VERSIONING §3 has no row for this kind of change

[`VERSIONING.md`](../VERSIONING.md) §3 classifies changes by their effect on
**packages**: adding a field, removing one, narrowing one. Much of
[05](05-interface.md) is a requirement on the **renderer** — the spacing scale
must increase, overflow must not be clipped, a scroll container must reach its
content, focus must scroll into view. No package can violate any of them, they
change nothing about which packages conform, and §3's table does not classify
them at all.

This matters because such a requirement can be added or weakened without any
row of that table saying a version bump is due, while an implementation a user
relies on quietly changes what it draws. 0.1 already has one of these
("MUST actually render each intent differently") and the same gap applies to it.

## Status

`0.2/` — **draft**, opened 2026-08-22, layout only. Not frozen, not released,
not implemented, not conformance-bearing. See
[`VERSIONING.md`](../VERSIONING.md) §6 for what a release would require; none of
those four items can be written yet.
