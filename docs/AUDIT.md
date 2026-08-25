# Auditing this repository

Written for someone who has been handed this and asked to find what is wrong
with it. It tells you where the weak parts are rather than where the strong ones
are, because you can find the strong ones yourself and the point of a review is
the rest.

## Start here, in this order

| | |
|---|---|
| 1 | [`../SECURITY.md`](../SECURITY.md) §3 — **what has never been examined.** Read this before anything else; it is the list of places where nobody has looked |
| 2 | [`../spec/GOVERNANCE.md`](../spec/GOVERNANCE.md) §1 — who wrote this and who has reviewed it. The answer is one party for all of it |
| 3 | [`../SECURITY.md`](../SECURITY.md) §1 — the 40 invariants, each naming the test that holds it |
| 4 | [`../spec/0.1/`](../spec/0.1/) — the standard. English is normative |
| 5 | [`../ARCHITECTURE.md`](../ARCHITECTURE.md) — layering, and the 22 machine-enforced rules |

## The wallet is on another branch

`main` does **not** contain the wallet. `crates/tcc-chain`, `crates/tcc-keystore`,
key derivation, 24-word recovery phrases, the web-wallet import path, the
transaction-confirmation screen and the real macOS Keychain all live on
`giai-doan-3.1`, and `SECURITY.md` on `main` has no invariants covering them.

That means a review of `main` alone skips the riskiest part of the project. This
was raised by an outside reviewer on 2026-08-16 (finding F2) and is stated here
rather than fixed by merging, because merging an unaudited wallet into the branch
people are told to read would trade one problem for a worse one.

```bash
git checkout giai-doan-3.1     # the wallet, and the parts most worth attacking
```

## Reproduce every claim in about five minutes

The two Python cross-checks need two packages that are **not** in any
requirements file, and without them both commands fail with a missing-import
error rather than a useful message (finding F5):

```bash
pip install dilithium-py blake3
```

```bash
cargo test --workspace                              # 382 tests
# Feature-gated code is NOT built by --workspace, and a hand-copied list of the
# flag combinations drifts: by 2026-08-25 the one in CLAUDE.md named a test
# target deleted with the web engine and omitted four combinations CI still ran.
# This script derives them from ci.yml instead — 20 commands, ~10 minutes.
tools/kiem-theo-co.sh
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
tools/kiem-luat-phu-thuoc.sh                        # 24 architecture rules
cargo run -p tcc-conformance                        # 154 conformance vectors, nine groups
cargo run -p tcc-cli -- verify examples/hello-tcc
cargo run -p tcc-fuzz --release                     # byte-mutation fuzzer
python3 conformance/doi-chieu-doc-lap.py            # cross-check vs dilithium-py
python3 conformance/dung-goi-doc-lap.py             # package built in Python, verified by Rust
cargo audit                                         # 0 vulnerabilities, 12 known warnings
tools/kiem-so-lieu.sh                               # the two numbers above must be real
tools/kiem-khoi-ung-dung.sh                         # the product binary, all five window paths
tools/dem-unsafe.sh                                 # `unsafe` in the code actually compiled
```

The commands above run on **macOS, Linux and Windows** in CI. Windows was added
last and broke immediately — not the code, the checkout: git rewrites LF to CRLF
there by default and a signature covers raw bytes, so every signature in the
repository died. `.gitattributes` fixes it here, and `../spec/0.1/01-package.md`
records the general form, which is that any transport normalising text destroys
a package.

⚠️ **A block of commands used to stand here** — adversarial examples driven
through a real WebKit view, which then reported what it saw. They went with the
web engine on 2026-08-23, and nothing replaced them. That matters more than the
commands did: they were the only check where a **third party** looked at what we
drew. `../SECURITY.md` §3.7 lists what else went and what is genuinely weaker.

