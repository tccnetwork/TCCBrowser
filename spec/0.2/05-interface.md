# 05 — Interface: layout

> ## ⛔ DRAFT. NO IMPLEMENTATION SATISFIES THIS. NOT CONFORMANCE-BEARING.
>
> Rule 1 of [`spec/README.md`](../README.md) says the standard is **extracted
> from code that already runs, never written ahead of it**. This file is the
> exception, and it is marked as one on every page: it is written **ahead** of
> the code, so it is a **proposal for clauses a layout system will have to
> satisfy**, not a description of one that exists.
>
> Nothing here may be cited as a requirement, quoted in a conformance claim, or
> used to reject a package. Its vectors
> ([`conformance/vectors/layout.json`](../../conformance/vectors/layout.json))
> are deliberately **not wired into the runner**, so no implementation can be
> said to pass or fail them yet.
>
> **This English text is normative** for the draft. [Bản tiếng
> Việt](vi/05-giao-dien.md) is a translation; where the two disagree, this text
> governs.

## What this file is, and what it is not

This file drafts **one subject: layout.** It is a fragment, not a version.

[`spec/0.1/README.md`](../0.1/README.md) states that everything an
implementation must satisfy is inside its own version directory and that
nothing outside it is conformance-bearing. A real 0.2 therefore has to restate
**every** clause of 0.1, not link back to it. This directory does not do that
yet, so it is not a version — it is a draft of the clauses that the version
would add. Every cross-reference here into `0.1/` is **informative**: it tells
you what this draft assumes is still true, not what 0.2 requires.

Everything in [`0.1/05-interface.md`](../0.1/05-interface.md) that is not about
layout — the six node kinds, the display-string rules, the accessibility rules,
the ban on package-drawn masked inputs, action ids, behaviour lookup — is
assumed unchanged and is **not restated**. That assumption is the largest
unfinished item in this draft.

## Why layout at all, and why this shape

0.1 gives an app two layout words: `flow` (`row` or `column`) and `gap`. That
is enough for a list and nothing else. Every real screen — a header that stays
put while a list scrolls under it, a sidebar beside content, a row of buttons
pushed to the far end — needs at least a way to say *how big* and *where*.

The model drafted here is **Flexbox**, in the subset the `taffy` crate
implements, because the implementation will use `taffy` and a standard that
describes a different model from the one its reference implementation runs is a
standard that will be quietly ignored where the two differ. Two Flexbox
behaviours are deliberately **removed** rather than described, and each removal
says so and says why.

**No pixels.** [0.1](../0.1/05-interface.md) forbids an app from declaring
appearance, and it is a security rule, not a taste one: an app that can set
sizes in device units can draw a button that looks exactly like the browser's
own. So every value below is a **word from a closed set**, never a number.
There is no length unit in this draft, and adding one would reopen that hole.

## 1. The frame, and the root node

The implementation lays the tree into a **frame**: a rectangle it chooses, of
**definite** extent on both axes — finite, greater than zero, and known before
layout begins.

The root node is laid out **as if it were the only child of an implicit group**
with exactly these values:

| | |
|---|---|
| `flow` | `column` |
| `gap` | `none` |
| `padding` | `none` |
| `align_main` | `start` |
| `align_cross` | `stretch` |
| `wrap` | `false` |
| `scroll` | `false` |
| extent | the frame |

Two consequences, both stated because leaving either implicit is the mistake
this project already paid for once — 0.1 lost time to *"is the root node at
depth 0 or depth 1"*:

- **The root node's main axis is vertical**, whatever its own `flow` says. A
  node's `flow` governs its **children**, never itself. A root group with
  `flow: "row"` still has a vertical main axis of its own; its children have a
  horizontal one.
- **The root node's box is the frame.** Its extent is the frame's extent on
  both axes. Declaring `size`, `min` or `max` on the root node is rejected with
  `bad-layout` — the frame is not negotiable, so such a declaration could not
  take effect, and 0.1's reason for rejecting unknown fields applies unchanged:
  the author would believe a property took effect when it did not.

The implicit group is **not a node**. It is not counted against the node limit,
it has no accessibility node, and it is not addressable. Where a rule below
counts ancestors, the implicit group is not one of them.