The lesson attached to them is kept, because it outlives them. Those runs were
`continue-on-error` on Linux for months, behind a comment saying WebKitGTK under
a virtual display does not reliably get a window handle. The comment was wrong.
The failure was `the underlying handle is not available`, which mentions nothing
about GTK and reads exactly like a display that never came up. Running the same
binary three times in one job gave 0/3 — not flaky, failing every time, because
the code called `build(&window)` where Linux requires `build_gtk`. **A check
exempted because "the infrastructure is unreliable" is the best hiding place a
real bug has.**

The renderer draws to **pixels**, with no markup and no web engine anywhere in
the process. It used to be the second of two, kept so that "the interface layer
does not depend on a browser" was a claim something could falsify. On 2026-08-23
the first one was deleted and **no application changed a line** — which is the
strongest evidence for that claim, and also why the claim now has nothing
watching it. Attack it accordingly:

```bash
# The signed example package, drawn entirely in Rust. On macOS turn on
# VoiceOver (Cmd-F5): the screen should read, and its buttons should work.
cargo run -p tcc-shell --features window-tro-nang \
  --example man-hinh-raster examples/hello-tcc
# The permission dialog on that renderer — the screen worth attacking.
cargo run -p tcc-shell --features window-tro-nang \
  --example man-hinh-raster examples/hello-tcc hop-thoai
```

If any of these disagree with what the documents claim, the documents are wrong
and that is itself a finding.

## Where I would attack it

Ranked by where a real flaw is most likely, not by how much code there is.

> **Re-ranked 2026-08-23.** The old list opened with tier 2 — arbitrary web
> pages — as "newest and largest by far". Tier 2 no longer exists; neither does
> the web engine under it. A ranked list that sends a reviewer at a surface that
> was deleted wastes the scarcest thing in an audit, which is the reviewer's
> attention. What replaced it is smaller and younger, and that is not the same
> as safer.

**1. The layout engine, and the vocabulary on top of it.** Newest by a wide
margin: `taffy` arrived 2026-08-22 and the sizing vocabulary the day after. In
the two days since, self-review found **three** defects in it — a declaration
accepted and silently ignored (`scroll`), a fraction that resolved on one axis
and quietly did nothing on the other, and `fill` mapped to the wrong axis so that
it *shrank* a group nobody asked to shrink. All three are the same shape: **the
package declares something and the renderer does something else.** Assume more of
that shape rather than fewer. The tests that now guard it (`moi_loi_khai_...`,
`fill_chia_khoang_trong_cua_cha`) were written after the fact and prove only what
was already found.

**2. The renderer, and the project's only `unsafe`.** It has its own window, its
own hit-testing, and an accessibility adapter that accepts activation requests
from the platform — meaning a request to press a button arrives from **outside
the process**. The single `unsafe` in the repository is there: handing an
`NSView` pointer to AccessKit. Four bugs of one shape were found in this path in
a single day, including one where a queued accessibility request could overwrite
the user's answer to a permission dialog (F3).

⚠️ The cross-renderer test that used to compare twelve screens across two
renderers is **gone** — there is only one renderer to compare. Architecture rule
1 still forbids `tcc-ui` from depending on a renderer, but the strongest check on
that boundary was the second implementation, and it no longer exists.

**3. The unauthenticated parse.** `verify_package` must decode the manifest at
steps 1–2 and only verifies the signature at step 4, because the public key lives
inside the manifest. There is no way to reorder it. Every byte of `manifest.json`
reaches `serde_json` and `validate_shape` with nothing authenticated. Both
fuzzers point here; neither is proof.

**4. `ml-dsa` 0.1.1.** No published independent audit. It is behind a trait so it
can be swapped, and the signature is hybrid so Ed25519 still stands if it fails —
but the library parses attacker-controlled bytes.

**5. Font parsing, which is new and easy to miss.** `ttf-parser` sits between a
signed package and the screen, and it parses real files from the operating
system. It is unmaintained (RUSTSEC-2026-0192). It forbids `unsafe` itself, so
the realistic failure is a panic rather than memory corruption — and a panic in
the draw path takes the window down while a user is reading a transaction they
are about to sign. `../SECURITY.md` §3.5b has the whole chain, including what was
checked and what was only recalled.