### The frame always scrolls

Content may exceed the frame. Because the frame is the edge of the drawing
surface, anything painted beyond it cannot be seen by moving anything inside
the tree.

So: the implementation **MUST** make the frame itself scroll far enough that
**every painted box can be brought fully into view**, on both axes.

**Why:** without this, §9's promise — that nothing a package draws can become
unreachable — stops at the top of the tree, which is exactly where the largest
overflow happens. A warning pushed one line past the bottom of the window is a
warning that was never shown.

The frame's scrolling is not a node and does not count for the nesting rule in
§9.3.

## 2. Axes

For a group, `flow` names its **main axis**: `row` → horizontal, `column` →
vertical. The **cross axis** is the other one.

**A node's own main and cross axes are its PARENT group's**, not its own. This
sentence is the one an implementer gets wrong. `size.main` on a child of a
`row` is a horizontal size; the same JSON on a child of a `column` is a
vertical one.

`row` runs in the **reading direction of the interface**, and children are
placed along the main axis in **array order**. There is no way to reverse
either. **Why:** a reversal field would let visual order and array order
disagree, and array order is what the accessibility tree and the keyboard
traversal follow — so the button a user sees under the pointer and the button
the keyboard is on could be different buttons. 0.1 already refuses to let a
package separate what a control looks like from what it does; this is the same
refusal in the second dimension.

⚠️ Geometry in a right-to-left interface is **not drafted**. The conformance
vectors for this file are all specified left-to-right and say so. This is a
gap, not a decision.

## 3. Definiteness — the rule that makes layout terminate

Each node has **two physical axes**, and each of them is either **definite**
(its extent is known without measuring the node's content) or
**content-derived** (its extent is whatever its content needs).

Which of a node's declarations governs which physical axis is decided by the
**parent's** `flow` (§2). Definiteness is therefore tracked **per physical
axis**, not per `main`/`cross` name.

An axis of node **N**, child of group **P**, is **definite** exactly when one of
these holds:

1. N is the root node. Both its axes are definite (§1).
2. The declaration governing that axis in N's `size` is a **fraction** (§4.1),
   **and** P's inner extent on that axis is definite, **and** — if that axis is
   P's cross axis — `P.wrap` is `false`.
3. The declaration governing that axis is `fill`, **and** P's inner extent on
   that axis is definite, **and** — if that axis is P's cross axis —
   `P.wrap` is `false`. (§4.4: on the cross axis `fill` means the whole of the
   parent's inner cross extent, so it resolves exactly as `full` does.)
4. The axis is **P's cross axis**, N declares no `size` on it, `P.align_cross`
   is `stretch`, `P.wrap` is `false`, **and** P's inner cross extent is
   definite. (This is stretching: the single line fills the container, so the
   child's cross extent comes from the container rather than from its content.)

Otherwise the axis is **content-derived**.

The two `wrap` conditions are what §8's last row states in words: inside a
wrapping group a child's cross extent comes from **its line**, and a line is
sized by its content. So no declaration can make it definite.

**A `fill` or a fraction declared on an axis whose parent extent is
content-derived is rejected with `bad-layout`.**

**Why, and this is the load-bearing paragraph of the whole file:** a fraction
makes a child's size depend on its parent; `content` makes a parent's size
depend on its children. Allow both to meet and the dependency is a **cycle** —
the parent cannot be sized until the child is, and the child cannot be sized
until the parent is. CSS resolves such cycles by silently treating the
percentage as automatic, which means the author's declaration is discarded
without a word. TCC cannot do that: 0.1 rejects unknown fields for precisely
this reason, and a declaration that is silently discarded is worse than one
that is silently misspelled, because it looks right in the file.

The rule has a second effect worth naming. With cycles excluded, sizes resolve
in **one downward pass** (definite extents flow from the frame toward the
leaves) followed by **one upward pass** (content-derived extents flow from the
leaves toward the frame). Layout is therefore **linear in node count** and
bounded by 0.1's existing limit of 10,000 nodes. No new tree limit is needed,
and none is added. An implementation that iterates to a fixed point is not
required to, and a package cannot make it.

⚠️ **Definiteness is a property of the declaration, not of the outcome.** A
group whose content happens to be zero-sized is still content-derived. An
implementation **MUST NOT** decide definiteness by measuring — two
implementations measuring different fonts would then accept different packages.

## 4. Sizing

Three optional fields, allowed on **every** node kind, leaf and group alike:
`size`, `min`, `max`.

Each is an object with two optional keys, `main` and `cross`, each holding one
**extent** (§4.1):

```json
"size": { "main": "half", "cross": "fill" },
"min":  { "main": "content" },
"max":  { "cross": "third" }
```

- An unknown key inside any of the three → `bad-json`.
- An object with **neither** `main` nor `cross` → `bad-json`. It declares
  nothing, so one of the two readings of it — "no effect" and "some default" —
  is wrong, and the author cannot tell which.
- A value outside the extent vocabulary, or outside the subset a field permits
  (§4.2) → `bad-json`. The vocabulary is a closed set; the same reasoning 0.1
  gives for `emphasis` applies: an implementation that ignores a size word it
  does not know renders a different screen from the one that was signed.

### 4.1 Extents — the vocabulary, and what each means exactly

There are **nine** extent words. They are the whole vocabulary; there is no
tenth, and none of them is a number.

| Word | Extent |
|---|---|
| `content` | Exactly what the node's own content needs on that axis. For a leaf, the natural size of what it draws. For a group, the extent its children occupy after they are laid out, plus its own `padding` on both edges of that axis. |
| `fill` | An equal share of the parent's **free space** on the parent's main axis (§4.4). |
| `full` | The whole of the parent's **inner** extent on that axis. |
| `half` | One half of it. |
| `third` | One third of it. |
| `quarter` | One quarter of it. |
| `two_thirds` | Two thirds of it. |
| `three_quarters` | Three quarters of it. |
| `none` | **No constraint.** Permitted only in `min` and `max` (§4.2). |

The six words `full`, `half`, `third`, `quarter`, `two_thirds`,
`three_quarters` are the **fractions**.

**A fraction resolves against the parent group's INNER extent on that axis**,
where *inner* means the parent's own extent on that axis **minus its `padding`
on both edges of that axis** (§7).

Three things a fraction is explicitly **not** measured against, each stated
because each is a plausible reading that would make two implementations
disagree:

- **not** the parent's outer extent — padding is removed first;
- **not** what is left after the siblings — `half` is half of the parent whether
  it is the only child or the fifth;
- **not** reduced by `gap` — two `half` children in a group with any `gap` other
  than `none` therefore **overflow their parent, and that is correct**. §9.1
  says overflow is never destroyed, so an overflow is visible to the author who
  caused it. Quietly shrinking the two children to fit would mean neither
  declaration took effect while both appeared to.

⚠️ The word **fraction** here means a ratio of a parent extent. It is not a
number, is not written as one, and no arithmetic on it is available to a
package. `two_thirds` and `three_quarters` are single words in a closed set;
they were chosen because a sidebar and a content pane are the two shapes that
0.1's absence of layout blocked most often, and for no other reason. A version
that needs a seventh fraction adds a **word**, and adding a word is a version
bump ([VERSIONING](../VERSIONING.md) §3).

### 4.2 Which extents each field permits

| Field | Permitted extents | Default |
|---|---|---|
| `size.main` | `content`, `fill`, any fraction | `content` |
| `size.cross` | `content`, `fill`, any fraction | absent — see below |
| `min.main` | `none`, `content`, any fraction | `content` |
| `min.cross` | `none`, `content`, any fraction | `none` |
| `max.main` | `none`, `content`, any fraction | `none` |
| `max.cross` | `none`, `content`, any fraction | `none` |

Anything else is `bad-json`: `fill` in a `min` or a `max` (a minimum "share of
the free space" is not a minimum of anything), and `none` in a `size` (a node
with no extent is not drawable).

Three defaults are not obvious and each is stated on purpose:

- **`size.cross` has no default value; it is ABSENT by default**, which is not
  the same as `content`. Absent is what §3 rule 4 and §6 test for: an absent
  cross size is what lets `align_cross: "stretch"` stretch the node. Writing
  `"cross": "content"` explicitly **turns stretching off** for that node. The
  two are different declarations with different results, and an implementation
  that folds them together renders a different screen.
- **`min.main` defaults to `content`.** A node is not shrunk below what its
  content needs along the main axis. This is Flexbox's automatic minimum size,
  and it is the single most surprising thing in Flexbox: it is why a long
  label makes a row overflow instead of squeezing. It is kept because the
  alternative is a button whose text is cut in half, and 0.1's display-string
  rules exist to stop text being altered between signing and showing.
- **`min.cross` defaults to `none`, not `content`.** The automatic minimum
  applies to the main axis only. Two implementations disagreeing here would
  differ on every wrapped line.

### 4.3 Clamping

Each axis is resolved in this exact order:

1. Take the extent from `size` (or its default).
2. Clamp it to at most `max` on that axis.
3. Clamp it to at least `min` on that axis.

So `min` wins over `max` where they overlap. But an overlap is **not**
reachable, because:

**`min` above `max` on the same axis is rejected with `bad-layout`.**

Comparison is by the ratios the words name, and a fraction is comparable with a
fraction: `half` above `quarter` is a violation. `content` is **not**
comparable with a fraction — its value depends on the content — so
`min: {"main": "content"}` with `max: {"main": "quarter"}` is **accepted**, and
step 3 may then push the node past its own maximum. That asymmetry is deliberate
and is the reason clamping order is written out above rather than left to a
reader.

**Why reject rather than resolve:** a package whose `min` exceeds its `max` has
one of the two doing nothing. 0.1's rule for unknown fields is the same
judgement — a declaration that cannot take effect is refused rather than
absorbed.

### 4.4 `fill`, and free space

`fill` applies on the **parent's main axis only**. On the cross axis, `fill`
means the whole of the parent's inner cross extent (equivalently, `full`); it
is permitted there and means exactly that.

For a group, on its own main axis:

```text
free space = inner main extent
           − sum of the resolved main extents of its children
           − sum of the gaps between them
free space is clamped at zero and never goes negative
```

Every child whose `size.main` is `fill` receives **an equal share** of the free
space, and that share is then clamped by that child's own `min` and `max`
(§4.3).

Boundaries, all stated:

- **A single pass.** If clamping changes a share, the difference is **NOT**
  redistributed to the other `fill` children. The leftover simply stays
  unoccupied. Flexbox redistributes by freezing violations and looping; that
  loop is where implementations diverge most, and its result cannot be checked
  by a vector that does not know the content sizes.
- **Zero free space** — every `fill` child gets nothing beyond its `min`, which
  by default is `content`.
- **No weights.** There is no way to say "twice as much". The fractions already
  express proportion, and a weight is a number, which §"No pixels" excludes.
- `fill` on a child of a group whose inner main extent is content-derived is
  `bad-layout` (§3).

### 4.5 There is no shrinking

If the children's resolved extents exceed the parent's inner extent, they are
**not** reduced. They overflow (§9.1).

**Why:** Flexbox's shrink algorithm scales each item by its base size and
re-runs on violation; two implementations agreeing on it to the pixel is
demonstrably hard, and TCC's whole reason for a declarative tree is that the
signed thing and the drawn thing agree. Overflow is safe here in a way it is
not on the web, because §9 guarantees overflow is always reachable and never
silently cut.

## 5. Main-axis alignment

`align_main` on `group` only. Closed set; anything else is `bad-json`.

| Value | Free space goes |
|---|---|
| `start` (default) | after the last child |
| `end` | before the first child |
| `center` | half before the first child, half after the last |
| `between` | divided equally into the **gaps between** children, on top of `gap`; nothing before the first or after the last |
| `evenly` | divided equally into **every** space: before the first, between each pair, after the last |

Boundaries:

- **When free space is zero or negative, all five behave as `start`.** In
  particular, if any child is `fill`, free space is zero after §4.4, so
  `align_main` has no visible effect. This is not an error; it is stated so an
  author who sees no movement knows why.
- **`between` with exactly one child behaves as `start`** — there are no gaps
  to put the space in. With zero children there is nothing to place.
- **`evenly` with zero children** leaves the space empty.
- The space `align_main` distributes is **in addition to** `gap`, never instead
  of it.
- In a wrapping group, `align_main` applies **within each line, separately**
  (§8).

## 6. Cross-axis alignment

`align_cross` on `group` only. Closed set; anything else is `bad-json`.

| Value | Each child is placed |
|---|---|
| `stretch` (default) | occupying the whole cross extent available to it |
| `start` | at the cross-axis start edge |
| `end` | at the cross-axis end edge |
| `center` | midway between them |

Boundaries:

- **`stretch` applies only to children that declare no `size.cross`** (§4.2). A
  child with an explicit cross size keeps it and is placed at the cross-axis
  **start** edge.
- The "cross extent available" is the group's inner cross extent when
  `wrap` is `false`, and **the line's** cross extent when `wrap` is `true`
  (§8).
- There is **no per-child override.** Flexbox has one; this draft does not,
  because nothing in the reference screens needed it and rule 1 of
  [`spec/README.md`](../README.md) forbids specifying ahead of the code. If one
  is added it is a version bump.

## 7. Padding

`padding` on `group` only. Same closed vocabulary as `gap`: `none`, `small`,
`medium`, `large`. **Default `none`.** Anything else is `bad-json`.

Padding applies to **all four edges equally**. There is no per-edge padding, for
the same reason there is no per-child cross alignment: nothing has needed it,
and rule 1 forbids inventing it here.

- Padding is **inside** the node's own extent: a group of extent E with padding
  P has an inner extent of E − 2P on each axis. A group whose extent is
  `content` is the reverse: its extent is its content plus 2P.
- The **inner** extent is what fractions resolve against (§4.1), what free
  space is computed from (§4.4), and what children are placed within.

### 7.1 The scale must be a scale

The **magnitudes** of `none`/`small`/`medium`/`large` are the implementation's
choice — 0.1 forbids the app from setting them. But two requirements make the
words mean something:

- `none` **MUST** be exactly zero.
- The four **MUST** be strictly increasing: `none` < `small` < `medium` <
  `large`.

**Why:** 0.1 already states that an implementation "MUST actually render each
intent differently", because otherwise the intent layer "reduces to a code
comment". The same argument applies here and 0.1 does not make it: nothing in
0.1 stops an implementation from drawing `gap: "large"` and `gap: "small"`
identically, at which point the only spacing vocabulary the standard has says
nothing. **This draft therefore imposes the same two requirements on `gap`**,
which is a new requirement on implementations of an existing 0.1 field. See
§11.2 — [VERSIONING](../VERSIONING.md) §3 has no row for this kind of change.

These are requirements on the **renderer**, not on the validator. No package can
violate them, so they carry no error code (§11.3).

### 7.2 Padding and scrolling

When a group scrolls (§9), its padding belongs to the **scrolled content**, not
to the frame around it. At maximum scroll, the padding on the end edge
**MUST** still be visible between the last child and the container's edge.

**Why:** this is a bug real browsers shipped for years — content scrolled to the
end touching the container edge, so a reader cannot tell whether the last line
is the last line or merely the last line *drawn*. For a screen that ends in a
consequence ("this will delete every key"), "is there more below?" is not a
cosmetic question.

## 8. Wrapping

`wrap` on `group` only. A JSON boolean; **default `false`**. Anything that is
not a boolean — including `"true"` — is `bad-json`.

When `wrap` is `true` and the children's total main extent exceeds the group's
inner main extent, children continue on a new **line**, offset along the cross
axis.

| | |
|---|---|
| Order | Lines are filled in **array order**. A child never moves to an earlier line, and never past a later sibling. |
| A child too big for a line | occupies a line **alone** and overflows it (§9.1). It is not shrunk (§4.5) and not cut. |
| `gap` | applies **both** between children on a line **and** between lines. Flexbox splits these into two properties; this draft has one word and it governs both. |
| `align_main` | applies **within each line, independently**. A short last line is aligned on its own. |
| `align_cross` | applies **within each line**: each line's cross extent is that of its tallest (or widest) child, and `stretch` stretches to the **line**, not to the container. |
| Line packing | Lines are packed from the cross-axis **start**, with `gap` between them. There is **no** control over how leftover cross space is distributed between lines — Flexbox's line-alignment property has no equivalent here. |
| `flow: "column"` + `wrap` | wraps into **columns**: the main axis is vertical, so a new line is a new column, offset horizontally. |
| Cross extents inside | A child of a wrapping group has a **content-derived** cross axis, because a line's cross extent comes from its content. A `fill` or a fraction on that axis is therefore `bad-layout` (§3). |

### 8.1 Two rules the reference renderer already applies

Rule 1 of [`spec/README.md`](../README.md) is that a clause is extracted from
code that **runs**. These two run today, in this repository's reference renderer. They are written
here because a reader who implements everything above and omits them produces a
**visibly different** screen from the same package — which is the exact thing a
standard exists to prevent.

**A group nested inside a row occupies its own line.** A `group` child of a
`flow: "row"` group starts a new line and takes the whole of it, regardless of
how little it needs.

This is a **0.1-compatibility default, and 0.2 supersedes it.** A 0.1 tree has no
size vocabulary at all, so the renderer had to choose for the app; §4 gives the
app the words to say it itself. An implementation of 0.2 **MUST** apply this rule
to a group whose `size.main` is absent, and **MUST NOT** apply it to one that
states a `size.main` — stating a size is the app taking the decision back.

**Buttons alone on a line are drawn at equal main extent — but only if they still
fit.** When every child on a completed line is a `button`, and there are at least
two, an implementation **MUST** draw them all at the widest one's extent, unless
doing so would make the line exceed the container, in which case it **MUST** draw
them at their natural extents.

This one is **not** a compatibility default and 0.2 does not supersede it: it is
a security requirement, and it holds however the app sizes its buttons.

> The transaction-confirmation screen deliberately gives both buttons the same
> tone, because making "Sign" stand out pushes the user one way at the most
> dangerous moment. Extent pushes too. A button visibly larger than the one
> beside it is the same nudge expressed as geometry instead of colour — and
> geometry is not covered by the tone vocabulary, so nothing else here stops it.

The escape clause is not a nicety. The reference renderer widened
unconditionally, and on 2026-08-21 a button was measured at 681.8→1008.7 on an
image 640 wide: **not one pixel of it was drawn**, and hit-testing still returned
it. The user clicks blank space and a button they have never seen runs. A row of
uneven buttons is better than one invisible button that works.

Both rules are watched by geometry vectors (§12) and carry no error code
(§11.3): a package cannot violate either.


## 9. Overflow and scrolling

### 9.1 Content is never destroyed

Content that exceeds its container is, by default, **drawn beyond the
container's edge** and not cut. This is Flexbox's visible-overflow default, and
here it is a requirement rather than a default: an implementation **MUST NOT**
clip a node's box except where §9.2 permits it.

**Why — this is the security clause of this file.** Clipping edits a
user-visible string after it was signed. A `tone: "danger"` button labelled
*"Delete every key on this device"* clipped at its container's edge reads
*"Delete every key"*, and 0.1 spends a whole section forbidding a package from
altering what a user reads immediately before acting. An implementation that
clips is doing to the label exactly what 0.1 forbids the package to do.

⚠️ Do not read this as "overflow is fine". It is *visible*, which is what makes
it the author's problem rather than the user's.

### 9.2 Scroll containers

`scroll` on `group` only. A JSON boolean; **default `false`**. Anything that is
not a boolean is `bad-json`. `scroll` on a leaf node is an unknown field on that
kind and is `bad-json`, per 0.1.

A group with `scroll: true` is a **scroll container**. It **is** permitted to
clip its content, and in exchange it takes on an obligation:

- Its **scroll axis** is its **main axis** when `wrap` is `false`, and its
  **cross axis** when `wrap` is `true`. (Wrapping removes main-axis overflow by
  construction and moves it to the cross axis, so scrolling the main axis would
  scroll nothing.)
- **Its extent on the scroll axis MUST be definite** (§3). If it is
  content-derived, the container grows with its content, never overflows, and
  therefore never scrolls: `bad-scroll`. Note that the default `size.main` is
  `content`, so **declaring `scroll` forces an explicit bounded size** on that
  axis. This is the intended shape, not an accident of the defaults.
- The implementation **MUST** be able to bring **every** part of its content
  fully into view. A scroll container that clips content it cannot reach is not
  conforming.
- The scroll offset **MUST** start at the scroll axis's **start** on first
  layout. **Why:** starting anywhere else hides the beginning of the content,
  and the beginning is where a heading, a warning, or a `warning` emphasis
  belongs.
- Moving keyboard focus to a node inside a scroll container **MUST** scroll that
  node into view. **Why:** otherwise focus sits on a button that is not on
  screen, and the next key press activates something the user cannot see.
- A node scrolled out of view is **still in the accessibility tree**, still has
  its role and label, and 0.1's requirement that every node yield an
  accessibility node is unchanged. Scrolling is not a way to remove a node.

### 9.3 Scroll containers do not nest on the same axis

A scroll container **MUST NOT** have another scroll container among its
ancestors with the **same** scroll axis. Violating this is `bad-scroll`.

Nesting on **different** axes is permitted and is the normal shape of a
horizontal strip inside a vertical page.

Ancestry is counted over the **whole tree**, not just the immediate parent, and
the implicit group of §1 and the frame's own scrolling (§1) are **not**
ancestors for this purpose — neither is a node, so a root group with
`scroll: true` is legal.

**Why:** two reasons, either sufficient.

1. §9.2 promises every part of the content can be brought into view. With two
   scrollers on one axis, whether a given box can be reached depends on the
   *positions* of both, which is a run-time property — so the promise stops
   being checkable at all, and an unchecked promise is what
   [`spec/README.md`](../README.md) says a clause without a check is.
2. Nested same-axis scrollers are the standard way to swallow a scroll gesture:
   the inner container consumes the movement, and the outer content — including
   whatever sits below the fold — is never reached by a user who does not know
   to move the pointer first.

## 10. What this draft does NOT have

Stated so nobody assumes otherwise, in the manner of
[0.1's list](../0.1/README.md):

- **No lengths, and no numbers of any kind** in layout. Nine extent words, two
  booleans, four spacing words.
- **No grid.** `taffy` implements Grid; this draft describes only its Flexbox
  half, because nothing has been built on Grid here and rule 1 forbids
  specifying ahead of the code.
- **No shrinking** (§4.5) and **no growth weights** (§4.4).
- **No per-child cross alignment** (§6) and **no line alignment** (§8).
- **No per-edge padding** (§7), and no margins at all. Space between siblings is
  `gap`; space inside a parent is `padding`. A third mechanism is a third way to
  express the same thing and a third place for two implementations to differ.
- **No aspect ratio**, so an `image` sizes by whichever axis is definite and
  this draft **does not say what happens to the other one**. That is a real hole
  and the most likely first addition.
- **No right-to-left geometry** (§2).
- **No overflow on one axis only.** `scroll` is one boolean, and §9.2 derives
  the axis. A container that must scroll on both is not expressible.

## 11. Error codes

### 11.1 Codes that already exist and are reused unchanged

| Code | Used here for |
|---|---|
| `bad-json` | Any unknown field, any unknown key inside `size`/`min`/`max`, any value outside a closed set, any non-boolean `wrap`/`scroll`, an empty `size`/`min`/`max` object, and `scroll` or `padding` on a node kind that has no such field |

`bad-json` is in [0.1's table](../0.1/06-error-codes.md) and 0.1 already routes
"a field this standard does not define" and "a node kind not in the standard"
to it. Every **shape** violation in this file is that same class, and giving it
a new code would split one condition across two codes for no gain.

### 11.2 Two codes that are NEW

Neither is in 0.1's table. Neither exists in any implementation. Both are
proposed by this draft and nothing more:

| Code | When |
|---|---|
| `bad-layout` | A layout declaration that cannot take effect: `size`/`min`/`max` on the root node (§1); `fill` or a fraction on an axis whose parent extent is content-derived (§3); `min` above `max` on the same axis (§4.3) |
| `bad-scroll` | A scroll declaration that cannot take effect or cannot be checked: a scroll container whose scroll-axis extent is content-derived (§9.2); a scroll container nested inside another on the same axis (§9.3) |

Both follow 0.1's naming, which is a `bad-` prefix for "this field's value
violates its constraints" (`bad-path`, `bad-app-id`, `bad-scope`, `bad-entry`,
`bad-action-id`). Both are **cross-field** conditions — they cannot be decided
by looking at one field alone — which is why they are not `bad-json`;
0.1 sets the same precedent with `action-host-not-granted`, a cross-field
condition with its own code and the stated reason that a generic code "says
nothing and cannot be matched by the conformance suite".

⚠️ **Adding these two codes is what makes this draft fail CI rule 22**, which
requires every hyphenated token in backticks anywhere under `spec/` to appear in
`spec/0.1/06-error-codes.md`. That rule reads **0.1's** table for **every**
version's files, so it forbids any future version from adding any error code at
all. The conflict is recorded in [README](README.md) rather than avoided by
renaming the codes, because a code named to slip past a check is a check that
has stopped working.

### 11.3 Clauses that carry no code, and why that is not an omission

Several requirements here bind the **implementation**, not the package:
§7.1 (the spacing scale must increase), §9.1 (no clipping), §9.2 (reachability,
initial offset, focus), §1 (the frame scrolls), §8.1 (nested groups take a line;
buttons alone on a line are drawn equal). No package can violate them, so there
is nothing to reject and no code to report.

0.1 has the same shape — "MUST actually render each intent differently" has no
code either — and
[`spec/README.md`](../README.md)'s audit names this exact category as the first
kind of defect it found: *a clause with nothing watching it*. These clauses are
watched by the **geometry vectors** in §12, which is the only reason this draft
is allowed to state them.

### 11.4 Precedence

The layout checks belong to step **10** of the precedence table in
[0.1's 06](../0.1/06-error-codes.md) — "Interface file, capabilities,
behaviour". Within the interface file, they run in this order, and an
implementation **MUST** report the first to fail:

| # | Check | Codes |
|---|---|---|
| 1 | Shape: fields, keys, closed sets, booleans | `bad-json` |
| 2 | Display strings and 0.1's tree limits | `unsafe-display-string`, `text-too-long`, `too-deep`, `too-many-nodes` |
| 3 | Definiteness and clamping (§3, §4.3) | `bad-layout` |
| 4 | Scrolling (§9.2, §9.3) | `bad-scroll` |

Shape precedes everything for 0.1's stated reason: nothing else can be decided
until the tree is known to be a tree. Definiteness precedes scrolling because
`bad-scroll` for a content-derived scroll axis is stated **in terms of**
definiteness, so reporting it before definiteness has been established would be
reporting a conclusion drawn from an unchecked premise.

## 12. Conformance vectors

Every clause above has at least one vector in
[`conformance/vectors/layout.json`](../../conformance/vectors/layout.json), per
rule 2 of [`spec/README.md`](../README.md). Each vector names the section it
checks.

That file holds **two** kinds of case, and the second kind is new:

- **`cases`** — accept/reject, in the shape
  [`ui.json`](../../conformance/vectors/ui.json) already uses: `tree`,
  `expect_pass`, and `code` on a rejecting case. Any existing runner can read
  these.
- **`geometry`** — assertions about the **boxes** layout produces. These are
  new, because the existing format cannot express a layout requirement at all:
  it can only say accept or reject, and "the two `fill` children are the same
  size" is neither.

Geometry cases assert **relations between boxes**, never absolute appearance,
so they hold whatever magnitudes an implementation picks for `gap` and
`padding` — which 0.1 leaves to the implementation and this draft does not take
back. The full key-by-key description of the geometry shape is in that file's
own `format` object; it is written there rather than here because
[`conformance/FORMAT.md`](../../conformance/FORMAT.md) belongs to the whole
repository and this draft owns none of it.

⚠️ The geometry vectors are **not wired into this repository's runner** and
cannot be, because no implementation performs layout yet. They are executable by
a reader and inert here. That is the honest state of a draft written ahead of
its code, and it is the reason this file may not be cited as a conformance
requirement.