**6. Anything the tests assert about *appearance*.** Three times now a rule was
declared, tested, green, and wrong on screen: `Tone::Danger` drawn identically to
everything else (B31); the wallet capability that had to look different and did
not; and — 2026-08-23 — text drawn **outside its own box** on Linux while every
box-geometry test passed, because the tests measured boxes and the bug was in
ink. The accessibility tree proves a blind user can hear it; it proves nothing
about what a sighted user sees.

**7. The permission dialog, and the single event loop.** It is drawn by the same
renderer as the app, in the same process — the separation that used to exist
(two engines, two processes) is gone. What holds the two apart now is that `tao`
allows exactly one event loop per process, so an app screen and a permission
dialog cannot both exist (§3.1b). That guarantee is **architectural, not
enforced**: `open_sequence` now swaps screens inside one loop, and anyone adding
a second window breaks the property silently.

### The advisory paragraph that had to be rewritten

A paragraph stood here explaining, with three checkable facts, why
RUSTSEC-2026-0097 against `rand 0.7.3` did not reach the cryptography. Every fact
was true when written. **`rand` is now absent from `Cargo.lock` entirely**
(`grep -c '^name = "rand"' Cargo.lock` → 0): it left with the web engine on
2026-08-23, along with `fxhash`. A careful, correct, checkable paragraph about a
package that is no longer there.

That is worth more to a reviewer than the paragraph was. **Everything in these
documents has a date, and the ones with the most careful reasoning are the ones
most likely to survive past their subject** — because they read as settled.
Re-run the commands rather than trusting the prose; where the two disagree, the
prose is wrong and that is itself a finding.

`cargo audit` today: **0 vulnerabilities, 12 warnings** across 452 dependencies —
nine GTK crates (now pulled by `tao`, the windowing library, not by any web
engine), `proc-macro-error` at build time, and `ttf-parser`. The one worth your
time is `ttf-parser`; see the ranked list above and `../SECURITY.md` §3.5b.

## Known weak, already written down

Do not spend time rediscovering these; they are in `SECURITY.md` §3 with the
reasoning:

- No key registry. **A signature proves integrity, never identity.** Packages
  are self-signed.
- No independent security audit — that is what you are doing.
- `verify` leaks by timing which half of a signature failed (35 µs against
  221 µs). Left in deliberately, with the condition for revisiting stated.
- No constant-time proof for signing; the measurement is a screen, not a proof.
- No container format. A package is a directory.
- No package updates, no WASM.
- **There is no web browsing at all any more**, and that removed defences as
  well as a feature. `../SECURITY.md` §3.7 lists them one by one: markup
  escaping, the content-security policy, the hostile-manifest check driven
  through a real engine, the navigation guards, the address bar. Most defended a
  class of attack that now has no door — but one thing is genuinely weaker, and
  it is stated there: **nothing outside our own code looks at what we draw.**

## The two things this project most needs from you

**A second implementation.** Everything else is documentation. Until an
implementation written by someone else passes
[`../conformance/`](../conformance/), "conformant to TCC 0.1" means "agrees with
this one implementation" and nothing more. The vectors are plain JSON and
[`../conformance/FORMAT.md`](../conformance/FORMAT.md) is the schema.

**A reading of [`../spec/0.1/`](../spec/0.1/) by someone who has not read the
source.** Build a package directory from the specification text alone. If you
need to ask a question, or to look at the code, the specification has failed and
the question is the finding. Auditing the clauses this way already produced four
classes of defect that 237 passing tests were blind to — including four error
codes that could never fire and a requirement the standard gave no means to
satisfy (`../spec/README.md` records them).

## How to report

Not a public issue. Use GitHub's private vulnerability reporting on this
repository, or the TCC IT department.

A report that the **design** is wrong is worth more here than a report that the
code disagrees with the design. The code has 382 tests watching it. The design
has had one pair of eyes.
